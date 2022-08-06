use chrono::Utc;
use druid::im::Vector;
use druid::kurbo::{BezPath, Circle, ParamCurveNearest, Shape};
use druid::piet::{
    CairoImage, Device, FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder,
};
use druid::widget::{prelude::*, CrossAxisAlignment, LabelText, LensWrap};
use druid::widget::{Align, Button, Checkbox, Controller, Flex, Label, List, TextBox};
use druid::{
    Affine, AppDelegate, AppLauncher, BoxConstraints, Color, Cursor, Data, Env, Event,
    FontDescriptor, Handled, LayoutCtx, Lens, LensExt, LocalizedString, MouseButtons, MouseEvent,
    Point, Rect, RenderContext, Selector, Size, TextLayout, Widget, WidgetExt, WindowDesc,
};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use rgb::RGB;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use crate::app_delegate::*;
use crate::data::*;

// bitmaps large than 10,000 x 10,000 will crash
const BITMAP_SIZE: usize = 1000;
const NUMBER_TILES_WIDTH: usize = 20;

#[derive(Copy, Clone)]
struct PathsRanges {
    longmin: f64,
    latmin: f64,
    longmax: f64,
    latmax: f64,
}
// -> (xmin, ymin, xmax, ymax)
fn min_max_trips_coords(trips: &Vec<Vec<(f64, f64)>>) -> PathsRanges {
    let x_iter = trips
        .iter()
        .map(|trip| trip.iter().map(|point| point.0))
        .flatten();
    let longmin = x_iter.clone().fold(0. / 0., f64::min);
    let longmax = x_iter.clone().fold(0. / 0., f64::max);
    let y_iter = trips
        .iter()
        .map(|trip| trip.iter().map(|point| point.1))
        .flatten();
    let latmin = y_iter.clone().fold(0. / 0., f64::min);
    let latmax = y_iter.clone().fold(0. / 0., f64::max);
    PathsRanges {
        longmin,
        latmin,
        longmax,
        latmax,
    }
}

pub struct MapWidget {
    mouse_position: Option<Point>,
    all_trip_paths_bitmap: Vec<(Color, Color, BezPath)>,
    all_trip_paths_canvas_grouped: Vec<(Rect, Vec<(String, Color, Color, BezPath)>)>,
    // all_trip_paths_canvas_translated: Vec<BezPath>,
    all_trip_paths_canvas: Vec<(String, Color, Color, BezPath)>,
    hovered_trip_paths: Vec<(String, Color, Color, BezPath)>,
    filtered_trip_paths: Vec<(String, Color, Color, BezPath)>,
    deleted_trip_paths: Vec<(String, Color, Color, BezPath)>,
    selected_trip_path: Option<(String, Color, Color, BezPath)>,
    // selected_trip_paths: Vec<BezPath>,
    // stop_circles: Vec<Circle>,
    // highlighted_stop_circle: Option<Circle>,
    // selected_stop_circle: Option<Circle>,
    stop_circles_base: Vec<Point>,
    stop_circles_canvas: Vec<Point>,
    highlighted_stop_circle: Option<Point>,
    selected_stop_circle: Option<Point>,
    // zoom_level: f64,
    speed: f64,
    click_down_pos: Option<Point>,
    drag_last_pos: Option<Point>,
    focal_point: Point,
    minimap_image: Option<CairoImage>,
    cached_image: Option<CairoImage>,
    redraw_base: bool,
    remake_paths: bool,
    redraw_highlights: bool,
    // TODO don't need to make vec of coords every time, only need to check what is selected, so maybe store into to separate vecs. also should store in a field to cache, and allow methods on the data to simplify code below.
    // vector of trips (selected, vector of stop coords)
    pub trips_coords: Vec<Vec<(f64, f64)>>,
}
impl MapWidget {
    pub fn new(speed: f64, offset: Point) -> MapWidget {
        println!("new widget");
        MapWidget {
            mouse_position: None,
            all_trip_paths_bitmap: Vec::new(),
            all_trip_paths_canvas: Vec::new(),
            all_trip_paths_canvas_grouped: Vec::new(),
            // all_trip_paths_canvas_translated: Vec::new(),
            hovered_trip_paths: Vec::new(),
            filtered_trip_paths: Vec::new(),
            deleted_trip_paths: Vec::new(),
            selected_trip_path: None,
            stop_circles_base: Vec::new(),
            stop_circles_canvas: Vec::new(),
            highlighted_stop_circle: None,
            selected_stop_circle: None,
            // zoom_level,
            speed,
            click_down_pos: None,
            drag_last_pos: None,
            focal_point: offset,
            minimap_image: None,
            cached_image: None,
            redraw_base: true,
            remake_paths: true,
            redraw_highlights: true,
            trips_coords: Vec::new(),
        }
    }

