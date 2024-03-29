use std::collections::HashMap;

use druid::im::Vector;
use druid::keyboard_types::Key;
use druid::{AppDelegate, Env, Event, Point, Selector};

use crate::data::*;
// use crate::my_trip_derived_lenses::route_id;

// command selectors
// (<item type>, <id>)
pub const ITEM_DELETE: Selector<(String, String)> = Selector::new("item.delete");
// (<item type>, <id>)
pub const ITEM_UPDATE: Selector<(String, String)> = Selector::new("item.update");
// (<item type>, <parent id>)
pub const ITEM_NEW_CHILD: Selector<(String, String)> = Selector::new("item.new.child");
pub const EDIT_DELETE: Selector<usize> = Selector::new("edit.delete");
pub const EDIT_STOP_TIME_CHOOSE: Selector = Selector::new("edit.stop_time.choose");
pub const EDIT_STOP_TIME_UPDATE: Selector<String> = Selector::new("edit.stop_time.update");
pub const NEW_STOP: Selector<Point> = Selector::new("new.stop");

/// Selector(trip_id, stop_sequence, before) so before: true, after: false
pub const ADD_STOP_TIME_CHOOSE: Selector<bool> = Selector::new("add.stop_time.choose");
/// Selector<route id>
pub const ADD_TRIP: Selector<String> = Selector::new("add.trip");
/// Selector<agency id>
pub const ADD_ROUTE: Selector<Option<String>> = Selector::new("add.route");
// pub const ADD_STOP_TIME_UPDATE: Selector<String> = Selector::new("add.stop_time.update");

