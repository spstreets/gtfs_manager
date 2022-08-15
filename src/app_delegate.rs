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
use crate::map::MapWidget;
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
pub const EDIT_STOP_TIME_UPDATE: Selector<String> = Selector::new("edit.stop_time.update");
pub const EDIT_STOP_TIME_CHOOSE: Selector = Selector::new("edit.stop_time.choose");
pub const SELECT_STOP_LIST: Selector<String> = Selector::new("select.stop.list");
pub const SELECT_STOP_MAP: Selector<String> = Selector::new("select.stop.map");
pub const SELECT_AGENCY: Selector<Option<String>> = Selector::new("select.agency");
pub const SELECT_ROUTE: Selector<String> = Selector::new("select.route");
pub const SELECT_TRIP: Selector<String> = Selector::new("select.trip");
pub const SELECT_STOP_TIME: Selector<(String, u16)> = Selector::new("select.stop_time");
pub const SELECT_NOTHING: Selector = Selector::new("select.nothing");

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
        dbg!("got cmd");
        if let Some(item_delete) = cmd.get(ITEM_DELETE) {
            dbg!(item_delete);

            // data.edits.clear();
            // for agency in data.agencies.iter_mut() {
            //     for route in agency.routes.iter_mut() {
            //         if item_delete.0 == "route".to_string() && route.id() == item_delete.1 {
            //             route.live = false;
            //             data.actions.push_back(Action {
            //                 id: data.actions.len(),
            //                 edit_type: EditType::Delete,
            //                 item_type: "route".to_string(),
            //                 item_id: route.id(),
            //                 // item_data: Some(Rc::new(route.clone())),
            //             });
            //         } else {
            //             for trip in route.trips.iter_mut() {
            //                 if item_delete.0 == "trip".to_string() && trip.id() == item_delete.1 {
            //                     trip.live = false;
            //                     data.actions.push_back(Action {
            //                         id: data.actions.len(),
            //                         edit_type: EditType::Delete,
            //                         item_type: "trip".to_string(),
            //                         item_id: trip.id(),
            //                         // item_data: Some(Rc::new(trip.clone())),
            //                     });
            //                 } else {
            //                     for stop_time in trip.stops.iter_mut() {
            //                         if item_delete.0 == "stop_time".to_string()
            //                             && stop_time.id() == item_delete.1
            //                         {
            //                             stop_time.live = false;
            //                             data.actions.push_back(Action {
            //                                 id: data.actions.len(),
            //                                 edit_type: EditType::Delete,
            //                                 item_type: "stop_time".to_string(),
            //                                 item_id: stop_time.id(),
            //                                 // item_data: Some(Rc::new(stop_time.clone())),
            //                             });
            //                         }
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }
            // druid::Handled::No
            druid::Handled::Yes
        } else if let Some(item_update) = cmd.get(ITEM_UPDATE) {
            dbg!(item_update);
            // if item_update.0 == "trip".to_string() {
            //     for agency in data.agencies.iter() {
            //         for route in agency.routes.iter() {
            //             for trip in route.trips.iter() {
            //                 if trip.id() == item_update.1 {
            //                     dbg!(&trip.trip_headsign);
            //                     if data.actions.len() > 0
            //                         && trip.id()
            //                             == data
            //                                 .actions
            //                                 .last()
            //                                 .unwrap()
            //                                 .item_data
            //                                 .as_ref()
            //                                 .unwrap()
            //                                 .id()
            //                         && data.actions.last().unwrap().edit_type == EditType::Update
            //                     {
            //                         data.actions
            //                             .get_mut(data.actions.len() - 1)
            //                             .unwrap()
            //                             .item_data = Some(Rc::new(trip.clone()));
            //                     } else {
            //                         let edit = Action {
            //                             id: data.actions.len(),
            //                             edit_type: EditType::Update,
            //                             item_type: "trip".to_string(),
            //                             item_id: trip.id(),
            //                             item_data: Some(Rc::new(trip.clone())),
            //                         };
            //                         data.actions.push_back(edit);
            //                     }

            //                     // edit = Action {
            //                     //     id: data.edits.len(),
            //                     //     edit_type: EditType::Update,
            //                     //     item_type: "trip".to_string(),
            //                     //     item_id: trip.id(),
            //                     //     item_data: Some(Rc::new(trip.clone())),
            //                     // };
            //                     // match data.edits.iter().position(|edit| {
            //                     //     edit.item_id == item_update.1
            //                     //         && edit.edit_type == EditType::Update
            //                     // }) {
            //                     //     Some(index) => {
            //                     //         data.edits.set(index, edit);
            //                     //     }
            //                     //     None => {
            //                     //         data.edits.push_back(edit);
            //                     //     }
            //                     // };
            //                 }
            //                 // for stop_time in trip.stops.iter() {
            //                 //     if !stop_time.live {
            //                 //         data.edits.push_back(Edit {
            //                 //             id: data.edits.len(),
            //                 //             edit_type: EditType::Delete,
            //                 //             item_type: "stop_time".to_string(),
            //                 //             item_id: stop_time.id(),
            //                 //             item_data: Some(Rc::new(stop_time.clone())),
            //                 //         });
            //                 //     }
            //                 // }
            //             }
            //         }
            //     }
            // }

            // for stop_time in data.gtfs.stop_times {
            //     if stop_time.
            // }
            // for agency in data.gtfs.agencies {
            //     if agency.
            // }
            druid::Handled::Yes

            // delete edits
        } else if let Some(item) = cmd.get(ITEM_NEW_CHILD) {
            println!("new child");
            dbg!(item);
            let (item_type, parent_id) = item;

            // data.edits.clear();
            for agency in data.agencies.iter_mut() {
                if item_type == "agency" && &agency.id() == parent_id {
                    agency.new_child();
                    data.actions.push_back(Action {
                        id: data.actions.len(),
                        edit_type: EditType::Create,
                        // todo is the item type route? or should it be a trip?
                        item_type: "route".to_string(),
                        item_id: agency.id(),
                        // item_data: Some(Rc::new(agency.clone())),
                    });
                } else {
                    // for route in agency.routes.iter_mut() {
                    //     if item_type == "route" && &route.id() == parent_id {
                    //         route.new_child();
                    //         data.actions.push_back(Action {
                    //             id: data.actions.len(),
                    //             edit_type: EditType::Create,
                    //             // todo is the item type route? or should it be a trip?
                    //             item_type: "trip".to_string(),
                    //             item_id: route.id(),
                    //             // item_data: Some(Rc::new(route.clone())),
                    //         });
                    //     } else {
                    //         // for trip in route.trips.iter_mut() {
                    //         //     if item_type == "trip" {
                    //         //         trip.live = false;
                    //         //         data.edits.push_back(Edit {
                    //         //             id: data.edits.len(),
                    //         //             edit_type: EditType::Delete,
                    //         //             item_type: "trip".to_string(),
                    //         //             item_id: trip.id(),
                    //         //             item_data: Some(Rc::new(trip.clone())),
                    //         //         });
                    //         //     } else {
                    //         //         for stop_time in trip.stops.iter_mut() {
                    //         //             if item_type == "stop_time" {
                    //         //                 stop_time.live = false;
                    //         //                 data.edits.push_back(Edit {
                    //         //                     id: data.edits.len(),
                    //         //                     edit_type: EditType::Delete,
                    //         //                     item_type: "stop_time".to_string(),
                    //         //                     item_id: stop_time.id(),
                    //         //                     item_data: Some(Rc::new(stop_time.clone())),
                    //         //                 });
                    //         //             }
                    //         //         }
                    //         //     }
                    //         // }
                    //     }
                    // }
                }
            }
            // druid::Handled::No
            druid::Handled::Yes
        } else if let Some(edit_id) = cmd.get(EDIT_DELETE) {
            dbg!(edit_id);
            let edit = data.actions.get(*edit_id).unwrap();
            // if edit.item_type == "stop_time".to_string() {
            //     for agency in data.agencies.iter_mut() {
            //         for route in agency.routes.iter_mut() {
            //             for trip in route.trips.iter_mut() {
            //                 for stop_time in trip.stops.iter_mut() {
            //                     if stop_time.id() == edit.item_id {
            //                         stop_time.live = true;
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }
            // if edit.item_type == "trip".to_string() {
            //     for agency in data.agencies.iter_mut() {
            //         for route in agency.routes.iter_mut() {
            //             for trip in route.trips.iter_mut() {
            //                 if trip.id() == edit.item_id {
            //                     trip.live = true;
            //                 }
            //             }
            //         }
            //     }
            // }
            data.actions.retain(|edit| edit.id != *edit_id);
            druid::Handled::Yes
        } else if let Some(route_id) = cmd.get(NEW_TRIP) {
            let new_trip = MyTrip::new(route_id.clone());
            data.trips.push_front(new_trip);

            druid::Handled::Yes
        } else if let Some(stop_id) = cmd.get(SELECT_STOP_MAP) {
            for stop in data.stops.iter_mut() {
                if &stop.id == stop_id {
                    // stop.scroll_to_me += 1;
                    stop.selected = true;
                } else {
                    stop.selected = false;
                }
            }

            // TODO when selecting a stop on the map, we want to deselect all other items, unless the stop is on an already selected trip then we actually want to select the stop_time (in this case the map itself should be submitting SELECT_STOP_TIME). If selecting a stop from a stop_time on the list we don't want to deselect the stop_time and it's ancesstors.
            data.selected_agency_id = None;
            data.selected_route_id = None;
            data.selected_trip_id = None;
            data.selected_stop_time_id = None;
            druid::Handled::Yes
        } else if let Some(stop_id) = cmd.get(SELECT_STOP_LIST) {
            dbg!("select_stop_list");
            data.selected_stop_id = Some(stop_id.clone());
            for stop in data.stops.iter_mut() {
                if &stop.id == stop_id {
                    // stop.scroll_to_me += 1;
                    stop.selected = true;
                } else {
                    stop.selected = false;
                }
            }

            // data.selected_agency_id = None;
            // data.selected_route_id = None;
            // data.selected_trip_id = None;
            // data.selected_stop_time_id = None;
            druid::Handled::Yes
        } else if let Some(agency_id) = cmd.get(SELECT_AGENCY) {
            // TODO why is the if statement needed?
            if data.selected_agency_id != Some(agency_id.clone()) {
                dbg!("update agency");
                data.selected_agency_id = Some(agency_id.clone());
            }

            // TODO below is unwantedly clearing child selections even when clicking the current selection which the above if statement's purpose is to avoid
            // clear child selections when a new selection is made
            data.selected_route_id = None;
            data.selected_trip_id = None;
            data.selected_stop_time_id = None;
            data.selected_stop_id = None;

            for agency in data.agencies.iter_mut() {
                if &agency.id == agency_id {
                    agency.selected = true;
                } else {
                    agency.selected = false;
                }
            }
            // data.routes = data
            //     .all_routes
            //     .iter()
            //     .filter(|route| &route.agency_id == agency_id)
            //     // .take(20)
            //     .cloned()
            //     .collect::<Vector<_>>();
            druid::Handled::Yes
        } else if let Some(route_id) = cmd.get(SELECT_ROUTE) {
            if data.selected_route_id != Some(route_id.clone()) {
                data.selected_route_id = Some(route_id.clone());
            }
            data.selected_trip_id = None;
            data.selected_stop_time_id = None;
            data.selected_stop_id = None;
            // TODO need to set data.stop_times = Vector::new();

            for route in data.routes.iter_mut() {
                if &route.id == route_id {
                    route.selected = true;
                } else {
                    route.selected = false;
                }
            }
            druid::Handled::Yes
        } else if let Some(trip_id) = cmd.get(SELECT_TRIP) {
            if data.selected_trip_id != Some(trip_id.clone()) {
                data.selected_trip_id = Some(trip_id.clone());
            }
            data.selected_stop_time_id = None;
            data.selected_stop_id = None;

            for trip in data.trips.iter_mut() {
                if &trip.id == trip_id {
                    trip.selected = true;
                } else {
                    trip.selected = false;
                }
            }

            dbg!("filter and assign stop times");
            // data.stop_times = data
            //     .all_stop_times
            //     .iter()
            //     .filter(|stop_time| &stop_time.trip_id == trip_id)
            //     .cloned()
            //     .collect::<Vector<_>>();

            druid::Handled::Yes
        } else if let Some(_) = cmd.get(SELECT_NOTHING) {
            data.selected_agency_id = None;
            data.selected_route_id = None;
            data.selected_trip_id = None;
            data.selected_stop_time_id = None;
            data.selected_stop_id = None;

            for trip in data.trips.iter_mut() {
                trip.selected = false;
            }
            druid::Handled::Yes
        } else if let Some(_) = cmd.get(EDIT_STOP_TIME_CHOOSE) {
            // let (trip_id, stop_sequence) = stop_time_pk;
            data.map_stop_selection_mode = true;

            druid::Handled::Yes
        } else if let Some(stop_id) = cmd.get(EDIT_STOP_TIME_UPDATE) {
            // let (trip_id, stop_sequence) = stop_time_pk;
            // set the new stop id
            println!(
                "update stop_time {:?} to stop_id: {}",
                data.selected_stop_time_id, stop_id
            );

            druid::Handled::Yes
        } else if let Some(stop_time_pk) = cmd.get(SELECT_STOP_TIME) {
            println!("select stop_time");
            data.selected_stop_time_id = Some(stop_time_pk.clone());
            // These below will already be set when navigating the list, but won't necessarily be when selecting the map?? No...
            // data.selected_route_id
            dbg!(&data.selected_stop_time_id);
            dbg!(&data.selected_trip_id);
            dbg!(&data.selected_route_id);
            dbg!(&data.selected_agency_id);

            data.selected_stop_id = None;

            let (trip_id, stop_sequence) = stop_time_pk;
            for stop_time in data.stop_times.iter_mut() {
                if &stop_time.trip_id == trip_id && &stop_time.stop_sequence == stop_sequence {
                    stop_time.selected = true;
                } else {
                    stop_time.selected = false;
                }
            }
            druid::Handled::Yes
        } else {
            druid::Handled::No
        }
    }
}
