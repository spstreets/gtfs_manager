use chrono::Utc;
use druid::im::Vector;
use druid::kurbo::{BezPath, Circle, ParamCurveNearest, Shape};
use druid::piet::{
    CairoImage, CairoRenderContext, Device, FontFamily, ImageFormat, InterpolationMode, Text,
    TextLayoutBuilder,
};
use druid::widget::{prelude::*, CrossAxisAlignment, LabelText, LensWrap};
use druid::widget::{Align, Button, Checkbox, Controller, Flex, Label, List, TextBox};
use druid::{
    Affine, AppDelegate, AppLauncher, BoxConstraints, Color, Cursor, Data, Env, Event,
    FontDescriptor, Handled, LayoutCtx, Lens, LensExt, LocalizedString, MouseButtons, MouseEvent,
    Point, Rect, RenderContext, Selector, Size, TextLayout, Vec2, Widget, WidgetExt, WindowDesc,
};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use rgb::RGB;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::ops::Div;
use std::rc::Rc;

use crate::app_delegate::*;
use crate::data::*;

// bitmaps large than 10,000 x 10,000 will crash. This no longer seems to be a problem, was possibly because of the way we were drawing to it or something rather than an inherent problem with bitmaps of that size. 20,000 does add about 2GB to the memory use of the app though, so not a perfect solution. This is possibly why we will want immediate mode to kick in at some point?
// why is different sizes a problem?
const REFERENCE_SIZE: usize = 1_000;
// const BITMAP_SIZE: usize = 1_000;
const BITMAP_SIZE: usize = 1_000;
const BITMAP_SIZE_LARGE: usize = 20_000;
const NUMBER_TILES_WIDTH: usize = 20;
const MINIMAP_PROPORTION: f64 = 0.3;

const PATH_HIGHLIGHTED: f64 = 1.5;
const PATH_BLACK_BACKGROUND_MULT: f64 = 2.;
const PATH_WHITE_BACKGROUND_MULT: f64 = 3.;
const SMALL_CIRCLE_BLACK_BACKGROUND_MULT: f64 = 1.5;
const SMALL_CIRCLE_MULT: f64 = 1.;
const LARGE_CIRCLE_WHITE_BACKGROUND_MULT: f64 = 4.;
const LARGE_CICLE_BLACK_BACKGROUND_MULT: f64 = 3.5;
const LARGE_CIRCLE_MULT: f64 = 3.;

/// For storing a point normalised to [0, 1]. However will not panic if values fall outside [0, 1].
#[derive(Default)]
struct NormalPoint {
    x: f64,
    y: f64,
}
impl NormalPoint {
    const CENTER: NormalPoint = NormalPoint { x: 0.5, y: 0.5 };
    fn from_canvas_point(point: Point, size: Size) -> NormalPoint {
        NormalPoint {
            x: point.x / size.width,
            y: point.y / size.height,
        }
    }
    /// returns a Point relative to the given Size. eg for NormalPoint::CENTER and Size: 1000,1000, we get Point: 500,500.
    fn to_point_within_size(&self, size: Size) -> Point {
        Point::new(self.x * size.width, self.y * size.height)
    }
    fn to_point_within_size_centering_removed(&self, size: Size, zoom: f64) -> Point {
        Point::new(
            (self.x - (zoom * 0.5)) * size.width,
            (self.y - (zoom * 0.5)) * size.height,
        )
    }
    /// tranlates point by a vector in the given space (size)
    fn translate(&self, vector: Vec2, size: Size) -> NormalPoint {
        NormalPoint {
            x: self.x + vector.x / size.width,
            y: self.y + vector.y / size.height,
        }
    }
}

/// returns a bounding box rect for all latlong points. Note (x0,y0) is the bottom left of the rect.
fn min_max_trips_coords(trips: &Vec<Vec<Point>>) -> Rect {
    let x_iter = trips
        .iter()
        .map(|trip| trip.iter().map(|point| point.x))
        .flatten();
    let longmin = x_iter.clone().fold(0. / 0., f64::min);
    let longmax = x_iter.clone().fold(0. / 0., f64::max);
    let y_iter = trips
        .iter()
        .map(|trip| trip.iter().map(|point| point.y))
        .flatten();
    let latmin = y_iter.clone().fold(0. / 0., f64::min);
    let latmax = y_iter.clone().fold(0. / 0., f64::max);
    Rect::new(longmin, latmin, longmax, latmax)
}

#[derive(Default)]
pub struct MapWidget {
    mouse_position: Option<Point>,
    /// (trip_id, color, text_color, path)
    all_trip_paths_combined: Vec<(String, Color, Color, BezPath)>,
    all_trip_paths_from_shapes: Vec<(String, Color, Color, BezPath)>,
    all_trip_paths_from_stop_coords: Vec<(String, Color, Color, BezPath)>,
    // all_trip_paths_bitmap_grouped: Vec<(Rect, Vec<(String, Color, Color, BezPath)>)>,
    all_trip_paths_bitmap_grouped: Vec<(Rect, Vec<usize>)>,
    // hovered_trip_paths: Vec<(String, Color, Color, BezPath)>,
    // hovered_trip_paths: Vec<usize>,
    filtered_trip_paths: Vec<(String, Color, Color, BezPath)>,
    deleted_trip_paths: Vec<(String, Color, Color, BezPath)>,
    // could just store an index here. as long as we don't change the order of all_trip_paths_combined and only add new trips to the end, this shouldn't be a problem. Could we enforce this in a type?
    // selected_trip_path: Option<(String, Color, Color, BezPath)>,
    // selected_trip_path: Option<usize>,
    // this is for updating a stop_time stop_id
    hovered_stop_id: Option<String>,
    // selected_trip_paths: Vec<BezPath>,
    // stop_circles: Vec<Circle>,
    // highlighted_stop_circle: Option<Circle>,
    // selected_stop_circle: Option<Circle>,
    stop_circles: Vec<Point>,
    highlighted_stop_circle: Option<Point>,
    selected_stop_circle: Option<Point>,
    speed: f64,
    down_click_pos: Option<Point>,
    drag_last_pos: Option<Point>,
    // focal_point should be a lat long coord which is then converted as required, in order to preserve focus between zoom levels. but then we have to dertmine what the ORIGIN coord is. better to just have focal point as a point in [0,1] space.
    focal_point: NormalPoint,
    minimap_image: Option<CairoImage>,
    cached_image_small: Option<CairoImage>,
    cached_image_large: Option<CairoImage>,
    cached_image_map: HashMap<ZoomLevel, CairoImage>,
    cached_image_vec: Vec<CairoImage>,
    recreate_bitmap: bool,
    stop_selection_mode: bool,
    // TODO don't need to make vec of coords every time, only need to check what is selected, so maybe store into to separate vecs. also should store in a field to cache, and allow methods on the data to simplify code below.
    // vector of trips (selected, vector of stop coords)
    // pub trips_coords_from_shapes: Vec<Vec<Point>>,
}
impl MapWidget {
    pub fn new() -> MapWidget {
        myprint!("new widget");
        let mut map_widget = MapWidget::default();
        map_widget.speed = 1.;
        map_widget.recreate_bitmap = true;
        map_widget.focal_point = NormalPoint::CENTER;
        map_widget
    }

