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
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use crate::app_delegate::*;
use crate::data::*;

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
    all_trip_paths: Vec<BezPath>,
    highlighted_trip_paths: Vec<BezPath>,
    selected_trip_paths: Vec<BezPath>,
    stop_circles: Vec<Circle>,
    highlighted_stop_circle: Option<Circle>,
    selected_stop_circle: Option<Circle>,
    zoom_level: f64,
    speed: f64,
    drag_start: Option<Point>,
    focal_point: Point,
    limit: Option<usize>,
    cached_image: Option<CairoImage>,
    redraw_base: bool,
    redraw_highlights: bool,
}
impl MapWidget {
    pub fn new(zoom_level: f64, speed: f64, offset: Point) -> MapWidget {
        println!("new widget");
        MapWidget {
            mouse_position: None,
            all_trip_paths: Vec::new(),
            highlighted_trip_paths: Vec::new(),
            selected_trip_paths: Vec::new(),
            stop_circles: Vec::new(),
            highlighted_stop_circle: None,
            selected_stop_circle: None,
            zoom_level,
            speed,
            drag_start: None,
            focal_point: offset,
            // limit: Some(50),
            limit: None,
            cached_image: None,
            redraw_base: true,
            redraw_highlights: true,
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

    fn long_lat_to_canvas(
        point: &(f64, f64),
        ranges: PathsRanges,
        width: f64,
        height: f64,
        zoomed_paint_width: f64,
        zoomed_paint_height: f64,
        x_padding: f64,
        y_padding: f64,
        focal_point: Point,
        zoom: f64,
    ) -> Point {
        let (long, lat) = point;
        let x2 = (*long - ranges.longmin) * (zoomed_paint_width / width) + x_padding
            - focal_point.x * zoom;
        let y2 = (*lat - ranges.latmin) * (zoomed_paint_height / height) + y_padding
            - focal_point.y * zoom;
        Point::new(x2, y2)
    }

    fn draw_cache(&self, ctx: &mut PaintCtx, rect: Rect) {
        ctx.transform(Affine::translate(self.focal_point.to_vec2() * -1.));
        ctx.transform(Affine::scale(self.zoom_level));
        ctx.draw_image(
            &self.cached_image.as_ref().unwrap(),
            rect,
            InterpolationMode::Bilinear,
        );
    }
}
impl Widget<AppData> for MapWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        match event {
            Event::Wheel(mouse_event) => {
                let mut change = mouse_event.wheel_delta.y;
                let multiplier = 2000. / self.speed;
                if change > 0. {
                    // zoom out
                    self.zoom_level *= multiplier / (change + multiplier);
                } else if change < 0. {
                    // zoom in
                    self.zoom_level *= (change.abs() + multiplier) / multiplier;
                }
                ctx.request_paint();
            }
            Event::MouseDown(mouse_event) => {
                ctx.override_cursor(&Cursor::Pointer);
                self.drag_start = Some(mouse_event.pos);
            }
            Event::MouseMove(mouse_event) => {
                if let Some(drag_start) = self.drag_start {
                    if mouse_event.buttons.has_left() {
                        let drag_end = mouse_event.pos;
                        self.focal_point = Point::new(
                            self.focal_point.x - (drag_end.x - drag_start.x) / self.zoom_level,
                            self.focal_point.y - (drag_end.y - drag_start.y) / self.zoom_level,
                        );
                        self.drag_start = Some(drag_end);
                    } else {
                        self.drag_start = None;
                        ctx.clear_cursor();
                    }
                    ctx.request_paint();
                } else {
                    self.mouse_position = Some(mouse_event.pos);
                    let mut highlighted_trip_paths = Vec::new();
                    for trip_path in &self.all_trip_paths {
                        for seg in trip_path.segments() {
                            // NOTE accuracy arg in .nearest() isn't used for lines
                            if seg.nearest(mouse_event.pos, 1.).distance_sq < 1. {
                                self.redraw_highlights = true;
                                highlighted_trip_paths.push(trip_path.clone());
                            }
                        }
                    }
                    self.highlighted_trip_paths = highlighted_trip_paths;

                    self.highlighted_stop_circle = None;
                    for circle in &self.stop_circles {
                        if circle.contains(mouse_event.pos) {
                            self.redraw_highlights = true;
                            self.highlighted_stop_circle = Some(circle.clone());
                        }
                    }
                    ctx.request_paint();
                }
            }
            Event::MouseUp(me) => {
                if let Some(drag_start) = self.drag_start {
                    if me.pos == drag_start {
                        // TODO differentiate between stop click and path click
                        // TODO looping over every stop kills performance. Need to do something like calculate beforehand which stops are within a tile, find which tile the cursor is in and only loop over those stops. At this point, it might also be worth tiling the bitmaps
                        for (stop_circle, stop) in
                            self.stop_circles.iter().zip(data.stops.iter_mut())
                        {
                            if stop_circle.contains(me.pos) {
                                self.redraw_highlights = true;
                                self.selected_stop_circle = Some(*stop_circle);
                                ctx.submit_command(SELECT_STOP_LIST.with(stop.id.clone()));
                            }
                        }
                        // for (path, trip) in
                        //     self.all_trip_paths.iter().zip(data.stops.iter_mut())
                        // {
                        //     if stop_circle.contains(me.pos) {
                        //         ctx.submit_command(SHOW_STOP.with(stop.id.clone()));
                        //     }
                        // }
                    } else {
                        // todo understand why .clear_cursor() doesn't work here
                        ctx.override_cursor(&Cursor::Arrow);
                        self.drag_start = None;
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
        // 'outer: for (agency, old_agency) in data.agencies.iter().zip(old_data.agencies.iter()) {
        //     if !agency.visible.same(&old_agency.visible) {
        //         ctx.request_paint();
        //         break 'outer;
        //     } else {
        //         for (route, old_route) in agency.routes.iter().zip(old_agency.routes.iter()) {
        //             if !route.visible.same(&old_route.visible) {
        //                 ctx.request_paint();
        //                 break 'outer;
        //             } else {
        //                 for (trip, old_trip) in route.trips.iter().zip(old_route.trips.iter()) {
        //                     if !trip.visible.same(&old_trip.visible) {
        //                         ctx.request_paint();
        //                         break 'outer;
        //                     } else {
        //                         for (stop, old_stop) in trip.stops.iter().zip(old_trip.stops.iter())
        //                         {
        //                             if !stop.selected.same(&old_stop.selected) {
        //                                 ctx.request_paint();
        //                                 break 'outer;
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        // TODO is this ok or need to loop through and compare items?
        if !data.trips.same(&old_data.trips) {
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

        let size = ctx.size();
        let rect = size.to_rect();
        ctx.clip(rect);
        // ctx.fill(rect, &Color::grey(0.1));

        if self.redraw_base {
            dbg!("redraw");
            self.redraw_base = false;
            self.redraw_highlights = false;

            let mut trips_coords = data.trips_coords();

            if let Some(limit) = self.limit {
                trips_coords = trips_coords.iter().cloned().take(limit).collect::<Vec<_>>();
            }

            // find size of path data
            // TODO don't clone coord vecs
            let ranges = min_max_trips_coords(
                &trips_coords
                    .iter()
                    .map(|trip| trip.1.clone())
                    .collect::<Vec<_>>(),
            );
            let width = ranges.longmax - ranges.longmin;
            let height = ranges.latmax - ranges.latmin;

            // calculate size of maximum properly propotioned box we can paint in
            let (mut paint_width, mut paint_height) = if width > height {
                (size.width, size.height * height / width)
            } else {
                (size.width * width / height, size.height)
            };

            let zoom = self.zoom_level;
            let (zoomed_paint_width, zoomed_paint_height) =
                (paint_width * zoom, paint_height * zoom);

            let (x_padding, y_padding) = MapWidget::calculate_padding(
                width,
                height,
                size,
                zoom,
                zoomed_paint_width,
                zoomed_paint_height,
                paint_width,
                paint_height,
            );

            // make trips paths
            self.all_trip_paths = trips_coords
                .iter()
                .filter(|(selected, _)| !selected)
                .map(|(_, trip_coords)| {
                    let mut path = BezPath::new();
                    for (i, coord) in trip_coords.iter().enumerate() {
                        if i == 0 {
                            path.move_to(MapWidget::long_lat_to_canvas(
                                coord,
                                ranges,
                                width,
                                height,
                                zoomed_paint_width,
                                zoomed_paint_height,
                                x_padding,
                                y_padding,
                                self.focal_point,
                                zoom,
                            ));
                        } else {
                            path.line_to(MapWidget::long_lat_to_canvas(
                                coord,
                                ranges,
                                width,
                                height,
                                zoomed_paint_width,
                                zoomed_paint_height,
                                x_padding,
                                y_padding,
                                self.focal_point,
                                zoom,
                            ));
                        }
                    }
                    path
                })
                .collect::<Vec<_>>();

            self.selected_trip_paths = trips_coords
                .iter()
                .filter(|(selected, _)| *selected)
                .map(|(_, trip_coords)| {
                    let mut path = BezPath::new();
                    for (i, coord) in trip_coords.iter().enumerate() {
                        if i == 0 {
                            path.move_to(MapWidget::long_lat_to_canvas(
                                coord,
                                ranges,
                                width,
                                height,
                                zoomed_paint_width,
                                zoomed_paint_height,
                                x_padding,
                                y_padding,
                                self.focal_point,
                                zoom,
                            ));
                        } else {
                            path.line_to(MapWidget::long_lat_to_canvas(
                                coord,
                                ranges,
                                width,
                                height,
                                zoomed_paint_width,
                                zoomed_paint_height,
                                x_padding,
                                y_padding,
                                self.focal_point,
                                zoom,
                            ));
                        }
                    }
                    path
                })
                .collect::<Vec<_>>();

            self.stop_circles = data
                .stops
                .iter()
                .map(|stop| {
                    Circle::new(
                        MapWidget::long_lat_to_canvas(
                            &(stop.coord.0, stop.coord.1),
                            ranges,
                            width,
                            height,
                            zoomed_paint_width,
                            zoomed_paint_height,
                            x_padding,
                            y_padding,
                            self.focal_point,
                            zoom,
                        ),
                        if zoom > 6. { 6. } else { zoom },
                    )
                })
                .collect::<Vec<_>>();

            let mut cached_image;
            {
                let mut device = Device::new().unwrap();
                let mut target = device.bitmap_target(1000, 1000, 1.).unwrap();
                let mut piet_context = target.render_context();

                piet_context.save();
                piet_context.transform(Affine::scale(1000. / ctx.size().height));

                // paint the map
                for path in &self.all_trip_paths {
                    piet_context.stroke(path, &Color::GREEN, 1.0);
                }
                for path in &self.highlighted_trip_paths {
                    piet_context.stroke(path, &Color::NAVY, 3.);
                }
                for path in &self.selected_trip_paths {
                    piet_context.stroke(path, &Color::YELLOW, 3.);
                }

                // if let Some(mouse_position) = self.mouse_position {
                //     let circle = Circle::new(mouse_position, 10.);
                //     ctx.fill(circle, &Color::OLIVE);
                // }

                let mut selected_circle = None;
                for (circle, stop) in self.stop_circles.iter().zip(data.stops.iter()) {
                    if stop.selected {
                        selected_circle = Some(Circle::new(
                            circle.center,
                            if circle.radius < 2. {
                                2. * 1.4
                            } else {
                                circle.radius * 1.4
                            },
                        ));
                    } else {
                        piet_context.fill(circle, &Color::BLUE);
                    }
                }
                if let Some(selected_circle) = selected_circle {
                    piet_context.fill(selected_circle, &Color::FUCHSIA);
                }

                if let Some(circle) = self.highlighted_stop_circle {
                    let circle =
                        Circle::new(circle.center, if zoom > 6. { 6. * 1.4 } else { zoom * 1.4 });
                    piet_context.fill(circle, &Color::PURPLE);
                }

                piet_context.restore();
                // piet_context.with_save(|ctx| {
                //     {}
                //     Ok(())
                // });

                piet_context.finish().unwrap();
                let image_buf = target.to_image_buf(ImageFormat::RgbaPremul).unwrap();
                // let cached_image = ctx
                cached_image = ctx
                    .make_image(
                        1000,
                        1000,
                        // image_buf.to_image(piet_context),
                        image_buf.raw_pixels(),
                        ImageFormat::RgbaPremul,
                    )
                    .unwrap();
            }
            // std::mem::drop(piet_context);
            // let cached_image = image_buf.to_image(ctx);
            self.cached_image = Some(cached_image);
            ctx.draw_image(
                &self.cached_image.as_ref().unwrap(),
                rect,
                InterpolationMode::Bilinear,
            );
        } else if self.redraw_highlights {
            dbg!("redraw highlights");
            self.draw_cache(ctx, rect);
            ctx.fill(Circle::new(Point::new(100., 100.), 50.), &Color::FUCHSIA);
            for path in &self.highlighted_trip_paths {
                ctx.stroke(path, &Color::NAVY, 3.);
            }
            for path in &self.selected_trip_paths {
                ctx.stroke(path, &Color::YELLOW, 3.);
            }
            if let Some(selected_circle) = self.selected_stop_circle {
                ctx.fill(selected_circle, &Color::FUCHSIA);
            }

            if let Some(circle) = self.highlighted_stop_circle {
                let circle = Circle::new(
                    circle.center,
                    if self.zoom_level > 6. {
                        6. * 1.4
                    } else {
                        self.zoom_level * 1.4
                    },
                );
                ctx.fill(circle, &Color::PURPLE);
            }
            self.redraw_highlights = false;
        } else {
            dbg!("use cache");
            self.draw_cache(ctx, rect);
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
