use std::rc::Rc;

use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::keyboard_types::Key;
use druid::lens::{self, LensExt};
use druid::text::{EditableText, TextStorage};
use druid::widget::{
    Button, Checkbox, Container, Controller, CrossAxisAlignment, Either, Flex, FlexParams, Label,
    LabelText, List, MainAxisAlignment, Painter, RadioGroup, Scroll, Stepper, TextBox,
};
use druid::{
    AppDelegate, AppLauncher, Color, Data, Env, Event, EventCtx, FontDescriptor, FontFamily,
    FontWeight, Insets, Lens, LocalizedString, PaintCtx, Point, RenderContext, Selector, UnitPoint,
    UpdateCtx, Widget, WidgetExt, WindowDesc,
};
use gtfs_structures::ContinuousPickupDropOff;
use rgb::RGB8;

use crate::data::*;
// use crate::map::MapWidget;
// use crate::my_trip_derived_lenses::route_id;

// command selectors
// (<item type>, <id>)
pub const ITEM_DELETE: Selector<(String, String)> = Selector::new("item.delete");
// (<item type>, <id>)
pub const ITEM_UPDATE: Selector<(String, String)> = Selector::new("item.update");
// (<item type>, <parent id>)
pub const ITEM_NEW_CHILD: Selector<(String, String)> = Selector::new("item.new.child");
pub const NEW_TRIP: Selector<String> = Selector::new("new.trip");
pub const EDIT_DELETE: Selector<usize> = Selector::new("edit.delete");
pub const SELECT_STOP: Selector<String> = Selector::new("show.stop");
pub const SELECT_AGENCY: Selector<Option<String>> = Selector::new("select.agency");
pub const SELECT_ROUTE: Selector<String> = Selector::new("select.route");
pub const SELECT_TRIP: Selector<String> = Selector::new("select.trip");
pub const SELECT_STOP_TIME: Selector<(String, u16)> = Selector::new("select.stop_time");

pub struct Delegate;
impl AppDelegate<AppData> for Delegate {
    fn event(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        window_id: druid::WindowId,
        event: Event,
        data: &mut AppData,
        env: &Env,
    ) -> Option<Event> {
        match &event {
            Event::KeyDown(key_event) => {
                // not firing for some reason
                println!("keydown");
                match key_event.key {
                    Key::ArrowUp => {
                        println!("arrowup");
                    }
                    Key::ArrowDown => {
                        println!("arrowdown");
                    }
                    _ => {}
                }
            }
            _ => {}
        };
        Some(event)
    }
    fn command(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppData,
        env: &Env,
    ) -> druid::Handled {
        dbg!("command");
        if let Some(agency_id) = cmd.get(SELECT_AGENCY) {
            for agency in data.agencies.iter_mut() {
                if &agency.id == agency_id {
                    agency.selected = true;
                } else {
                    agency.selected = false;
                }
            }
            druid::Handled::Yes
        } else if let Some(route_id) = cmd.get(SELECT_ROUTE) {
            for route in data.routes.iter_mut() {
                if &route.id == route_id {
                    route.selected = true;
                } else {
                    route.selected = false;
                }
            }
            druid::Handled::Yes
        } else if let Some(trip_id) = cmd.get(SELECT_TRIP) {
            for trip in data.trips.iter_mut() {
                if &trip.id == trip_id {
                    trip.selected = true;
                } else {
                    trip.selected = false;
                }
            }

            druid::Handled::Yes
        } else {
            druid::Handled::No
        }
    }
}