    fn group_paths_into_rects(&self) -> Vec<(Rect, Vec<usize>)> {
        myprint!("paint: redraw base: group paths");
        let mut all_trip_paths_bitmap_grouped = Vec::new();
        for m in 0..NUMBER_TILES_WIDTH {
            for n in 0..NUMBER_TILES_WIDTH {
                let rect = Rect::from_origin_size(
                    (
                        REFERENCE_SIZE as f64 * m as f64 / NUMBER_TILES_WIDTH as f64,
                        REFERENCE_SIZE as f64 * n as f64 / NUMBER_TILES_WIDTH as f64,
                    ),
                    (
                        REFERENCE_SIZE as f64 / NUMBER_TILES_WIDTH as f64,
                        REFERENCE_SIZE as f64 / NUMBER_TILES_WIDTH as f64,
                    ),
                );
                let mut group_paths = Vec::new();
                // TODO maybe only store the parts of the path actually in the box, so that when checking for hover later, we are not comparing parts of the path that we know are not in the box. A further optimization would be to deduplicate line segments that are shared by all trips in the route
                // no intersection test yet: https://xi.zulipchat.com/#narrow/stream/260979-kurbo/topic/B.C3.A9zier-B.C3.A9zier.20intersection
                for (index, ((id, color, text_color, trip_path))) in
                    self.all_trip_paths_combined.iter().enumerate()
                {
                    let path_bounding_box = trip_path.bounding_box();
                    if path_bounding_box.contains(Point::new(rect.x0, rect.y0))
                        || path_bounding_box.contains(Point::new(rect.x1, rect.y0))
                        || path_bounding_box.contains(Point::new(rect.x1, rect.y1))
                        || path_bounding_box.contains(Point::new(rect.x0, rect.y1))
                    {
                        group_paths.push(index);
                    }
                    // TODO below should produce less false positives than above, thus making searching for hovered paths quicker but doesn't handle segments which travel all the way through a rect. Use below but fall back to above when seg longer than rect
                    // for seg in trip_path.segments() {
                    //     if rect.contains(seg.as_line().unwrap().p0)
                    //         || rect.contains(seg.as_line().unwrap().p1)
                    //     {
                    //         group_paths.push(index);
                    //         break;
                    //     }
                    // }
                }
                all_trip_paths_bitmap_grouped.push((rect, group_paths));
            }
        }
        all_trip_paths_bitmap_grouped
    }
    fn update_single_path_in_grouped_rects(&mut self, trip_index: usize, trip_path: BezPath) {
        myprint!("fn: update_single_path_in_grouped_rects");
        let path_bounding_box = trip_path.bounding_box();
        for (rect, group_paths) in &mut self.all_trip_paths_bitmap_grouped {
            // remove previous references if any
            let new_group_paths = group_paths
                .iter()
                .filter(|index| index != &&trip_index)
                .cloned()
                .collect::<Vec<_>>();
            *group_paths = new_group_paths;

            // check if rect intersects with bounding box and add trip index to group
            if path_bounding_box.contains(Point::new(rect.x0, rect.y0))
                || path_bounding_box.contains(Point::new(rect.x1, rect.y0))
                || path_bounding_box.contains(Point::new(rect.x1, rect.y1))
                || path_bounding_box.contains(Point::new(rect.x0, rect.y1))
            {
                group_paths.push(trip_index);
            }
        }
    }

    fn latlong_to_canvas(latlong: Point, latlong_rect: Rect, canvas_max_dimension: f64) -> Point {
        let latlong_origin_vec = latlong - latlong_rect.origin();
        // NOTE since (x0,y0) is the bottom left of latlong_rect, we need to flip the xaxis to match the top left origin of canvas
        // flip latlong vec since latlong coord system has origin bottom left and canvas uses top left
        let flipped_latlong_origin_vec = Vec2::new(
            latlong_origin_vec.x,
            latlong_rect.height() - latlong_origin_vec.y,
        );
        // below we are effectively pretending that we have extended latlong_rect's shortest side to match it's longest side so it is now square (since this makes no material difference) so that we can then convert is to canvas space, it just means that the portion of the canvas equivalent to the portion of the rect we extended will have no points in it
        (flipped_latlong_origin_vec * canvas_max_dimension / latlong_rect.size().max_side())
            .to_point()
    }

    fn draw_paths_onto_paint_ctx(&self, data: &AppData, ctx: &mut PaintCtx) {
        ctx.save();
        // tranform
        // ctx.transform(Affine::translate(self.focal_point.to_vec2() * -1.));
        // ctx.transform(Affine::scale(zoom));

        // get focal point in context of zoomed canvas, and reverse.
        // eg canvas 100^2, zoom x2, and a center focal point gives the point (-100,-100). So if we start drawing the 200^2 map here then the the center of the map will be at (0,0) as expected
        // what if the paths were sized to a 400^2 canvas/bitmap? say one of these paths took up a (0,0) to (100,100) space then, keeping the same transforms as before, it would cover the same area as before, (-100,-100) to (100,100) so we starting painting the 400^2 a little before the canvas starts but then the final 3/4 is painted after the canvas. ie only the top left quarter of paths will be drawn between (-100,-100) and (100,100). This bitmap is x4 the size of the canvas. what if we just scale again by x100/400? Then the origin would remain (-100,-100). a 100^2 sized path, would now rather than extend to (100,100), reduce to (0,0) at 1/2 (original case) with just translate no scale, and then (-50,-50) at 1/4 so the entire 400^2 paths would end at (100,100), perfect!
        let transformed_focal_point = self
            .focal_point
            .to_point_within_size(ctx.size() * data.map_zoom_level.to_f64())
            // .to_point_within_size(BITMAP_SIZE_REFERENCE as f64 * data.map_zoom_level.to_f64())
            .to_vec2()
            * -1.;
        // now we translate the canvas to the focal point, so following the example we are now painting the point (0,0) at (-100,-100) on the canvas
        ctx.transform(Affine::translate(transformed_focal_point));

        // this makes the focal point the center, rather than top left
        let center_adjust = ctx.size() * 0.5;
        ctx.transform(Affine::translate(center_adjust.to_vec2()));

        // given the paths already sized to the 100^2 canvas, if we draw them now with origin (-100,-100) we would not see anything, so we need to scale the context by the zoom amount, so that drawing with origin (-100,-100) the paths take up 200^2 space so the bottom right quarter covers the canvas
        ctx.transform(Affine::scale(data.map_zoom_level.to_f64()));
        let ctx_max_side = ctx.size().max_side();
        ctx.transform(Affine::scale(ctx_max_side / REFERENCE_SIZE as f64));

        // NOTE ctx.transform() doesn't change ctx.size()
        let path_width = data.map_zoom_level.path_width(ctx.size().max_side());

        let s_circle_bb = path_width * 0.8;
        let s_circle = path_width * 0.6;

        for (_trip_id, color, _text_color, path) in &self.all_trip_paths_combined {
            // for (_trip_id, color, _text_color, path) in &self.all_trip_paths_canvas {
            ctx.stroke(path, color, path_width);
        }

        // for (point, stop) in self.stop_circles.iter().zip(data.stops.iter()) {
        //     ctx.fill(Circle::new(*point, s_circle_bb), &Color::BLACK);
        //     ctx.fill(Circle::new(*point, s_circle), &Color::WHITE);
        // }

        // draw paths
        ctx.restore();
    }
    fn draw_shapes_onto_bitmap_ctx(
        &self,
        data: &AppData,
        ctx: &mut CairoRenderContext,
        bitmap_size: usize,
        zoom_level: ZoomLevel,
    ) {
        ctx.save();
        // NOTE: don't need to transform for focal point and zoom since we are just creating an image and it is then the image itself which will be transformed. Only need to transform from reference size to actual bitmap size

        // NOTE: can't use ctx.size() since it is not actually defined on RenderContext trait and has not otherwise been defined on CairoRenderContext

        // NOTE: can't use current zoom value since we wan't to create bitmaps at different zoom levels in advance.
        // let path_width = data.map_zoom_level.path_width(bitmap_size as f64);
        // path width should use reference size since we are scaling them to the size of the bitmap anyway, else the would be too big/small from the scaling
        let path_width = zoom_level.path_width(REFERENCE_SIZE as f64);
        ctx.transform(Affine::scale(bitmap_size as f64 / REFERENCE_SIZE as f64));
        for (_trip_id, color, _text_color, path) in &self.all_trip_paths_combined {
            ctx.stroke(path, color, path_width);
        }

        // for (point, stop) in self.stop_circles.iter().zip(data.stops.iter()) {
        //     ctx.fill(Circle::new(*point, path_width * 0.4), &Color::BLACK);
        //     ctx.fill(Circle::new(*point, path_width * 0.2), &Color::WHITE);
        // }

        // draw paths
        ctx.restore();
    }
    fn make_bitmap(
        &self,
        data: &AppData,
        ctx: &mut CairoRenderContext,
        zoom_level: ZoomLevel,
    ) -> CairoImage {
        let mut cached_image;
        {
            let bitmap_size = BITMAP_SIZE * zoom_level.to_usize();
            let mut device = Device::new().unwrap();
            // 0.1 makes the image very small
            // let mut target = device.bitmap_target(1000, 1000, 0.1).unwrap();
            let mut target = device.bitmap_target(bitmap_size, bitmap_size, 1.).unwrap();
            let mut piet_context = target.render_context();

            // piet_context.save();
            // piet_context.transform(Affine::scale(1000. / ctx.size().height));
            self.draw_shapes_onto_bitmap_ctx(data, &mut piet_context, bitmap_size, zoom_level);

            piet_context.finish().unwrap();
            let image_buf = target.to_image_buf(ImageFormat::RgbaPremul).unwrap();
            cached_image = ctx
                .make_image(
                    bitmap_size,
                    bitmap_size,
                    image_buf.raw_pixels(),
                    ImageFormat::RgbaPremul,
                )
                .unwrap();
        }
        cached_image
    }