    fn calculate_padding(
        width: f64,
        height: f64,
        size: Size,
        zoom: f64,
        zoomed_paint_width: f64,
        zoomed_paint_height: f64,
        paint_width: f64,
        paint_height: f64,
    ) -> (f64, f64) {
        if width > height {
            (
                (size.width - zoomed_paint_width) / 2.,
                (size.height - zoomed_paint_height) / 2.
                    + (zoomed_paint_width - zoomed_paint_height) / 2.,
            )
        } else {
            (
                (size.width - zoomed_paint_width) / 2. + (paint_height - paint_width) / 2. / zoom,
                (size.height - zoomed_paint_height) / 2.,
            )
        }
    }

    // fn long_lat_to_canvas(
    //     point: &(f64, f64),
    //     ranges: PathsRanges,
    //     width: f64,
    //     height: f64,
    //     zoomed_paint_width: f64,
    //     zoomed_paint_height: f64,
    //     focal_point: Point,
    //     zoom: f64,
    // ) -> Point {
    //     let (long, lat) = point;
    //     let x2 = (*long - ranges.longmin) * (zoomed_paint_width / width) - focal_point.x * zoom;
    //     let y2 = (*lat - ranges.latmin) * (zoomed_paint_height / height) - focal_point.y * zoom;
    //     Point::new(x2, y2)
    // }
    fn long_lat_to_canvas(
        point: &(f64, f64),
        ranges: PathsRanges,
        width: f64,
        height: f64,
        paint_width: f64,
        paint_height: f64,
        // focal_point: Point,
        // zoom: f64,
    ) -> Point {
        let (long, lat) = point;
        let x2 = (*long - ranges.longmin) * (paint_width / width);
        let y2 = (*lat - ranges.latmin) * (paint_height / height);
        Point::new(x2, y2)
    }

