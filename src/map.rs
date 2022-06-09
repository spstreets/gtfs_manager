use druid::im::Vector;
use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::{prelude::*, CrossAxisAlignment, LabelText, LensWrap};
use druid::widget::{Align, Button, Checkbox, Controller, Flex, Label, List, TextBox};
use druid::{
    Affine, AppDelegate, AppLauncher, BoxConstraints, Color, Data, Env, Event, FontDescriptor,
    Handled, LayoutCtx, Lens, LensExt, LocalizedString, Point, Rect, RenderContext, Selector, Size,
    TextLayout, Widget, WidgetExt, WindowDesc,
};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use crate::data::*;

pub struct MapWidget;
// impl MapWidget {
//     fn new(path: Vec<(i64, i64)>) -> MapWidget {
//         MapWidget { hidden: 1, path }
//     }
// }
impl Widget<AppData> for MapWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut AppData, env: &Env) {}
    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppData,
        data: &AppData,
        env: &Env,
    ) {
        if !old_data.same(data) {
            ctx.request_layout();
            ctx.request_paint();
        }
    }
    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.max()
        } else {
            let size = Size::new(300.0, 300.0);
            bc.constrain(size)
        }
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        // Clear the whole widget with the color of your choice
        // (ctx.size() returns the size of the layout rect we're painting in)
        // Note: ctx also has a `clear` method, but that clears the whole context,
        // and we only want to clear this widget's area.
        // RenderContext
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &Color::WHITE);

        let trips = data.agencies[0]
            .routes
            .iter()
            .filter_map(|route| {
                if route.trips.len() > 0 {
                    Some(
                        route.trips[0]
                            .stops
                            .iter()
                            .map(|stop| stop.coord)
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

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
        let xmin = x.iter().cloned().fold(0. / 0., f64::min);
        let ymin = y.iter().cloned().fold(0. / 0., f64::min);
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
        let xmax = x.iter().cloned().fold(0. / 0., f64::max);
        let ymax = y.iter().cloned().fold(0. / 0., f64::max);
        let width = xmax - xmin;
        let height = ymax - ymin;

        let mypoint_to_coord = |point: &(f64, f64)| {
            let (x, y) = point;
            let x2 = (*x - xmin) * (size.width / width);
            let y2 = (*y - ymin) * (size.height / height);
            Point::new(x2, y2)
        };
        for trip in trips {
            let mut path = BezPath::new();
            for (i, coord) in trip.iter().enumerate() {
                if i == 0 {
                    path.move_to(mypoint_to_coord(coord));
                } else {
                    path.line_to(mypoint_to_coord(coord));
                }
            }
            let stroke_color = Color::GREEN;
            ctx.stroke(path, &stroke_color, 1.0);
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