    // TODO base should include any highlights that don't require a hover, eg selection, deleted, since we don't want to draw these cases when panning. But to make this performant, need to keep the base map, draw it, draw highlights on top, then save this image for use when panning or hovering
    fn draw_bitmap_onto_paint_context(&self, data: &AppData, ctx: &mut PaintCtx) {
        ctx.with_save(|ctx: &mut PaintCtx| {
            let rect = ctx.size().to_rect();
            let zoom = data.map_zoom_level.to_f64();
            let transformed_focal_point = self
                .focal_point
                .to_point_within_size(ctx.size() * zoom)
                .to_vec2()
                * -1.;
            ctx.transform(Affine::translate(transformed_focal_point));

            // this makes the focal point the center, rather than top left
            let center_adjust = ctx.size() * 0.5;
            ctx.transform(Affine::translate(center_adjust.to_vec2()));
            // do we need zoom if the BITMAP is already sized to it's zoom level? Yes because we are not scaling the bitmap, but the rect which we are drawing it into
            ctx.transform(Affine::scale(zoom));
            let bitmap = self.cached_image_map.get(&data.map_zoom_level).unwrap();
            ctx.draw_image(
                bitmap,
                // rect.scale_from_origin(zoom),
                rect,
                InterpolationMode::Bilinear,
            );
        });
    }
    fn draw_highlights(&self, data: &AppData, ctx: &mut PaintCtx) {
        myprint!("draw highlights");
        // what is ctx.save() for? doing transforms which we want to be temporary
        ctx.save();

        // tranform

        // get focal point in context of zoomed canvas, and reverse.
        // eg canvas 100^2, zoom x2, and a center focal point gives the point (-100,-100). So if we start drawing the 200^2 map here then the the center of the map will be at (0,0) as expected
        // what if the paths were sized to a 400^2 canvas/bitmap? say one of these paths took up a (0,0) to (100,100) space then, keeping the same transforms as before, it would cover the same area as before, (-100,-100) to (100,100) so we starting painting the 400^2 a little before the canvas starts but then the final 3/4 is painted after the canvas. ie only the top left quarter of paths will be drawn between (-100,-100) and (100,100). This bitmap is x4 the size of the canvas. what if we just scale again by x100/400? Then the origin would remain (-100,-100). a 100^2 sized path, would now rather than extend to (100,100), reduce to (0,0) at 1/2 (original case) with just translate no scale, and then (-50,-50) at 1/4 so the entire 400^2 paths would end at (100,100), perfect!
        let transformed_focal_point = self
            .focal_point
            .to_point_within_size(ctx.size() * data.map_zoom_level.to_f64())
            // .to_point_within_size(BITMAP_SIZE_REFERENCE as f64 * data.map_zoom_level.to_f64())
            .to_vec2()
            * -1.;
        // now we translate the canvas to the focal point, so following the example we are now painting the point (0,0) at (-100,-100) on the canvas
        ctx.transform(Affine::translate(transformed_focal_point));

        // this makes the focal point the center, rather than top left
        let center_adjust = ctx.size() * 0.5;
        ctx.transform(Affine::translate(center_adjust.to_vec2()));

        // given the paths already sized to the 100^2 canvas, if we draw them now with origin (-100,-100) we would not see anything, so we need to scale the context by the zoom amount, so that drawing with origin (-100,-100) the paths take up 200^2 space so the bottom right quarter covers the canvas
        ctx.transform(Affine::scale(data.map_zoom_level.to_f64()));
        let ctx_max_side = ctx.size().max_side();
        ctx.transform(Affine::scale(ctx_max_side / REFERENCE_SIZE as f64));

        let path_width = data.map_zoom_level.path_width(ctx.size().max_side()) * PATH_HIGHLIGHTED;
        let path_bb = path_width * PATH_BLACK_BACKGROUND_MULT;
        let path_wb = path_width * PATH_WHITE_BACKGROUND_MULT;

        let s_circle_bb = path_width * SMALL_CIRCLE_BLACK_BACKGROUND_MULT;
        let s_circle = path_width * SMALL_CIRCLE_MULT;

        let l_circle_wb = path_width * LARGE_CIRCLE_WHITE_BACKGROUND_MULT;
        let l_circle_bb = path_width * LARGE_CICLE_BLACK_BACKGROUND_MULT;
        let l_circle = path_width * LARGE_CIRCLE_MULT;

        // draw paths
        // for (_, color, text_color, path) in &self.filtered_trip_paths {
        //     ctx.stroke(path, &Color::BLACK, path_bb);
        //     ctx.stroke(path, color, path_width);
        // }
        for index in &data.hovered_trip_paths {
            let (_trip_id, color, text_color, path) =
                self.all_trip_paths_combined.get(*index).unwrap();
            ctx.stroke(path, &Color::BLACK, path_bb);
            ctx.stroke(path, color, path_width);
        }
        // dbg!(self.selected_trip_path.as_ref().map(|thing| &thing.0));

        if let Some((index, id)) = &data.selected_trip_id {
            let (trip_id, color, text_color, path) =
                self.all_trip_paths_combined.get(*index).unwrap();
            // dbg!(path);
            ctx.stroke(path, &Color::WHITE, path_wb);
            ctx.stroke(path, &Color::BLACK, path_bb);
            ctx.stroke(path, color, path_width);
            // drawing larger stops on top of path selection
            let stop_times_range = data.stop_time_range_from_trip_id.get(trip_id).unwrap();
            for i in stop_times_range.0..stop_times_range.1 {
                let stop_time = data.stop_times.get(i).unwrap();
                let stop_index = *data.stop_index_from_id.get(&stop_time.stop_id).unwrap();
                let point = self.stop_circles[stop_index];
                let stop = data.stops.get(stop_index).unwrap();
                ctx.fill(Circle::new(point.clone(), s_circle_bb), &Color::BLACK);
                ctx.fill(Circle::new(point.clone(), s_circle), &Color::WHITE);
                if let Some(hovered_stop_time_id) = &data.hovered_stop_time_id {
                    if stop_time.trip_id == hovered_stop_time_id.0
                        && stop_time.stop_sequence == hovered_stop_time_id.1
                    {
                        ctx.fill(Circle::new(point.clone(), l_circle_bb), &Color::BLACK);
                        ctx.fill(Circle::new(point.clone(), l_circle), &Color::WHITE);
                    }
                }
                if let Some(selected_stop_time_id) = &data.selected_stop_time_id {
                    if stop_time.trip_id == selected_stop_time_id.0
                        && stop_time.stop_sequence == selected_stop_time_id.1
                    {
                        ctx.fill(Circle::new(point.clone(), l_circle_wb), &Color::WHITE);
                        ctx.fill(Circle::new(point.clone(), l_circle_bb), &Color::BLACK);
                        ctx.fill(Circle::new(point.clone(), l_circle), &Color::WHITE);
                    }
                }
            }
        }
        ctx.restore();
    }
    fn draw_stop_highlights(&self, data: &AppData, ctx: &mut PaintCtx) {
        myprint!("draw_stop_highlights");
        ctx.save();

        let transformed_focal_point = self
            .focal_point
            .to_point_within_size(ctx.size() * data.map_zoom_level.to_f64())
            // .to_point_within_size(BITMAP_SIZE_REFERENCE as f64 * data.map_zoom_level.to_f64())
            .to_vec2()
            * -1.;
        ctx.transform(Affine::translate(transformed_focal_point));
        let center_adjust = ctx.size() * 0.5;
        ctx.transform(Affine::translate(center_adjust.to_vec2()));
        ctx.transform(Affine::scale(data.map_zoom_level.to_f64()));
        let ctx_max_side = ctx.size().max_side();
        ctx.transform(Affine::scale(ctx_max_side / REFERENCE_SIZE as f64));

        let path_width = data.map_zoom_level.path_width(ctx.size().max_side()) * PATH_HIGHLIGHTED;

        let s_circle_bb = path_width * SMALL_CIRCLE_BLACK_BACKGROUND_MULT;
        let s_circle = path_width * SMALL_CIRCLE_MULT;

        let l_circle_wb = path_width * LARGE_CIRCLE_WHITE_BACKGROUND_MULT;
        let l_circle_bb = path_width * LARGE_CICLE_BLACK_BACKGROUND_MULT;
        let l_circle = path_width * LARGE_CIRCLE_MULT;

        // paint all stops first, then paint hovered stop over the top. This ensures it is not covered by any unhovered stop and avoids needing to compare stop ids every loop iteration (actually we still need to compare stop ids to find the Point, unless eg we save the index of the Point)
        for (point, stop) in self.stop_circles.iter().zip(data.stops.iter()) {
            ctx.fill(Circle::new(*point, s_circle_bb), &Color::BLACK);
            ctx.fill(Circle::new(*point, s_circle), &Color::WHITE);
        }
        if let Some(stop_id) = &self.hovered_stop_id {
            let point = self
                .stop_circles
                .iter()
                .zip(data.stops.iter())
                .find_map(|(point, stop)| {
                    if &stop.id == stop_id {
                        Some(point)
                    } else {
                        None
                    }
                })
                .unwrap();
            ctx.fill(Circle::new(*point, l_circle_bb), &Color::BLACK);
            ctx.fill(Circle::new(*point, l_circle), &Color::WHITE);
        }
        ctx.restore();
    }
    fn draw_minimap(&self, data: &AppData, ctx: &mut PaintCtx) {
        ctx.with_save(|ctx: &mut PaintCtx| {
            let rect = ctx.size().to_rect();
            ctx.transform(Affine::scale(MINIMAP_PROPORTION));
            ctx.fill(rect, &Color::WHITE);
            ctx.draw_image(
                &self.minimap_image.as_ref().unwrap(),
                rect,
                InterpolationMode::Bilinear,
            );

            // paint minimap viewfinder
            let zoom = data.map_zoom_level.to_f64();
            ctx.clip(rect);
            // Not sure why I need to use ctx.size() here rather than eg ctx.size() * MINIMAP_PROPORTION
            let transformed_focal_point = self.focal_point.to_point_within_size(ctx.size())
                - (ctx.size() * 0.5 / zoom).to_vec2();
            // // make path which is minimap sized rect with viewfinder hole in it
            let mut shadow = rect.to_path(0.1);
            // // ctx.clip(shape)
            let inner_rect = rect
                .with_size(rect.size() / zoom)
                .with_origin(transformed_focal_point);
            shadow.move_to(inner_rect.origin());
            shadow.line_to(Point::new(inner_rect.x1, inner_rect.y0));
            shadow.line_to(Point::new(inner_rect.x1, inner_rect.y1));
            shadow.line_to(Point::new(inner_rect.x0, inner_rect.y1));
            shadow.close_path();
            // ctx.clip(inner_rect.scale_from_origin(-1.));

            ctx.fill_even_odd(shadow, &Color::rgba(0., 0., 0., 0.3));
            ctx.stroke(inner_rect, &Color::RED, 4.);
        });
    }