    // TODO base should include any highlights that don't require a hover, eg selection, deleted, since we don't want to draw these cases when panning. But to make this performant, need to keep the base map, draw it, draw highlights on top, then save this image for use when panning or hovering
    fn draw_base_from_cache(&self, ctx: &mut PaintCtx, rect: Rect, zoom: f64) {
        ctx.with_save(|ctx: &mut PaintCtx| {
            ctx.transform(Affine::translate(self.focal_point.to_vec2() * -1.));
            ctx.transform(Affine::scale(zoom));
            ctx.stroke(rect, &Color::GRAY, 2.);
            ctx.draw_image(
                &self.cached_image.as_ref().unwrap(),
                rect,
                InterpolationMode::Bilinear,
            );
        });
    }
    fn draw_highlights(&self, data: &AppData, ctx: &mut PaintCtx, rect: Rect, zoom: f64) {
        // ctx.with_save(|ctx: &mut PaintCtx| {
        ctx.save();
        ctx.transform(Affine::translate(self.focal_point.to_vec2() * -1.));
        ctx.transform(Affine::scale(zoom));
        for (_, color, text_color, path) in &self.filtered_trip_paths {
            ctx.stroke(path, &Color::BLACK, 5.);
            ctx.stroke(path, color, 3.);
        }
        for (_, color, text_color, path) in &self.hovered_trip_paths {
            ctx.stroke(path, &Color::BLACK, 5.);
            ctx.stroke(path, color, 3.);
        }
        dbg!(self.selected_trip_path.as_ref().map(|thing| &thing.0));
        if let Some((id, color, text_color, path)) = &self.selected_trip_path {
            ctx.stroke(path, &Color::WHITE, 9.);
            ctx.stroke(path, &Color::BLACK, 7.);
            ctx.stroke(path, color, 5.);
            // drawing larger stops on top of path selection
            if let Some(stop_times_range) = data.stop_time_range_from_trip_id.get(id) {
                // im slice requires mut borrow
                // let stop_ids = data
                //     .stop_times
                //     .slice(stop_times_range.0..stop_times_range.1)
                let stop_ids = data.gtfs.stop_times[stop_times_range.0..stop_times_range.1]
                    .iter()
                    .map(|stop_time| stop_time.stop_id.clone())
                    .collect::<Vec<_>>();
                for (point, stop) in self.stop_circles_canvas.iter().zip(data.stops.iter()) {
                    if stop_ids.contains(&stop.id) {
                        ctx.fill(Circle::new(point.clone(), 2.), &Color::BLACK);
                        ctx.fill(Circle::new(point.clone(), 1.), &Color::WHITE);
                    }
                }
            }
        }
        // });
        ctx.restore();
    }
    fn draw_minimap(&self, ctx: &mut PaintCtx, rect: Rect, zoom: f64) {
        ctx.with_save(|ctx: &mut PaintCtx| {
            ctx.transform(Affine::scale(0.3));
            ctx.fill(rect, &Color::WHITE);
            ctx.draw_image(
                &self.minimap_image.as_ref().unwrap(),
                rect,
                InterpolationMode::Bilinear,
            );

            // paint minimap viewfinder
            ctx.clip(rect);
            ctx.transform(Affine::scale(1. / zoom));
            ctx.transform(Affine::translate(self.focal_point.to_vec2()));
            ctx.stroke(rect, &Color::GRAY, 4. * zoom);
        });
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
                self.click_down_pos = Some(mouse_event.pos);
                self.drag_last_pos = Some(mouse_event.pos);
            }
            Event::MouseMove(mouse_event) => {
                if let Some(drag_start) = self.drag_last_pos {
                    println!("mouse move: drag");
                    if mouse_event.buttons.has_left() {
                        let drag_end = mouse_event.pos;
                        self.focal_point = Point::new(
                            // self.focal_point.x - (drag_end.x - drag_start.x) / self.zoom_level,
                            // self.focal_point.y - (drag_end.y - drag_start.y) / self.zoom_level,
                            self.focal_point.x - (drag_end.x - drag_start.x),
                            self.focal_point.y - (drag_end.y - drag_start.y),
                        );
                        self.drag_last_pos = Some(drag_end);
                    } else {
                        // we keep drag_start.is_some() even if the mouse has left the viewport, otherwise it is annoying if you slightly move your mouse outside the viewport and you loose your drag and have to click again
                        self.drag_last_pos = None;
                        ctx.clear_cursor();
                    }
                    ctx.request_paint();
                } else {
                    // println!("mouse move: check for highlight");
                    self.mouse_position = Some(mouse_event.pos);

                    // find and save all hovered paths
                    let mut highlighted_trip_paths = Vec::new();
                    let path_width = data.map_zoom_level.path_width(BITMAP_SIZE);

                    // ctx.transform(Affine::translate(self.focal_point.to_vec2() * -1.));
                    // ctx.transform(Affine::scale(zoom));
                    let translated_mouse_position = ((mouse_event.pos.to_vec2()
                        + self.focal_point.to_vec2())
                        / data.map_zoom_level.to_f64())
                    .to_point();

                    let path_width2 = path_width * path_width;
                    for (i, box_group) in self.all_trip_paths_canvas_grouped.iter().enumerate() {
                        let (rect, paths) = box_group;
                        if rect.contains(translated_mouse_position) {
                            // println!("in box: {}", i);
                            for (id, color, text_color, path) in paths {
                                for seg in path.segments() {
                                    // NOTE accuracy arg in .nearest() isn't used for lines
                                    // if seg.nearest(mouse_event.pos, 1.).distance_sq < 1. {
                                    if seg.nearest(translated_mouse_position, 1.).distance_sq
                                        < path_width2
                                    {
                                        // dbg!(id);
                                        highlighted_trip_paths.push((
                                            id.clone(),
                                            color.clone(),
                                            text_color.clone(),
                                            path.clone(),
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // for (i, trip_path) in self.all_trip_paths_canvas.iter().enumerate() {
                    //     for seg in trip_path.segments() {
                    //         // NOTE accuracy arg in .nearest() isn't used for lines
                    //         // if seg.nearest(mouse_event.pos, 1.).distance_sq < 1. {
                    //         if seg.nearest(translated_mouse_position, 1.).distance_sq < path_width2
                    //         {
                    //             dbg!(i);
                    //             highlighted_trip_paths.push(trip_path.clone());
                    //             break;
                    //         }
                    //     }
                    // }
                    if self.hovered_trip_paths != highlighted_trip_paths {
                        println!("mouse move: highlights changed");
                        self.hovered_trip_paths = highlighted_trip_paths;
                        self.redraw_highlights = true;
                    }

                    self.highlighted_stop_circle = None;
                    // for circle in &self.stop_circles {
                    //     if circle.contains(mouse_event.pos) {
                    //         self.redraw_highlights = true;
                    //         self.highlighted_stop_circle = Some(circle.clone());
                    //     }
                    // }
                    if self.redraw_highlights {
                        println!("mouse_move: paint");
                        ctx.request_paint();
                    }
                }
            }
            Event::MouseUp(me) => {
                if let Some(click_down_pos) = self.click_down_pos {
                    if me.pos == click_down_pos {
                        println!("mouse_up: same pos");
                        // TODO differentiate between stop click and path click
                        // TODO looping over every stop kills performance. Need to do something like calculate beforehand which stops are within a tile, find which tile the cursor is in and only loop over those stops. At this point, it might also be worth tiling the bitmaps

                        if let Some((id, color, text_color, path)) = self.hovered_trip_paths.get(0)
                        {
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
                            data.selected_trip_id = Some(id.clone());
                            self.selected_trip_path =
                                Some((id.clone(), color.clone(), text_color.clone(), path.clone()));
                        }
                        // for (stop_circle, stop) in
                        //     self.stop_circles.iter().zip(data.stops.iter_mut())
                        // {
                        //     if stop_circle.contains(me.pos) {
                        //         self.redraw_highlights = true;
                        //         self.selected_stop_circle = Some(*stop_circle);
                        //         ctx.submit_command(SELECT_STOP_LIST.with(stop.id.clone()));
                        //     }
                        // }

                        // for (path, trip) in
                        //     self.all_trip_paths.iter().zip(data.stops.iter_mut())
                        // {
                        //     if stop_circle.contains(me.pos) {
                        //         ctx.submit_command(SHOW_STOP.with(stop.id.clone()));
                        //     }
                        // }

                        self.click_down_pos = None;
                        self.redraw_highlights = true;
                        // NOTE don't need to call ctx.paint() since we are updating data which will trigger a paint
                    } else {
                        // todo understand why .clear_cursor() doesn't work here
                        ctx.override_cursor(&Cursor::Arrow);
                        self.click_down_pos = None;
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
        println!("update");
        // if selected trip changes
        if !data.selected_trip_id.same(&old_data.selected_trip_id) {
            println!("update: selected_trip: paint");
            self.selected_trip_path = self
                .all_trip_paths_canvas
                .iter()
                .find(|path| {
                    if let Some(trip_id) = &data.selected_trip_id {
                        &path.0 == trip_id
                    } else {
                        false
                    }
                })
                .cloned();
            // if trip is deselected and route is selected
            if self.selected_trip_path.is_none() {
                if let Some(route_id) = &data.selected_route_id {
                    let trip_ids = data
                        .trips
                        .iter()
                        .filter(|trip| &trip.route_id == route_id)
                        .map(|trip| trip.id.clone())
                        .collect::<Vec<_>>();
                    self.filtered_trip_paths = self
                        .all_trip_paths_canvas
                        .iter()
                        .filter(|(id, _color, text_color, _path)| trip_ids.contains(id))
                        .cloned()
                        .collect::<Vec<_>>();
                }
            }
            self.redraw_highlights = true;
            ctx.request_paint();
        }
        // if new route is selected
        if !data.selected_route_id.same(&old_data.selected_route_id)
            && data.selected_trip_id.is_none()
        {
            if let Some(route_id) = &data.selected_route_id {
                let trip_ids = data
                    .trips
                    .iter()
                    .filter(|trip| &trip.route_id == route_id)
                    .map(|trip| trip.id.clone())
                    .collect::<Vec<_>>();
                self.filtered_trip_paths = self
                    .all_trip_paths_canvas
                    .iter()
                    .filter(|(id, _color, text_color, _path)| trip_ids.contains(id))
                    .cloned()
                    .collect::<Vec<_>>();
                self.redraw_highlights = true;
                ctx.request_paint();
            }
        }
        // TODO should also check if agency is selected and highlight it's paths, but this will kill performance everytime we select SPTRANS in order to select a different route... also app should start with SPTRANS unselected

        if !data.trips.same(&old_data.trips) {
            println!("update: trips: paint");

            // only want to redraw base when a new trip is added, so leave for now
            // self.redraw_base = true;
            self.redraw_highlights = true;
            ctx.request_paint();
        }
        if !data.map_zoom_level.same(&&old_data.map_zoom_level) {
            println!("update: map_zoom_level: paint");
            // self.zoom_level = match data.map_zoom_level {
            //     ZoomLevel::One => 1.,
            //     ZoomLevel::Two => 2.,
            //     ZoomLevel::Three => 3.,
            // };
            self.redraw_base = true;
            // self.redraw_highlights = true;
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
        println!("layout");
        let size = Size::new(300.0, 300.0);
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

        println!("paint");
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.clip(rect);
        // ctx.fill(rect, &Color::grey(0.6));
        ctx.fill(rect, &Color::WHITE);

        if self.redraw_base {
            println!("paint: redraw");
            self.redraw_base = false;
            self.redraw_highlights = false;

            // find size of path data
            // TODO don't clone coord vecs
            let ranges = min_max_trips_coords(&self.trips_coords);
            let width = ranges.longmax - ranges.longmin;
            let height = ranges.latmax - ranges.latmin;

            // calculate size of maximum properly propotioned box we can paint in

            // let (zoomed_paint_width, zoomed_paint_height) =
            //     (paint_width * zoom, paint_height * zoom);

            let (mut base_paint_width, mut base_paint_height) = if width > height {
                (BITMAP_SIZE as f64, BITMAP_SIZE as f64 * height / width)
            } else {
                (BITMAP_SIZE as f64 * width / height, BITMAP_SIZE as f64)
            };
            let long_lat_to_canvas_closure_base = |coord: &(f64, f64)| {
                MapWidget::long_lat_to_canvas(
                    coord,
                    ranges,
                    width,
                    height,
                    base_paint_width,
                    base_paint_height,
                    // BITMAP_SIZE as f64,
                    // BITMAP_SIZE as f64,
                )
            };
            let (mut paint_width, mut paint_height) = if width > height {
                (size.width, size.height * height / width)
            } else {
                (size.width * width / height, size.height)
            };
            let long_lat_to_canvas_closure_canvas = |coord: &(f64, f64)| {
                MapWidget::long_lat_to_canvas(
                    coord,
                    ranges,
                    width,
                    height,
                    paint_width,
                    paint_height,
                )
            };

            if self.remake_paths {
                self.remake_paths = false;
                // TODO should be making data in update or on_added, not paint
                // make trips paths
                println!("{} paint: redraw base: make paths", Utc::now());
                self.all_trip_paths_bitmap = self
                    .trips_coords
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
                            color,
                            text_color,
                            bez_path_from_coords_iter(
                                coords
                                    .iter()
                                    .map(|coord| long_lat_to_canvas_closure_base(coord)),
                            ),
                        )
                    })
                    .collect::<Vec<_>>();

                self.all_trip_paths_canvas = self
                    .trips_coords
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
                                coords
                                    .iter()
                                    .map(|coord| long_lat_to_canvas_closure_canvas(coord)),
                            ),
                        )
                    })
                    .collect::<Vec<_>>();

                println!("{} paint: redraw base: group paths", Utc::now());
                for m in 0..NUMBER_TILES_WIDTH {
                    for n in 0..NUMBER_TILES_WIDTH {
                        let rect = Rect::from_origin_size(
                            (
                                ctx.size().width * m as f64 / NUMBER_TILES_WIDTH as f64,
                                ctx.size().height * n as f64 / NUMBER_TILES_WIDTH as f64,
                            ),
                            (
                                ctx.size().width / NUMBER_TILES_WIDTH as f64,
                                ctx.size().height / NUMBER_TILES_WIDTH as f64,
                            ),
                        );
                        let mut group_paths = Vec::new();
                        // no intersection test yet: https://xi.zulipchat.com/#narrow/stream/260979-kurbo/topic/B.C3.A9zier-B.C3.A9zier.20intersection
                        for (id, color, text_color, trip_path) in &self.all_trip_paths_canvas {
                            for seg in trip_path.segments() {
                                if rect.contains(seg.as_line().unwrap().p0)
                                    || rect.contains(seg.as_line().unwrap().p1)
                                {
                                    group_paths.push((
                                        id.clone(),
                                        color.clone(),
                                        text_color.clone(),
                                        trip_path.clone(),
                                    ));
                                    break;
                                }
                            }
                        }
                        self.all_trip_paths_canvas_grouped.push((rect, group_paths));
                    }
                }

                // self.all_trip_paths_canvas_translated = self.all_trip_paths_canvas.clone();
                // for trip_path in &mut self.all_trip_paths_canvas_translated {
                //     trip_path.apply_affine(Affine::translate(self.focal_point.to_vec2() * -1.));
                //     trip_path.apply_affine(Affine::scale(data.map_zoom_level.to_f64()));
                // }

                // self.selected_trip_paths = trips_coords
                //     .iter()
                //     .filter(|(selected, _)| *selected)
                //     .map(|(_, trip_coords)| {
                //         let mut path = BezPath::new();
                //         for (i, coord) in trip_coords.iter().enumerate() {
                //             if i == 0 {
                //                 path.move_to(long_lat_to_canvas_closure(coord));
                //             } else {
                //                 path.line_to(long_lat_to_canvas_closure(coord));
                //             }
                //         }
                //         path
                //     })
                //     .collect::<Vec<_>>();

                // don't need to set selected path here - should be done in update
                // self.selected_trip_path = self
                //     .trips_coords
                //     .iter()
                //     .zip(data.trips.iter())
                //     .filter(|(_coords, trip)| trip.selected)
                //     .map(|(coords, _)| {
                //         bez_path_from_coords_iter(
                //             coords
                //                 .iter()
                //                 .map(|coord| long_lat_to_canvas_closure_canvas(coord)),
                //         )
                //     })
                //     .get(0)
                //     .unwrap();

                self.stop_circles_base = data
                    .stops
                    .iter()
                    .map(|stop| {
                        // Circle::new(
                        //     long_lat_to_canvas_closure_base(&(stop.coord.0, stop.coord.1)),
                        //     // if zoom > 6. { 6. } else { zoom },
                        //     1.,
                        // )
                        long_lat_to_canvas_closure_base(&(stop.coord.0, stop.coord.1))
                    })
                    .collect::<Vec<_>>();
                self.stop_circles_canvas = data
                    .stops
                    .iter()
                    .map(|stop| {
                        // Circle::new(
                        //     long_lat_to_canvas_closure_base(&(stop.coord.0, stop.coord.1)),
                        //     // if zoom > 6. { 6. } else { zoom },
                        //     1.,
                        // )
                        long_lat_to_canvas_closure_canvas(&(stop.coord.0, stop.coord.1))
                    })
                    .collect::<Vec<_>>();
            }

            println!("{} paint: redraw base: make image", Utc::now());
            let mut cached_image;
            {
                let mut device = Device::new().unwrap();
                // 0.1 makes the image very small
                // let mut target = device.bitmap_target(1000, 1000, 0.1).unwrap();
                let mut target = device.bitmap_target(BITMAP_SIZE, BITMAP_SIZE, 1.).unwrap();
                let mut piet_context = target.render_context();

                piet_context.save();
                // piet_context.transform(Affine::scale(1000. / ctx.size().height));

                // paint the map
                let path_width = data.map_zoom_level.path_width(BITMAP_SIZE);
                for (color, _text_color, path) in &self.all_trip_paths_bitmap {
                    piet_context.stroke(path, color, path_width);
                }

                for (point, stop) in self.stop_circles_base.iter().zip(data.stops.iter()) {
                    piet_context.fill(Circle::new(*point, 1.), &Color::BLACK);
                    piet_context.fill(Circle::new(*point, 0.5), &Color::WHITE);
                }
                // let mut selected_circle = None;
                // for (circle, stop) in self.stop_circles.iter().zip(data.stops.iter()) {
                //     if stop.selected {
                //         selected_circle = Some(Circle::new(
                //             circle.center,
                //             if circle.radius < 2. {
                //                 2. * 1.4
                //             } else {
                //                 circle.radius * 1.4
                //             },
                //         ));
                //     } else {
                //         piet_context.fill(circle, &Color::BLUE);
                //     }
                // }
                // if let Some(selected_circle) = selected_circle {
                //     piet_context.fill(selected_circle, &Color::FUCHSIA);
                // }

                // if let Some(circle) = self.highlighted_stop_circle {
                //     let circle =
                //         Circle::new(circle.center, if zoom > 6. { 6. * 1.4 } else { zoom * 1.4 });
                //     piet_context.fill(circle, &Color::PURPLE);
                // }

                piet_context.restore();

                piet_context.finish().unwrap();
                let image_buf = target.to_image_buf(ImageFormat::RgbaPremul).unwrap();
                cached_image = ctx
                    .make_image(
                        BITMAP_SIZE,
                        BITMAP_SIZE,
                        image_buf.raw_pixels(),
                        ImageFormat::RgbaPremul,
                    )
                    .unwrap();
            }
            if self.minimap_image.is_none() {
                self.minimap_image = Some(cached_image.clone());
            }
            self.cached_image = Some(cached_image);
            println!("{} paint: redraw base: draw map", Utc::now());
            self.draw_base_from_cache(ctx, rect, data.map_zoom_level.to_f64());
            self.draw_minimap(ctx, rect, data.map_zoom_level.to_f64());
        } else if self.redraw_highlights {
            println!("paint: redraw highlights");
            self.draw_base_from_cache(ctx, rect, data.map_zoom_level.to_f64());

            self.draw_highlights(data, ctx, rect, data.map_zoom_level.to_f64());

            self.draw_minimap(ctx, rect, data.map_zoom_level.to_f64());
            // if let Some(selected_circle) = self.selected_stop_circle {
            //     ctx.fill(selected_circle, &Color::FUCHSIA);
            // }

            // if let Some(circle) = self.highlighted_stop_circle {
            //     let circle = Circle::new(
            //         circle.center,
            //         if data.map_zoom_level.to_f64() > 6. {
            //             6. * 1.4
            //         } else {
            //             data.map_zoom_level.to_f64() * 1.4
            //         },
            //     );
            //     ctx.fill(circle, &Color::PURPLE);
            // }
            self.redraw_highlights = false;
        } else {
            println!("paint: use cache");
            self.draw_base_from_cache(ctx, rect, data.map_zoom_level.to_f64());
            // TODO temporarily drawing highlights here too until we add another cache for non hover highlights
            self.draw_highlights(data, ctx, rect, data.map_zoom_level.to_f64());
            self.draw_minimap(ctx, rect, data.map_zoom_level.to_f64());
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
    }
}