pub const HOVER_STOP_TIME: Selector<Option<(String, u16)>> = Selector::new("hover.stop_time");
pub const HOVER_TRIP: Selector<Option<String>> = Selector::new("hover.trip");
pub const HOVER_ROUTE: Selector<Option<String>> = Selector::new("hover.route");

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
        _ctx: &mut druid::DelegateCtx,
        _window_id: druid::WindowId,
        event: Event,
        _data: &mut AppData,
        _env: &Env,
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
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppData,
        _env: &Env,
    ) -> druid::Handled {
        myprint!("got cmd");
        if let Some(item_delete) = cmd.get(ITEM_DELETE) {
            dbg!(item_delete);
            druid::Handled::Yes
        } else if let Some(item_update) = cmd.get(ITEM_UPDATE) {
            dbg!(item_update);
            druid::Handled::Yes

            // delete edits
        } else if let Some(item) = cmd.get(ITEM_NEW_CHILD) {
            myprint!("new child");
            let (item_type, parent_id) = item;

            // data.edits.clear();
            for agency in data.agencies.iter_mut() {
                if item_type == "agency" && &agency.id() == parent_id {
                    // agency.new_child();
                    data.actions.push_back(Action {
                        id: data.actions.len(),
                        edit_type: EditType::Create,
                        // todo is the item type route? or should it be a trip?
                        item_type: "route".to_string(),
                        item_id: agency.id(),
                        // item_data: Some(Rc::new(agency.clone())),
                    });
                }
            }
            // druid::Handled::No
            druid::Handled::Yes
        } else if let Some(edit_id) = cmd.get(EDIT_DELETE) {
            dbg!(edit_id);
            let _edit = data.actions.get(*edit_id).unwrap();
            data.actions.retain(|edit| edit.id != *edit_id);
            druid::Handled::Yes
        } else if let Some(agency_id) = cmd.get(ADD_ROUTE) {
            let new_route = MyRoute::new(agency_id.clone());
            data.routes.push_front(new_route);
            druid::Handled::Yes
        } else if let Some(_route_id) = cmd.get(ADD_TRIP) {
            data.map_stop_selection_mode = true;
            // let new_trip = MyTrip::new(route_id.clone());
            // data.trips.push_front(new_trip);
            druid::Handled::Yes
        } else if let Some(latlong) = cmd.get(NEW_STOP) {
            myprint!("handle NEW_STOP command");
            let new_stop = MyStop::new(*latlong);

            // need to resort stops? no, stops are not sorted
            data.stops.push_back(new_stop.clone());

            // let mut stop_index_from_id = HashMap::new();
            // data.stops.iter().enumerate().for_each(|(i, stop)| {
            //     stop_index_from_id.insert(stop.id.clone(), i);
            // });
            data.stop_index_from_id
                .insert(new_stop.id.clone(), data.stops.len() - 1);

            // first we need to determine whether we are adding to an existing trip, or creating a new trip by looking at whether a stop_time is selected or only route, else panic

            // update existing trip
            if let Some((trip_id, stop_sequence)) = &data.selected_stop_time_id {
                // determine whether stop selected is for updating a stop_time or creating a new one

                // create new stop_time
                if let Some(insert_stop_time_before) = data.insert_stop_time_before {
                    // data.stop_times is sorted and it's order is assumed fixed by stop_time_range_from_trip_id
                    // stop_times.sort_by(|stop1, stop2| stop1.stop_sequence.cmp(&stop2.stop_sequence));
                    // stop_times.sort_by(|x1, x2| x1.trip_id.cmp(&x2.trip_id));

                    // need to insert the new stop_time, update all the stop_sequences for the other stop_times in that trip, then resort stop_times (not actually necessary), and recreate stop_time_range_from_trip_id (might be avoidable if we store actual stop_times in a HashMap)

                    // could maybe use data.stop_times.insert_ord(item) ???
                    let (selected_stop_time_index, _) = data
                        .stop_times
                        .iter()
                        .enumerate()
                        .find(|(_index, stop_time)| {
                            &stop_time.trip_id == trip_id
                                && &stop_time.stop_sequence == stop_sequence
                        })
                        .unwrap();

                    // recalcuate n_stops for MyTrip
                    let trip = data
                        .trips
                        .iter_mut()
                        .find(|trip| &trip.id == trip_id)
                        .unwrap();
                    trip.n_stops += 1;

                    // insert new stop_time
                    data.stop_times.insert(
                        if insert_stop_time_before {
                            selected_stop_time_index
                        } else {
                            selected_stop_time_index + 1
                        },
                        MyStopTime::new(trip_id.clone(), new_stop.id.clone(), 99),
                    );
                    // udpate stop_time_range_from_trip_id (important to do this first to get correct range to update stop_sequences)

                    // data.stop_time_range_from_trip_id = stop_time_range_from_trip_id;
                    data.stop_time_range_from_trip_id =
                        make_stop_time_range_from_trip_id(&data.stop_times);

                    // udpate all stop_sequences for that trip
                    let range = data.stop_time_range_from_trip_id.get(trip_id).unwrap();
                    let mut stop_sequence_inc = 1;
                    for i in range.0..range.1 {
                        let stop_time = data.stop_times.get_mut(i).unwrap();
                        stop_time.stop_sequence = stop_sequence_inc;
                        stop_sequence_inc += 1;
                    }

                    data.insert_stop_time_before = None;

                    // update existing stop_time
                } else {
                    let selected_stop_time = data
                        .stop_times
                        .iter_mut()
                        .find(|stop_time| {
                            &stop_time.trip_id == trip_id
                                && &stop_time.stop_sequence == stop_sequence
                        })
                        .unwrap();
                    selected_stop_time.stop_id = new_stop.id.clone();
                    selected_stop_time.edited = true;
                }

            // new trip
            } else if let Some(selected_route_id) = &data.selected_route_id {
                // data.map_stop_selection_mode = true;
                let mut new_trip = MyTrip::new(selected_route_id.clone());
                new_trip.n_stops = 1;
                data.trips.push_front(new_trip.clone());

                // insert new stop_time
                data.stop_times.push_back(MyStopTime::new(
                    new_trip.id.clone(),
                    new_stop.id.clone(),
                    1,
                ));
                // udpate stop_time_range_from_trip_id (important to do this first to get correct range to update stop_sequences)

                data.stop_time_range_from_trip_id =
                    make_stop_time_range_from_trip_id(&data.stop_times);
            } else {
                panic!("shouldn't be able to select a stop here ");
            }

            druid::Handled::Yes
        } else if let Some(_) = cmd.get(EDIT_STOP_TIME_CHOOSE) {
            // let (trip_id, stop_sequence) = stop_time_pk;
            data.map_stop_selection_mode = true;

            druid::Handled::Yes
        } else if let Some(stop_id) = cmd.get(EDIT_STOP_TIME_UPDATE) {
            myprint!("cmd.get(EDIT_STOP_TIME_UPDATE)");
            // first we need to determine whether we are adding to an existing trip, or creating a new trip by looking at whether a stop_time is selected or only route, else panic

            if let Some((trip_id, stop_sequence)) = &data.selected_stop_time_id {
                // determine whether stop selected is for updating a stop_time or creating a new one

                // data.stop_times is sorted and it's order is assumed fixed by stop_time_range_from_trip_id
                // stop_times.sort_by(|stop1, stop2| stop1.stop_sequence.cmp(&stop2.stop_sequence));
                // stop_times.sort_by(|x1, x2| x1.trip_id.cmp(&x2.trip_id));

                // need to insert the new stop_time, update all the stop_sequences for the other stop_times in that trip, then resort stop_times (not actually necessary), and recreate stop_time_range_from_trip_id (might be avoidable if we store actual stop_times in a HashMap)

                // insert new stop_time before or after selected stop_time
                if let Some(insert_stop_time_before) = data.insert_stop_time_before {
                    // could maybe use data.stop_times.insert_ord(item) ???
                    let (selected_stop_time_index, _) = data
                        .stop_times
                        .iter()
                        .enumerate()
                        .find(|(_index, stop_time)| {
                            &stop_time.trip_id == trip_id
                                && &stop_time.stop_sequence == stop_sequence
                        })
                        .unwrap();

                    // recalcuate n_stops for MyTrip
                    let trip = data
                        .trips
                        .iter_mut()
                        .find(|trip| &trip.id == trip_id)
                        .unwrap();
                    trip.n_stops += 1;

                    // insert new stop_time
                    data.stop_times.insert(
                        if insert_stop_time_before {
                            selected_stop_time_index
                        } else {
                            selected_stop_time_index + 1
                        },
                        MyStopTime::new(trip_id.clone(), stop_id.clone(), 99),
                    );
                    // udpate stop_time_range_from_trip_id (important to do this first to get correct range to update stop_sequences)

                    // data.stop_time_range_from_trip_id = stop_time_range_from_trip_id;
                    data.stop_time_range_from_trip_id =
                        make_stop_time_range_from_trip_id(&data.stop_times);

                    // udpate all stop_sequences for that trip
                    let range = data.stop_time_range_from_trip_id.get(trip_id).unwrap();
                    let mut stop_sequence_inc = 1;
                    for i in range.0..range.1 {
                        let stop_time = data.stop_times.get_mut(i).unwrap();
                        stop_time.stop_sequence = stop_sequence_inc;
                        stop_sequence_inc += 1;
                    }
                    data.insert_stop_time_before = None;

                // edit existing stop time
                } else {
                    // set the new stop id
                    println!(
                        "update stop_time {:?} to stop_id: {}",
                        data.selected_stop_time_id, stop_id
                    );
                    let selected_stop_time = data
                        .stop_times
                        .iter_mut()
                        .find(|stop_time| {
                            &stop_time.trip_id == trip_id
                                && &stop_time.stop_sequence == stop_sequence
                        })
                        .unwrap();
                    selected_stop_time.stop_id = stop_id.clone();
                    selected_stop_time.edited = true;
                }
            } else if let Some(selected_route_id) = &data.selected_route_id {
                // data.map_stop_selection_mode = true;
                let mut new_trip = MyTrip::new(selected_route_id.clone());
                new_trip.n_stops = 1;
                data.trips.push_front(new_trip.clone());

                // insert new stop_time
                data.stop_times
                    .push_back(MyStopTime::new(new_trip.id.clone(), stop_id.clone(), 1));
                // udpate stop_time_range_from_trip_id (important to do this first to get correct range to update stop_sequences)

                data.stop_time_range_from_trip_id =
                    make_stop_time_range_from_trip_id(&data.stop_times);
            } else {
                panic!("shouldn't be able to select a stop here ");
            }

            druid::Handled::Yes
        } else if let Some(before) = cmd.get(ADD_STOP_TIME_CHOOSE) {
            data.map_stop_selection_mode = true;
            data.insert_stop_time_before = Some(*before);
            druid::Handled::Yes
        // } else if let Some(stop_id) = cmd.get(ADD_STOP_TIME_UPDATE) {
        //     data.insert_stop_time_before = Some(*before);

        //     druid::Handled::Yes
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
            myprint!("select_stop_list");
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
                myprint!("update agency");
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
            myprint!("select trip");
            let trip_index = data
                .trips
                .iter()
                .enumerate()
                .find(|(_index, trip)| &trip.id == trip_id)
                .unwrap()
                .0;
            data.selected_trip_id = Some((trip_index, trip_id.clone()));
            dbg!(&data.selected_trip_id);
            data.selected_stop_time_id = None;
            data.selected_stop_id = None;

            for trip in data.trips.iter_mut() {
                if &trip.id == trip_id {
                    trip.selected = true;
                } else {
                    trip.selected = false;
                }
            }

            myprint!("filter and assign stop times");
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
            // data.selected_trip_path = None;

            for trip in data.trips.iter_mut() {
                trip.selected = false;
            }
            druid::Handled::Yes
        } else if let Some(stop_time_id) = cmd.get(HOVER_STOP_TIME) {
            // TODO this is all way too heavy and needs simplifying

            // let (trip_id, stop_sequence) = stop_time_pk;
            // set the new stop id
            println!("hover stop_time {:?}", data.selected_stop_time_id);
            let previous_hovered_stop_time_id = data.hovered_stop_time_id.clone();
            data.hovered_stop_time_id = stop_time_id.clone();

            // need to also store hover state on MyStopTime because when dynamically setting border color for widget we are lensed to MyStopTime and don't have access to AppData
            // if we have a hovered stop_time, then set MyStopTime.hovered = true
            if let Some((trip_id, stop_sequence)) = stop_time_id {
                let range = data.stop_time_range_from_trip_id.get(trip_id).unwrap();
                for i in range.0..range.1 {
                    let stop_time = data.stop_times.get_mut(i).unwrap();
                    if &stop_time.stop_sequence == stop_sequence {
                        stop_time.hovered = true;
                    } else {
                        stop_time.hovered = false;
                    }
                }
            }
            // if we have None, then clear MyStopTime.hovered = true for previously selected trip
            if let Some((trip_id, _stop_sequence)) = previous_hovered_stop_time_id {
                let range = data.stop_time_range_from_trip_id.get(&trip_id).unwrap();
                for i in range.0..range.1 {
                    let stop_time = data.stop_times.get_mut(i).unwrap();
                    stop_time.hovered = false;
                }
            }

            druid::Handled::Yes
        } else if let Some(stop_time_pk) = cmd.get(SELECT_STOP_TIME) {
            myprint!("select stop_time");
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

fn make_stop_time_range_from_trip_id(
    stop_times: &Vector<MyStopTime>,
) -> HashMap<String, (usize, usize)> {
    let mut stop_time_range_from_trip_id = HashMap::new();
    let mut trip_start_index = 0;
    let mut trip_end_index = 0;
    let mut current_trip = stop_times[0].trip_id.clone();
    // let stop_times2 = data.gtfs.stop_times.clone();
    for stop_time in stop_times {
        // when we arrive at a new section of trip_id's insert the index range into to map, update the current trip, and reset the range start index
        if current_trip != stop_time.trip_id {
            stop_time_range_from_trip_id
                .insert(current_trip.clone(), (trip_start_index, trip_end_index));
            current_trip = stop_time.trip_id.clone();
            trip_start_index = trip_end_index;
        }
        trip_end_index += 1;
    }
    // insert final trip id
    stop_time_range_from_trip_id.insert(current_trip.clone(), (trip_start_index, trip_end_index));
    stop_time_range_from_trip_id
}