    fn is_path_hovered(
        &self,
        data: &AppData,
        ctx: &EventCtx,
        path: &BezPath,
        mouse_position: Point,
    ) -> bool {
        // let transformed_focal_point = self
        //     .focal_point
        //     .to_point_within_size(ctx.size() * data.map_zoom_level.to_f64());
        // let translated_mouse_position =
        //     (mouse_position.to_vec2() + transformed_focal_point.to_vec2());

        // let translated_mouse_position = (translated_mouse_position
        //     * (REFERENCE_SIZE as f64 / ctx.size().max_side())
        //     / data.map_zoom_level.to_f64())
        // .to_point();

        let fp = self
            .focal_point
            .to_point_within_size(Size::new(1., 1.))
            .to_vec2();
        let b = fp
            - Size::new(
                1. / (data.map_zoom_level.to_f64() * 2.),
                1. / (data.map_zoom_level.to_f64() * 2.),
            )
            .to_vec2();
        let a = mouse_position.to_vec2() / (ctx.size().max_side() * data.map_zoom_level.to_f64());
        let translated_mouse_position = b + a;
        let translated_mouse_position =
            (translated_mouse_position * REFERENCE_SIZE as f64).to_point();

        let path_width = data.map_zoom_level.path_width(REFERENCE_SIZE as f64);
        let path_width2 = path_width * path_width;

        path.segments()
            .any(|seg| seg.nearest(translated_mouse_position, 1.).distance_sq < path_width2)
    }
    fn find_hovered_paths(
        &self,
        data: &AppData,
        ctx: &EventCtx,
        mouse_position: Point,
    ) -> Vector<usize> {
        // converting a mouse Point (in canvas coords) to a Point in REFERENCE_SIZE coords
        // so we are trying to determine if a mouse Point which is bounded canvas coords but what it is hovering in the canvas might have been scaled and/or translated, and is over a path in REFERENCE_SIZE coords.

        // so lets say the map is zoomed x2, the focal point is CENTER, and the canvas is 100^2

        // top left focal point version:
        // so the origin of the map is at (-100,-100) and the end is at (100,100). a path being hovered at (0,0) is actually the center of the map, so should be translated to             (REFERENCE_SIZE/2,REFERENCE_SIZE/2). So we could work out what the normalised point is, ie (0.5,0.5) and then find that point in REFERENCE_SIZE?

        // centered focal point version:
        // so the origin of the map is at (-50,-50) and the end is at (150,150). a path being hovered at (0,0) should be translated to (REFERNCE_SIZE/4,REFERNCE_SIZE/4) and (0.5,0.5) is (REFERENCE_SIZE/2,REFERENCE_SIZE/2)

        // First we want to effectively pan the map so that the maps origin is at (0,0) on both the canvas and coords space, ie remove any panning and adjust the focal point accordingly
        // so the mouse is within a coord space of the same size, but an origin of (0,0)
        // to do this, we get the focal point and adjust for this. For centered focal point, this would be nothing for zoom x1, size/4 for zoom x2, size * 2/5 for x5, size * 4.5/10 for x10 etc. This is not an easy calculation so it seems it would be easier to remove the zoom scale first, then we only have to remove panning
        // if the mouse position is centered, it should not change.
        // if zoom is x1, the mouse position should not change.
        // at (0,0) for x2 zoom (so (0.25,0.25) normalised), we have a ctx*2 canvas, changing to a ctx sized canvas, with the same origin, so the mouse position should be changed to (-ctx.size/4, -ctx.size/4)
        // at (0,0) for x10 zoom (so (0.45,0.45) normalised), we have a ctx*10 canvas (size*4.5/10 out of view on either side of the viewport), changing to a ctx sized canvas, with the same origin, so the mouse position should be changed to (-ctx.size * 0.45, -ctx.size * 0.45)
        // these normal mults are actually all we need for REFERENCE_SIZE
        // x1: (0,0)
        // x2: (0.25,0.25)
        // x4: 1/4 + 1/8 = (0.375,0.375)
        // x5: (0.4,0.4)
        // x10: (0.45,0.45)
        // x20: 9/20 + 1/40 = (0.475,0.475)
        // they are 1/zoom *

        // let mouse_position_no_scale = mouse_position.to_vec2() - self.focal_point - (0.5,0.5) / data.map_zoom_level.to_f64();
        // let norm_mult = 0.5 - 1 / data.map_zoom_level.to_f64();

        // so following the example, below gives us the focal point in the scaled 200^2 ctx which would be (100,100)

        // so following the example, below gives us the (centering removed) focal point in the scaled 200^2 ctx which would be (0,0)
        let transformed_focal_point = self.focal_point.to_point_within_size_centering_removed(
            // Size::new(REFERENCE_SIZE as f64, REFERENCE_SIZE as f64) * data.map_zoom_level.to_f64(),
            ctx.size() * data.map_zoom_level.to_f64(),
            data.map_zoom_level.to_f64(),
        );
        // dbg!(transformed_focal_point);

        // top left focal point version:
        // now we have the focal point of the zoomed map, we want to move the focal point to the origin (so translate everything (100,100)) whilst keeping the mouse pointing at the same thing, so subtract the focal point from the mouse position
        // needs to also take into account scaling from REFERENCE_SIZE
        // let translated_mouse_position =
        //     (mouse_position.to_vec2() + transformed_focal_point.to_vec2());

        // centered focal point version:
        // now we have the focal point of the zoomed map, we want to move the origin of the canvas to (0,0) (or focal point to the center) of the 200^2 canvas whilst keeping the mouse pointing at the same thing, so translate everything (50,50) / add the focal point * 0.5 to the mouse position
        // (0,0) + (100,100) * 0.5 = (50,50)
        // if the focal point is center and zoom is x1 then we want to translate (0,0).
        let translated_mouse_position_pre =
            (mouse_position.to_vec2() + transformed_focal_point.to_vec2());
        // dbg!(mouse_position);
        // dbg!(translated_mouse_position_pre);

        // now we have the mouse position relative to a non-translated, but scaled/zoomed, 200^2 ctx. We want the mouse relative to a REFERENCE_SIZE ctx. so divide it by 2, then multiple by REFERENCE_SIZE / ctx.size().max_side
        // let translated_mouse_position = (translated_mouse_position_pre
        //     * (REFERENCE_SIZE as f64 / ctx.size().max_side())
        //     / data.map_zoom_level.to_f64())
        // .to_point();
        let translated_mouse_position = (translated_mouse_position_pre
            * (REFERENCE_SIZE as f64 / ctx.size().max_side()))
        .to_point();
        // dbg!(translated_mouse_position);

        // a and b are vectors normalised relative to the total size of the map ie ctx.size() * zoom
        // a is the vector from the viewport origin to the mouse position
        // b is the vector from the map origin to the viewport origin
        // a + b therefore gives the normalised vector of the mouse relative to the origin which can be used to place the mouse on the REFERENCE_SIZE ctx.
        let fp = self
            .focal_point
            .to_point_within_size(Size::new(1., 1.))
            .to_vec2();
        let b = fp
            - Size::new(
                1. / (data.map_zoom_level.to_f64() * 2.),
                1. / (data.map_zoom_level.to_f64() * 2.),
            )
            .to_vec2();
        let a = mouse_position.to_vec2() / (ctx.size().max_side() * data.map_zoom_level.to_f64());
        let translated_mouse_position = b + a;
        let translated_mouse_position =
            (translated_mouse_position * REFERENCE_SIZE as f64).to_point();

        let mut hovered_trip_paths = Vector::new();
        // let path_width = data.map_zoom_level.path_width(ctx.size().max_side());
        let path_width = data.map_zoom_level.path_width(REFERENCE_SIZE as f64);
        let path_width2 = path_width * path_width;
        for (i, (rect, path_indexes)) in self.all_trip_paths_bitmap_grouped.iter().enumerate() {
            // for (i, box_group) in self.all_trip_paths_canvas_grouped.iter().enumerate() {
            // let (rect, path_indexes) = box_group;
            if rect.contains(translated_mouse_position) {
                // dbg!(rect);
                // println!("in box: {}", i);
                for index in path_indexes {
                    let (_trip_id, _color, _text_color, path) =
                        self.all_trip_paths_combined.get(*index).unwrap();
                    for seg in path.segments() {
                        // NOTE accuracy arg in .nearest() isn't used for lines
                        // if seg.nearest(mouse_event.pos, 1.).distance_sq < 1. {
                        if seg.nearest(translated_mouse_position, 1.).distance_sq < path_width2 {
                            // dbg!(id);
                            hovered_trip_paths.push_back(*index);
                            break;
                        }
                    }
                }
            }
        }
        hovered_trip_paths
    }
    fn find_hovered_stop_time(
        &self,
        data: &AppData,
        ctx: &EventCtx,
        mouse_position: Point,
        trip_id: String,
    ) -> Option<(String, u16)> {
        // let transformed_focal_point = self
        //     .focal_point
        //     .to_point_within_size(ctx.size() * data.map_zoom_level.to_f64());
        // let translated_mouse_position =
        //     (mouse_position.to_vec2() + transformed_focal_point.to_vec2());

        // let translated_mouse_position = (translated_mouse_position
        //     * (REFERENCE_SIZE as f64 / ctx.size().max_side())
        //     / data.map_zoom_level.to_f64())
        // .to_point();

        let fp = self
            .focal_point
            .to_point_within_size(Size::new(1., 1.))
            .to_vec2();
        let b = fp
            - Size::new(
                1. / (data.map_zoom_level.to_f64() * 2.),
                1. / (data.map_zoom_level.to_f64() * 2.),
            )
            .to_vec2();
        let a = mouse_position.to_vec2() / (ctx.size().max_side() * data.map_zoom_level.to_f64());
        let translated_mouse_position = b + a;
        let translated_mouse_position =
            (translated_mouse_position * REFERENCE_SIZE as f64).to_point();

        if let Some(stop_times_range) = data.stop_time_range_from_trip_id.get(&trip_id) {
            let path_width = data.map_zoom_level.path_width(REFERENCE_SIZE as f64);
            let s_circle_bb = path_width * PATH_HIGHLIGHTED * SMALL_CIRCLE_BLACK_BACKGROUND_MULT;
            for i in stop_times_range.0..stop_times_range.1 {
                let stop_time = data.stop_times.get(i).unwrap();
                let stop_index = *data.stop_index_from_id.get(&stop_time.stop_id).unwrap();
                let point = self.stop_circles[stop_index];

                if Circle::new(point, s_circle_bb).contains(translated_mouse_position) {
                    return Some((trip_id.clone(), stop_time.stop_sequence));
                }
            }
            None
        } else {
            panic!("trip_id not found");
        }
    }
    fn find_hovered_stop(
        &self,
        data: &AppData,
        ctx: &EventCtx,
        mouse_position: Point,
    ) -> Option<String> {
        let fp = self
            .focal_point
            .to_point_within_size(Size::new(1., 1.))
            .to_vec2();
        let b = fp
            - Size::new(
                1. / (data.map_zoom_level.to_f64() * 2.),
                1. / (data.map_zoom_level.to_f64() * 2.),
            )
            .to_vec2();
        let a = mouse_position.to_vec2() / (ctx.size().max_side() * data.map_zoom_level.to_f64());
        let translated_mouse_position = b + a;
        let translated_mouse_position =
            (translated_mouse_position * REFERENCE_SIZE as f64).to_point();

        let path_width = data.map_zoom_level.path_width(REFERENCE_SIZE as f64);
        let s_circle_bb = path_width * PATH_HIGHLIGHTED * SMALL_CIRCLE_BLACK_BACKGROUND_MULT;
        for (stop_circle_point, stop) in self.stop_circles.iter().zip(data.stops.iter()) {
            // let stop_time = data.stop_times.get(i).unwrap();
            // let stop_index = *data.stop_index_from_id.get(&stop_time.stop_id).unwrap();
            // let stop_circle_point = self.stop_circles[stop_index];

            if Circle::new(*stop_circle_point, s_circle_bb).contains(translated_mouse_position) {
                return Some(stop.id.clone());
            }
        }
        None
    }
}

