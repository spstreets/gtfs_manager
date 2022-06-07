// ignore unused warnings while prototyping
#![allow(unused)]
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use clap::Parser;
use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::lens::{self, LensExt};
use druid::widget::{
    Button, Checkbox, CrossAxisAlignment, Either, Flex, FlexParams, Label, List, MainAxisAlignment,
    Scroll,
};
use druid::{
    AppLauncher, Color, Data, Env, Insets, Lens, LocalizedString, UnitPoint, Widget, WidgetExt,
    WindowDesc,
};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use gtfs_manager::ListItem;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Optional path to a GTFS zip. If missing demo data will be loaded
    pub path: Option<String>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyStopTime {
    selected: bool,
    // #[data(ignore)]
    // stop_time: Rc<StopTime>,
    name: String,
}

#[derive(Clone, Data, Default, Lens)]
struct MyTrip {
    selected: bool,
    // #[data(ignore)]
    // trip: Rc<RawTrip>,
    name: String,
    stops: Vector<MyStopTime>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyRoute {
    selected: bool,
    // #[data(ignore)]
    // route: Rc<Route>,
    name: String,
    trips: Vector<MyTrip>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyAgency {
    selected: bool,
    // #[data(ignore)]
    // agency: Rc<Agency>,
    name: String,
    routes: Vector<MyRoute>,
}

#[derive(Clone, Data, Default, Lens)]
struct AppData {
    agencies: Vector<MyAgency>,
}

fn stop_ui() -> impl Widget<MyStopTime> {
    Flex::row()
        .with_child(Checkbox::new("").lens(MyStopTime::selected))
        .with_child(Label::new(|data: &MyStopTime, _env: &_| {
            format!("{}", data.name)
        }))
        // .with_child(Either::new(
        //     |data: &Trip, _env: &Env| data.selected,
        //     List::new(stop_ui).lens(Trip::stops),
        //     Label::new(""),
        // ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}
fn trip_ui() -> impl Widget<MyTrip> {
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyTrip::selected))
        .with_child(Label::new(|data: &MyTrip, _env: &_| {
            format!("{}", data.name)
        }));

    Flex::column()
        .with_child(title)
        .with_child(Either::new(
            |data: &MyTrip, _env: &Env| data.selected,
            List::new(stop_ui).lens(MyTrip::stops),
            Flex::row(),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}
fn route_ui() -> impl Widget<MyRoute> {
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyRoute::selected))
        .with_child(Label::new(|data: &MyRoute, _env: &_| {
            format!("{}", data.name)
        }));

    Flex::column()
        .with_child(title)
        .with_child(Either::new(
            |data: &MyRoute, _env: &Env| data.selected,
            List::new(trip_ui).lens(MyRoute::trips),
            Flex::row(),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}
fn agency_ui() -> impl Widget<MyAgency> {
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyAgency::selected))
        .with_child(Label::new(|data: &MyAgency, _env: &_| {
            format!("{}", data.name)
        }));

    Flex::column()
        .with_child(title)
        .with_child(Either::new(
            |data: &MyAgency, _env: &Env| data.selected,
            List::new(route_ui).lens(MyAgency::routes),
            Flex::row(),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}

fn main_widget() -> impl Widget<AppData> {
    Scroll::new(List::new(agency_ui).lens(AppData::agencies))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    let gtfs = RawGtfs::new(&args.path.expect("must provide a path or url to a GTFS zip"))?;
    gtfs.print_stats();
    let mut agencies = gtfs.agencies.unwrap();
    let mut routes = gtfs.routes.unwrap();
    let mut trips = gtfs.trips.unwrap();
    let mut stop_times = gtfs.stop_times.unwrap();
    let mut stops = gtfs.stops.unwrap();

    // need to be able to grab a slice of stop times by trip id to avoid doing the below loads of times:
    // stop_times
    //     .iter()
    //     .filter(|stop_time| stop_time.trip_id == trip.id);
    trips.sort_by(|x1, x2| x1.id.cmp(&x2.id));
    stop_times.sort_by(|x1, x2| x1.trip_id.cmp(&x2.trip_id));
    let mut stop_time_range_from_trip_id = HashMap::new();
    let mut trip_start_index = 0;
    let mut trip_end_index = 0;
    let mut current_trip = stop_times[0].trip_id.clone();
    let stop_times2 = stop_times.clone();
    for stop_time in stop_times2 {
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

    // hash map for getting a top by stop_id
    let mut stop_map = HashMap::new();
    let stops2 = stops.clone();
    stops2.iter().for_each(|stop| {
        stop_map.insert(stop.id.clone(), stop.clone());
    });
    // let name = gtfs
    //     .stops
    //     .unwrap()
    //     .iter()
    //     .find(|stop| stop.id == stop_time.stop_id)
    //     .unwrap()
    //     .name
    //     .clone();

    // trip_start_index = trip_end_index;
    // for trip in trips_by_parent[trip_start_index..].iter() {
    //     if trip.route_id == route.id {
    //         trip_end_index += 1;
    //     } else {
    //         break;
    //     }
    // }

    // let mut routes_by_parent = routes.clone();
    // let mut trips_by_parent = trips.clone();
    // let mut stop_times_by_parent = stop_times.clone();

    // routes_by_parent.sort_by(|x1, x2| x1.agency_id.cmp(&x2.agency_id));
    // trips_by_parent.sort_by(|x1, x2| x1.route_id.cmp(&x2.route_id));
    // stop_times_by_parent.sort_by(|x1, x2| x1.trip_id.cmp(&x2.trip_id));

    agencies.sort_by(|x1, x2| x1.name.cmp(&x2.name));
    // routes.sort_by(|x1, x2| x1.id.cmp(&x2.id));
    // trips.sort_by(|x1, x2| x1.id.cmp(&x2.id));
    // stops.sort_by(|x1, x2| x1.id.cmp(&x2.id));

    // let mut agencies_iter = agencies.iter();
    // let mut routes_iter = routes.iter();
    // let mut trips_iter = trips.iter();
    // let mut stop_times_iter = stop_times.iter();
    // let mut stops_iter = stops.iter();
    // let gtfs = Gtfs::new(&args.path.expect("must provide a path or url to a GTFS zip"))?;
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((1000., 600.));

    // let mut route_start_index = 0;
    // let mut route_end_index = 0;

    // let mut trip_start_index = 0;
    // let mut trip_end_index = 0;

    // let app_data = AppData {
    //     agencies: agencies
    //         .iter()
    //         .map(|agency| {
    //             // update slice range
    //             route_start_index = route_end_index;
    //             for route in routes_by_parent[route_start_index..].iter() {
    //                 if route.agency_id == agency.id {
    //                     route_end_index += 1;
    //                 } else {
    //                     break;
    //                 }
    //             }

    //             // make Vector from this agency's slice of routes
    //             let routes = Vector::from_iter(
    //                 routes_by_parent[route_start_index..route_end_index]
    //                     .iter()
    //                     .cloned()
    //                     .map(|route| {
    //                         // the problem is here, that trips need to be sorted not by parent id, but by how parent is actually sorted, (by it's parent etc), because it's parent is not actually sorted by id

    //                         // update slice range
    //                         trip_start_index = trip_end_index;
    //                         for trip in trips_by_parent[trip_start_index..].iter() {
    //                             // dbg!(&trip.route_id);
    //                             // dbg!(&route.id);
    //                             if trip.route_id == route.id {
    //                                 trip_end_index += 1;
    //                             } else {
    //                                 break;
    //                             }
    //                         }

    //                         // dbg!(trip_start_index);
    //                         // dbg!(trip_end_index);

    //                         // make Vector from this agency's slice of routes
    //                         let trips = Vector::from_iter(
    //                             trips_by_parent[trip_start_index..trip_end_index]
    //                                 .iter()
    //                                 .cloned()
    //                                 .map(|trip| MyTrip {
    //                                     selected: false,
    //                                     // route: Rc::new(route.clone()),
    //                                     name: trip.id.clone(),
    //                                     stops: Vector::new(),
    //                                 }),
    //                         );
    //                         // let trips =
    //                         //     Vector::from_iter(trips[0..10].iter().cloned().map(|trip| {
    //                         //         MyTrip {
    //                         //             selected: false,
    //                         //             // route: Rc::new(route.clone()),
    //                         //             name: trip.id.clone(),
    //                         //             stops: Vector::new(),
    //                         //         }
    //                         //     }));

    //                         MyRoute {
    //                             selected: false,
    //                             // route: Rc::new(route.clone()),
    //                             name: route.short_name.clone(),
    //                             trips,
    //                         }
    //                     }),
    //             );

    //             MyAgency {
    //                 selected: false,
    //                 // agency: Rc::new(agency.clone()),
    //                 name: agency.name.clone(),
    //                 routes,
    //             }
    //         })
    //         .collect::<Vector<_>>(),
    // };
    println!("make data");
    // let app_data = AppData {
    //     agencies: agencies
    //         .iter()
    //         .map(|agency| MyAgency {
    //             selected: false,
    //             // agency: Rc::new(agency.clone()),
    //             name: agency.name.clone(),
    //             routes: routes
    //                 .iter()
    //                 .filter(|route| route.agency_id == agency.id)
    //                 .map(|route| MyRoute {
    //                     selected: false,
    //                     // route: Rc::new(route.clone()),
    //                     name: route.short_name.clone(),
    //                     trips: Vector::new(),
    //                 })
    //                 .collect::<Vector<_>>(),
    //         })
    //         .collect::<Vector<_>>(),
    // };
    let app_data = AppData {
        agencies: agencies
            .iter()
            .map(|agency| MyAgency {
                selected: false,
                // agency: Rc::new(agency.clone()),
                name: agency.name.clone(),
                routes: routes
                    .iter()
                    .filter(|route| route.agency_id == agency.id)
                    .map(|route| MyRoute {
                        selected: false,
                        // route: Rc::new(route.clone()),
                        name: route.short_name.clone(),
                        trips: trips
                            .iter()
                            .enumerate()
                            .filter(|(i, trip)| trip.route_id == route.id)
                            .map(|(i, trip)| {
                                // dbg!(i);
                                // dbg!(&trip.id);
                                let (start_index, end_index) =
                                    stop_time_range_from_trip_id.get(&trip.id).unwrap().clone();
                                MyTrip {
                                    selected: false,
                                    // trip: Rc::new(trip.clone()),
                                    name: trip.id.clone(),
                                    stops: stop_times[start_index..end_index]
                                        .iter()
                                        // .filter(|stop_time| stop_time.trip_id == trip.id)
                                        .map(|stop_time| MyStopTime {
                                            selected: false,
                                            // stop_time: Rc::new(stop_time.clone()),
                                            // name: gtfs
                                            //     .stops
                                            //     .unwrap()
                                            //     .iter()
                                            //     .find(|stop| stop.id == stop_time.stop_id)
                                            //     .unwrap()
                                            //     .name
                                            //     .clone(),
                                            // name: "dfa".to_string(),
                                            name: stop_map
                                                .get(&stop_time.stop_id)
                                                .unwrap()
                                                .name
                                                .clone(),
                                        })
                                        .collect::<Vector<_>>(),
                                }
                            })
                            .collect::<Vector<_>>(),
                    })
                    .collect::<Vector<_>>(),
            })
            .collect::<Vector<_>>(),
    };
    println!("launch app");

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(app_data)?;
    Ok(())
}
