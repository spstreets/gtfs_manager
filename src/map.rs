use druid::im::Vector;
use druid::kurbo::BezPath;
use druid::piet::{
    CairoImage, FontFamily, Image, ImageFormat, InterpolationMode, PietImage, Text,
    TextLayoutBuilder,
};
use druid::widget::{prelude::*, CrossAxisAlignment, LabelText, LensWrap};
use druid::widget::{Align, Button, Checkbox, Controller, Flex, Label, List, TextBox};
use druid::{
    Affine, AppDelegate, AppLauncher, BoxConstraints, Color, Cursor, Data, Env, Event,
    FontDescriptor, Handled, ImageBuf, LayoutCtx, Lens, LensExt, LocalizedString, MouseButtons,
    MouseEvent, Point, Rect, RenderContext, Selector, Size, TextLayout, Widget, WidgetExt,
    WindowDesc,
};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::data::*;

pub struct MapWidget {
    zoom_level: f64,
    speed: f64,
    drag_start: Option<Point>,
    focal_point: Point,
    // cache: Option< RenderContext::Image>,
    cache: Option<CairoImage>,
}
impl MapWidget {
    pub fn new(zoom_level: f64, speed: f64, offset: Point) -> MapWidget {
        println!("new widget");
        MapWidget {
            zoom_level,
            speed,
            drag_start: None,
            focal_point: offset,
            cache: None,
        }
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
                println!("scrolling");
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
                }
            }
            Event::MouseUp(_) => {
                if self.drag_start.is_some() {
                    // todo understand why .clear_cursor() doesn't work here
                    ctx.override_cursor(&Cursor::Arrow);
                    self.drag_start = None;
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
        println!("update");

        // data.agencies
        //     .iter()
        //     .zip(old_data.agencies.iter())
        //     .for_each(|(agency, old_agency)| {
        //         if !agency.selected.same(&old_agency.selected) {
        //             ctx.request_paint();
        //         }
        //     });
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
        println!("layout");
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
        // ctx.
        // Clear the whole widget with the color of your choice
        // (ctx.size() returns the size of the layout rect we're painting in)
        // Note: ctx also has a `clear` method, but that clears the whole context,
        // and we only want to clear this widget's area.
        // RenderContext

        // todo encode gtfs coords and painting coords into two distinct types for clarity
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.clip(rect);
        // ctx.
        // ctx.clear((), Color::grey(0.1));
        // ctx.fill(rect, &Color::grey(0.1));
        ctx.fill(rect, &Color::WHITE);
        match &self.cache {
            Some(image) => {
                let start = Instant::now();
                dbg!(image.size());
                ctx.draw_image(image, rect, InterpolationMode::Bilinear);
                let duration = start.elapsed();
                println!("draw cached image: {:?}", duration);
            }
            None => {
                println!("map paint");
                let start = Instant::now();

                let trips = data.trip_coords();

                // find size of path data
                let x = trips
                    .iter()
                    .map(|trip| trip.iter().map(|point| point.0))
                    .flatten()
                    .collect::<Vec<_>>();
                let y = trips
                    .iter()
                    .map(|trip| trip.iter().map(|point| point.1))
                    .flatten()
                    .collect::<Vec<_>>();
                let longmin = x.iter().cloned().fold(0. / 0., f64::min);
                let latmin = y.iter().cloned().fold(0. / 0., f64::min);
                let x = trips
                    .iter()
                    .map(|trip| trip.iter().map(|point| point.0))
                    .flatten()
                    .collect::<Vec<_>>();
                let y = trips
                    .iter()
                    .map(|trip| trip.iter().map(|point| point.1))
                    .flatten()
                    .collect::<Vec<_>>();
                let longmax = x.iter().cloned().fold(0. / 0., f64::max);
                let latmax = y.iter().cloned().fold(0. / 0., f64::max);
                let width = longmax - longmin;
                let height = latmax - latmin;

                // calculate size of maximum properly propotioned box we can paint in
                let (mut paint_width, mut paint_height) = if width > height {
                    (size.width, size.height * height / width)
                } else {
                    (size.width * width / height, size.height)
                };

                let zoom = self.zoom_level;
                let (zoomed_paint_width, zoomed_paint_height) =
                    (paint_width * zoom, paint_height * zoom);

                let (x_padding, y_padding) = if width > height {
                    (
                        (size.width - zoomed_paint_width) / 2.,
                        (size.height - zoomed_paint_height) / 2.
                            + (zoomed_paint_width - zoomed_paint_height) / 2.,
                    )
                } else {
                    (
                        (size.width - zoomed_paint_width) / 2.
                            + (paint_height - paint_width) / 2. / zoom,
                        (size.height - zoomed_paint_height) / 2.,
                    )
                };

                let long_lat_to_canvas = |point: &(f64, f64)| {
                    let (long, lat) = point;
                    let x2 = (*long - longmin) * (zoomed_paint_width / width) + x_padding
                        - self.focal_point.x * self.zoom_level;
                    let y2 = (*lat - latmin) * (zoomed_paint_height / height) + y_padding
                        - self.focal_point.y * self.zoom_level;
                    Point::new(x2, y2)
                };

                let duration = start.elapsed();
                println!("setup: {:?}", duration);
                let start = Instant::now();

                // performance is better drawing individual trips, rather than making one long path with lot's of .move_to()'s
                let mut paths = Vec::new();
                for trip in trips {
                    let mut path = BezPath::new();
                    for (i, coord) in trip.iter().enumerate() {
                        if i == 0 {
                            path.move_to(long_lat_to_canvas(coord));
                        } else {
                            path.line_to(long_lat_to_canvas(coord));
                        }
                    }
                    paths.push(path);
                }

                let duration = start.elapsed();
                println!("make paths: {:?}", duration);
                let start = Instant::now();

                let stroke_color = Color::GREEN;
                // let ctx2 = ctx.render_ctx.clone();
                for path in paths {
                    ctx.stroke(path, &stroke_color, 1.0);
                }

                let duration = start.elapsed();
                println!("paint paths: {:?}", duration);

                // PietImage
                // ImageBuf::from_data(raw_image)
                // capture_image_area is currently unimplemented for both piet Cairo and web
                dbg!(rect.size());
                let img = ctx.capture_image_area(rect).unwrap();
                dbg!(img.size());
                // ctx.draw_image(&img, rect, InterpolationMode::Bilinear);
                self.cache = Some(img.clone());
            }
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