fn bez_path_from_coords_iter<I, P>(coords_iter: I) -> BezPath
where
    I: Iterator<Item = P>,
    P: Into<Point>,
{
    let mut path = BezPath::new();
    for (i, coord) in coords_iter.enumerate() {
        if i == 0 {
            path.move_to(coord);
        } else {
            path.line_to(coord);
        }
    }
    path
}

impl Widget<AppData> for MapWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        match event {
            // Event::Wheel(mouse_event) => {
            //     let mut change = mouse_event.wheel_delta.y;
            //     let multiplier = 2000. / self.speed;
            //     if change > 0. {
            //         // zoom out
            //         self.zoom_level *= multiplier / (change + multiplier);
            //     } else if change < 0. {
            //         // zoom in
            //         self.zoom_level *= (change.abs() + multiplier) / multiplier;
            //     }
            //     ctx.request_paint();
            // }
            Event::MouseDown(mouse_event) => {
                ctx.override_cursor(&Cursor::Pointer);
                self.down_click_pos = Some(mouse_event.pos);
                self.drag_last_pos = Some(mouse_event.pos);
            }
            Event::MouseMove(mouse_event) => {
                // TODO is this the right place to do this?
                data.hovered_stop_time_id = None;

                // panning
                if let Some(drag_start) = self.drag_last_pos {
                    // println!("mouse move: drag");
                    if mouse_event.buttons.has_left() {
                        let drag_vector = mouse_event.pos.to_vec2() - drag_start.to_vec2();
                        // TODO no idea why I need to reverse the drag vector here! the direction of the drag vector is the same that we want to change the focal point...
                        self.focal_point = self.focal_point.translate(
                            drag_vector * -1.,
                            ctx.size() * data.map_zoom_level.to_f64(),
                        );
                        self.drag_last_pos = Some(mouse_event.pos);
                    } else {
                        // we keep drag_start.is_some() even if the mouse has left the viewport, otherwise it is annoying if you slightly move your mouse outside the viewport and you loose your drag and have to click again
                        self.drag_last_pos = None;
                        ctx.clear_cursor();
                    }
                    ctx.request_paint();

                // hovering
                } else {
                    // println!("mouse move: check for highlight");
                    self.mouse_position = Some(mouse_event.pos);

                    // if in normal mode check for path and stop_time hovers
                    if !data.map_stop_selection_mode {
                        // if hovering a stop on a selected path, highlight/englarge it
                        if let Some((index, id)) = &data.selected_trip_id {
                            let (trip_id, color, text_color, path) =
                                self.all_trip_paths_combined.get(*index).unwrap();
                            // println!("mouse move: check for hover: stop");
                            // TODO below is still going too slow and will cause a backup after lots of mouse move events - seems to be fixed now

                            // check if we are over selected path and then check for stop time, else check hovering paths

                            if self.is_path_hovered(data, ctx, path, mouse_event.pos) {
                                data.hovered_trip_paths = Vector::new();

                                data.hovered_stop_time_id = self.find_hovered_stop_time(
                                    data,
                                    ctx,
                                    mouse_event.pos,
                                    trip_id.clone(),
                                );
                            } else {
                                // check if hovering a path
                                // println!("mouse move: check for hover: path");
                                let hovered_trip_paths =
                                    self.find_hovered_paths(data, ctx, mouse_event.pos);

                                if data.hovered_trip_paths != hovered_trip_paths {
                                    // println!("mouse move: highlights changed");
                                    data.hovered_trip_paths = hovered_trip_paths;
                                    ctx.request_paint();
                                }
                            }
                        } else {
                            // check if hovering a path
                            // println!("mouse move: check for hover: path");
                            let hovered_trip_paths =
                                self.find_hovered_paths(data, ctx, mouse_event.pos);

                            if data.hovered_trip_paths != hovered_trip_paths {
                                // println!("mouse move: highlights changed");
                                data.hovered_trip_paths = hovered_trip_paths;
                                ctx.request_paint();
                            }
                        }
                    } else {
                        // should arguably
                        self.hovered_stop_id = self.find_hovered_stop(data, ctx, mouse_event.pos);
                        ctx.request_paint();
                    }

                    self.highlighted_stop_circle = None;
                }
            }
            Event::MouseUp(mouse_event) => {
                let transformed_focal_point = self
                    .focal_point
                    .to_point_within_size(ctx.size() * data.map_zoom_level.to_f64());

                let translated_mouse_position = ((mouse_event.pos.to_vec2()
                    + transformed_focal_point.to_vec2())
                    / data.map_zoom_level.to_f64())
                .to_point();

                // only handle up click if the down click was on the map
                if let Some(click_down_pos) = self.down_click_pos {
                    if mouse_event.pos == click_down_pos {
                        myprint!("mouse_up: same pos");
                        // if mouse inside minimap
                        let minimap_rect =
                            ctx.size().to_rect().scale_from_origin(MINIMAP_PROPORTION);
                        if minimap_rect.contains(mouse_event.pos) {
                            self.focal_point = NormalPoint::from_canvas_point(
                                mouse_event.pos,
                                ctx.size() * MINIMAP_PROPORTION,
                            );
                            self.down_click_pos = None;
                            ctx.request_paint();
                        } else {
                            // select a trip, stop_time, or stop
                            if !data.map_stop_selection_mode {
                                // TODO differentiate between stop click and path click
                                // TODO looping over every stop kills performance. Need to do something like calculate beforehand which stops are within a tile, find which tile the cursor is in and only loop over those stops. At this point, it might also be worth tiling the bitmaps

                                // select trip if we are hovering one or more (if we are hovering a selected trip, there should be no hovered trips)
                                if let Some(index) = data.hovered_trip_paths.get(0) {
                                    let (id, color, text_color, path) =
                                        self.all_trip_paths_combined.get(*index).unwrap();
                                    let route_id = data
                                        .trips
                                        .iter()
                                        .find(|trip| &trip.id == id)
                                        .unwrap()
                                        .route_id
                                        .clone();
                                    let agency_id = data
                                        .routes
                                        .iter()
                                        .find(|route| route.id == route_id)
                                        .unwrap()
                                        .agency_id
                                        .clone();
                                    data.selected_agency_id = Some(agency_id);
                                    data.selected_route_id = Some(route_id);
                                    data.selected_trip_id = Some((*index, id.clone()));
                                    data.selected_stop_time_id = None;
                                    // data.selected_trip_path = Some(*index);

                                    // check if hovering a stop on selected trip
                                } else if let Some((index, id)) = &data.selected_trip_id {
                                    let (trip_id, color, text_color, path) =
                                        self.all_trip_paths_combined.get(*index).unwrap();
                                    if self.is_path_hovered(data, ctx, path, mouse_event.pos) {
                                        if let Some(hovered_stop_time_id) =
                                            &data.hovered_stop_time_id
                                        {
                                            ctx.submit_command(
                                                SELECT_STOP_TIME.with(hovered_stop_time_id.clone()),
                                            );
                                        } else {
                                            ctx.submit_command(SELECT_TRIP.with(trip_id.clone()));
                                        }
                                    } else {
                                        ctx.submit_command(SELECT_NOTHING);
                                    }
                                }

                            // stop selection
                            // adding existing or new stops to existing or new trips
                            // here were are only submitting commands with either and existing stop's id, or a new stops latlong coord. It is the up to the handlers in AppDelegate to determine what to do with them, ie add to an existing trip, or create a new trip
                            } else {
                                // select an existing stop
                                if let Some(hovered_stop_id) = &self.hovered_stop_id {
                                    ctx.submit_command(
                                        EDIT_STOP_TIME_UPDATE.with(hovered_stop_id.clone()),
                                    );
                                    ctx.request_paint();
                                    data.map_stop_selection_mode = false;

                                // add a new stop
                                } else {
                                    // get square latlon
                                    let trips_coords_from_shapes = data.trips_coords_from_shapes();
                                    let long_lat_rect =
                                        min_max_trips_coords(&trips_coords_from_shapes);

                                    // normalise mouse position
                                    let fp = self
                                        .focal_point
                                        .to_point_within_size(Size::new(1., 1.))
                                        .to_vec2();
                                    let b = fp
                                        - Size::new(
                                            1. / (data.map_zoom_level.to_f64() * 2.),
                                            1. / (data.map_zoom_level.to_f64() * 2.),
                                        )
                                        .to_vec2();
                                    let a = mouse_event.pos.to_vec2()
                                        / (ctx.size().max_side() * data.map_zoom_level.to_f64());
                                    let normalised_mouse_position = b + a;

                                    // Note long_lat_rect_square represents a coord system with bottom left origin
                                    // We need to account for the fact the long_lat_rect is not square, so normalised_mouse_position (1,1) would actually fall outside of long_lat_rect. The easiest way to do this is to make long_lat_rect square
                                    let max_side = long_lat_rect.size().max_side();
                                    let long_lat_rect_square = Rect::new(
                                        long_lat_rect.x0,
                                        long_lat_rect.y1 - max_side,
                                        long_lat_rect.x0 + max_side,
                                        long_lat_rect.y1,
                                    );
                                    let latlong = Point::new(
                                        long_lat_rect_square.x0
                                            + long_lat_rect_square.width()
                                                * normalised_mouse_position.x,
                                        long_lat_rect_square.y1
                                            - long_lat_rect_square.height()
                                                * normalised_mouse_position.y,
                                    );

                                    ctx.submit_command(NEW_STOP.with(latlong));

                                    // TODO maybe this is not the best place to do this?
                                    // need to recreate self.stop_circles
                                    let trips_coords_from_shapes = data.trips_coords_from_shapes();
                                    let long_lat_rect =
                                        min_max_trips_coords(&trips_coords_from_shapes);
                                    let latlong_to_bitmap = |coord: Point| {
                                        MapWidget::latlong_to_canvas(
                                            coord,
                                            long_lat_rect,
                                            REFERENCE_SIZE as f64,
                                        )
                                    };
                                    self.stop_circles.push(latlong_to_bitmap(latlong));
                                    ctx.request_paint();
                                    data.map_stop_selection_mode = false;
                                }
                            }

                            self.down_click_pos = None;
                            // NOTE don't need to call ctx.paint() since we are updating data which will trigger a paint
                        }
                    } else {
                        // todo understand why .clear_cursor() doesn't work here
                        ctx.override_cursor(&Cursor::Arrow);
                        self.down_click_pos = None;
                        self.drag_last_pos = None;
                    }
                }
            }
            _ => {}
        }
    }
    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppData,
        data: &AppData,
        env: &Env,
    ) {
        // TODO is this ok or need to loop through and compare items?
        // need to differentiate between visible/selected/zoomed to determine whether we need to set self.redraw_base
        myprint!("update");
        // if selected trip changes
        myprint!("update: check: selected_trip_id");
        if !data.selected_trip_id.same(&old_data.selected_trip_id) {
            println!("{} update: selected_trip: paint", Utc::now());
            // TODO can't mutate data here, make sure below is being handled somewhere else
            // data.selected_trip_path = data.selected_trip_id.as_ref().map(|selected_trip_id| {
            //     self.all_trip_paths_combined
            //         .iter()
            //         .enumerate()
            //         .find(|(index, path)| &path.0 == selected_trip_id)
            //         .map(|(index, path)| index)
            //         .unwrap()
            // });

            // if trip is deselected and route is selected
            if data.selected_trip_id.is_none() {
                if let Some(route_id) = &data.selected_route_id {
                    let trip_ids = data
                        .trips
                        .iter()
                        .filter(|trip| &trip.route_id == route_id)
                        .map(|trip| trip.id.clone())
                        .collect::<Vec<_>>();
                    self.filtered_trip_paths = self
                        .all_trip_paths_combined
                        .iter()
                        .filter(|(id, _color, text_color, _path)| trip_ids.contains(id))
                        .cloned()
                        .collect::<Vec<_>>();
                }
            }
            ctx.request_paint();
        }

        // if new stop_time is hovered
        myprint!("update: check: hovered_stop_time_id");
        if !data
            .hovered_stop_time_id
            .same(&old_data.hovered_stop_time_id)
        {
            myprint!("update: hovered_stop_time_id: paint");
            ctx.request_paint();
        }
        // if new route is selected
        myprint!("update: check: selected_route_id");
        if !data.selected_route_id.same(&old_data.selected_route_id)
            && data.selected_trip_id.is_none()
        {
            myprint!("update: selected_route: paint");
            if let Some(route_id) = &data.selected_route_id {
                let trip_ids = data
                    .trips
                    .iter()
                    .filter(|trip| &trip.route_id == route_id)
                    .map(|trip| trip.id.clone())
                    .collect::<Vec<_>>();
                self.filtered_trip_paths = self
                    .all_trip_paths_combined
                    .iter()
                    .filter(|(id, _color, text_color, _path)| trip_ids.contains(id))
                    .cloned()
                    .collect::<Vec<_>>();
                ctx.request_paint();
            }
        }
        // TODO should also check if agency is selected and highlight it's paths, but this will kill performance everytime we select SPTRANS in order to select a different route... also app should start with SPTRANS unselected

        myprint!("update: check: trips");
        if !data.trips.same(&old_data.trips) {
            myprint!("update: trips: paint");

            // only want to redraw base when a new trip is added, so leave for now
            // self.redraw_base = true;
            ctx.request_paint();
        }

        // check for stop_times which have been edited
        myprint!("update: check: data_stop_time.stop_id");
        // check whether a new stop_time has been added
        // TODO also need to handle case where stop_time is deleted
        if data.stop_times.len() == old_data.stop_times.len() {
            for (i, (data_stop_time, old_data_stop_time)) in data
                .stop_times
                .iter()
                .zip(old_data.stop_times.iter())
                .enumerate()
            {
                // if stop_time has a new stop.id then update paths
                if !data_stop_time.stop_id.same(&old_data_stop_time.stop_id) {
                    myprint!("recreate path from stop coords");
                    // TODO make latlong_to_bitmap() here is expensive and could be avoided
                    let trips_coords_from_shapes = data.trips_coords_from_shapes();
                    let long_lat_rect = min_max_trips_coords(&trips_coords_from_shapes);
                    let latlong_to_bitmap = |coord: Point| {
                        MapWidget::latlong_to_canvas(coord, long_lat_rect, REFERENCE_SIZE as f64)
                    };

                    let (trip_index, trip) = data
                        .trips
                        .iter()
                        .enumerate()
                        .find(|(index, trip)| trip.id == data_stop_time.trip_id)
                        .unwrap();

                    let route = data
                        .routes
                        .iter()
                        .find(|route| route.id == trip.route_id)
                        .unwrap();
                    let RGB { r, g, b } = route.color.0;
                    let color = Color::rgb8(r, g, b);
                    let RGB { r, g, b } = route.text_color.0;
                    let text_color = Color::rgb8(r, g, b);
                    let coords = data.trip_coords_from_stop_coords(data_stop_time.trip_id.clone());

                    let new_path = bez_path_from_coords_iter(
                        coords.iter().map(|coord| latlong_to_bitmap(*coord)),
                    );
                    self.all_trip_paths_combined[trip_index] =
                        (trip.id.clone(), color, text_color, new_path.clone());
                    self.recreate_bitmap = true;

                    // remove existing references to trip in grouped paths, then calculate groups for new path and add those
                    // self.all_trip_paths_bitmap_grouped = self.group_paths_into_rects();
                    self.update_single_path_in_grouped_rects(trip_index, new_path);
                    ctx.request_paint();
                }
            }

        // recreate trip path for trip with stop_time added or deleted
        } else {
            // first ensure we actually have an updated trip, rather than an entirely new trip
            if data.trips.len() == old_data.trips.len() {
                // find which trip has been updated
                let ((trip_index, trip), _) = data
                    .trips
                    .iter()
                    .enumerate()
                    .zip(old_data.trips.iter())
                    .find(|((trip_index, trip), old_trip)| {
                        let data_range = data.stop_time_range_from_trip_id.get(&trip.id).unwrap();
                        let data_size = data_range.1 - data_range.0;
                        let old_data_range =
                            old_data.stop_time_range_from_trip_id.get(&trip.id).unwrap();
                        let old_data_size = old_data_range.1 - old_data_range.0;
                        data_size != old_data_size
                    })
                    .unwrap();
                myprint!("update: recreate trip path for trip with stop_time added or deleted");

                // TODO making latlong_to_bitmap() here is expensive and could be avoided
                let trips_coords_from_shapes = data.trips_coords_from_shapes();
                let long_lat_rect = min_max_trips_coords(&trips_coords_from_shapes);
                let latlong_to_bitmap = |coord: Point| {
                    MapWidget::latlong_to_canvas(coord, long_lat_rect, REFERENCE_SIZE as f64)
                };

                let route = data
                    .routes
                    .iter()
                    .find(|route| route.id == trip.route_id)
                    .unwrap();
                let RGB { r, g, b } = route.color.0;
                let color = Color::rgb8(r, g, b);
                let RGB { r, g, b } = route.text_color.0;
                let text_color = Color::rgb8(r, g, b);
                let coords = data.trip_coords_from_stop_coords(trip.id.clone());

                let new_path =
                    bez_path_from_coords_iter(coords.iter().map(|coord| latlong_to_bitmap(*coord)));
                self.all_trip_paths_combined[trip_index] =
                    (trip.id.clone(), color, text_color, new_path.clone());
                self.recreate_bitmap = true;

                // remove existing references to trip in grouped paths, then calculate groups for new path and add those
                // self.all_trip_paths_bitmap_grouped = self.group_paths_into_rects();
                self.update_single_path_in_grouped_rects(trip_index, new_path);
                ctx.request_paint();
            }
        }

        myprint!("update: check: map_zoom_level");
        if !data.map_zoom_level.same(&&old_data.map_zoom_level) {
            myprint!("update: map_zoom_level: paint");
            ctx.request_paint();
        }

        myprint!("update: check: map_stop_selection_mode");
        if !data
            .map_stop_selection_mode
            .same(&old_data.map_stop_selection_mode)
        {
            myprint!("update: map_stop_selection_mode: paint");
            ctx.request_paint();
        }

        // if !old_data.same(data) {
        //     println!("data has changed?!?!");
        //     ctx.request_layout();
        // }
    }
    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        // rather than changing the size ratio of the widget based on on the shape of what is being drawn, it is best to always keep the widget 1:1 and draw the paths relative to this
        // if bc.is_width_bounded() && bc.is_height_bounded() {
        //     bc.max()
        // } else {
        //     let size = Size::new(300.0, 300.0);
        //     bc.constrain(size)
        // }
        myprint!("layout");
        let size = Size::new(100.0, 100.0);
        bc.constrain(size);
        let max = bc.max().height.min(bc.max().width);
        // Size::new(300.0, 300.0)
        Size::new(max, max)
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        // dbg!("map paint");

        // Clear the whole widget with the color of your choice
        // (ctx.size() returns the size of the layout rect we're painting in)
        // Note: ctx also has a `clear` method, but that clears the whole context,
        // and we only want to clear this widget's area.
        // RenderContext

        // todo encode gtfs coords and painting coords into two distinct types for clarity

        myprint!("paint");
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.clip(rect);
        ctx.fill(rect, &Color::grey(0.4));
        // ctx.fill(rect, &Color::WHITE);

        if self.recreate_bitmap {
            println!("{} paint: redraw base: make image", Utc::now());
            self.recreate_bitmap = false;

            for (_name, zoom_level) in ZoomLevel::radio_group_vec() {
                // panning the map is laggy for *all* zoom levels if we create bitmaps at x10 or greater
                if zoom_level.to_usize() < 10 {
                    self.cached_image_map
                        .insert(zoom_level, self.make_bitmap(data, ctx, zoom_level));
                }
            }
            if self.minimap_image.is_none() {
                self.minimap_image =
                    Some(self.cached_image_map.get(&ZoomLevel::One).unwrap().clone());
            }
        }

        // TODO come up with proper heuristic based on the size of coords or density of paths to determine when to switch to immediate mode, so that it generalises to maps other than SP
        if self.cached_image_map.contains_key(&data.map_zoom_level) {
            myprint!("paint: draw bitmap");

            self.draw_bitmap_onto_paint_context(data, ctx);
            // TODO temporarily drawing highlights here too until we add another cache for non hover highlights
        } else {
            myprint!("paint: draw immediate mode");
            self.draw_paths_onto_paint_ctx(data, ctx);
        }
        if !data.map_stop_selection_mode {
            myprint!("paint: draw highlights");
            self.draw_highlights(data, ctx);
        } else {
            self.draw_stop_highlights(data, ctx);
        }
        myprint!("paint: draw minimap");
        self.draw_minimap(data, ctx);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                // TODO this should obviously be decoupled from widget impl
                let trips_coords_from_shapes = data.trips_coords_from_shapes();
                // let trips_coords_from_shapes = data.trips_coords_from_stop_coords();
                let trips_coords_from_stop_coords = data.trips_coords_from_stop_coords();

                let size = ctx.size();
                // find size of path data
                // NOTE (x0,y0) is the bottom left of the rect
                let long_lat_rect = min_max_trips_coords(&trips_coords_from_shapes);

                let latlong_to_bitmap = |coord: Point| {
                    MapWidget::latlong_to_canvas(coord, long_lat_rect, REFERENCE_SIZE as f64)
                };

                // TODO should be making data in update or on_added, not paint
                // translate trip paths to a given canvas size and store colors
                myprint!("paint: redraw base: make paths");
                self.all_trip_paths_from_shapes = trips_coords_from_shapes
                    .iter()
                    .zip(data.trips.iter())
                    .filter(|(_coords, trip)| trip.visible)
                    .map(|(coords, trip)| {
                        let route = data
                            .routes
                            .iter()
                            .find(|route| route.id == trip.route_id)
                            .unwrap();
                        let RGB { r, g, b } = route.color.0;
                        let color = Color::rgb8(r, g, b);
                        let RGB { r, g, b } = route.text_color.0;
                        let text_color = Color::rgb8(r, g, b);
                        (
                            trip.id.clone(),
                            color,
                            text_color,
                            bez_path_from_coords_iter(
                                coords.iter().map(|coord| latlong_to_bitmap(*coord)),
                            ),
                        )
                    })
                    .collect::<Vec<_>>();

                // there is no point precalculating these because if the stop_time.stop_id has changed then we will need to recreate the path anyway
                // self.all_trip_paths_from_stop_coords = trips_coords_from_stop_coords
                //     .iter()
                //     .zip(data.trips.iter())
                //     .filter(|(_coords, trip)| trip.visible)
                //     .map(|(coords, trip)| {
                //         let route = data
                //             .routes
                //             .iter()
                //             .find(|route| route.id == trip.route_id)
                //             .unwrap();
                //         let RGB { r, g, b } = route.color.0;
                //         let color = Color::rgb8(r, g, b);
                //         let RGB { r, g, b } = route.text_color.0;
                //         let text_color = Color::rgb8(r, g, b);
                //         (
                //             trip.id.clone(),
                //             color,
                //             text_color,
                //             bez_path_from_coords_iter(
                //                 coords.iter().map(|coord| latlong_to_bitmap(*coord)),
                //             ),
                //         )
                //     })
                //     .collect::<Vec<_>>();

                self.all_trip_paths_combined = self.all_trip_paths_from_shapes.clone();
                self.all_trip_paths_bitmap_grouped = self.group_paths_into_rects();

                self.stop_circles = data
                    .stops
                    .iter()
                    .map(|stop| latlong_to_bitmap(stop.latlong))
                    .collect::<Vec<_>>();
            }
            _ => {}
        }
    }
}
