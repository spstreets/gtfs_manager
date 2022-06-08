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
    stop_sequence: u16,
    // #[data(ignore)]
    // #[lens(ignore)]
    // stop_time: Rc<RawStopTime>,
    // stop_time: RawStopTime,
    name: String,
}

#[derive(Clone, Data, Default, Lens)]
struct MyTrip {
    selected: bool,
    // #[data(ignore)]
    // trip: RawTrip,
    name: String,
    stops: Vector<MyStopTime>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyRoute {
    selected: bool,
    #[data(ignore)]
    route: Route,
    trips: Vector<MyTrip>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyAgency {
    selected: bool,
    #[data(ignore)]
    agency: Agency,
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
            format!("{}", data.route.short_name)
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
            format!("{}", data.agency.name)
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

fn make_initial_data(gtfs: RawGtfs) -> AppData {
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

    agencies.sort_by(|x1, x2| x1.name.cmp(&x2.name));

    let app_data = AppData {
        agencies: agencies
            .iter()
            .map(|agency| {
                let mut routes = routes
                    .iter()
                    .filter(|route| route.agency_id == agency.id)
                    .map(|route| MyRoute {
                        selected: false,
                        route: route.clone(),
                        trips: trips
                            .iter()
                            .enumerate()
                            .filter(|(i, trip)| trip.route_id == route.id)
                            .map(|(i, trip)| {
                                let (start_index, end_index) =
                                    stop_time_range_from_trip_id.get(&trip.id).unwrap().clone();
                                let mut stops = stop_times[start_index..end_index]
                                    .iter()
                                    // .filter(|stop_time| stop_time.trip_id == trip.id)
                                    .map(|stop_time| MyStopTime {
                                        selected: false,
                                        stop_sequence: stop_time.stop_sequence,
                                        // stop_time: Rc::new(stop_time.clone()),
                                        // stop_time: stop_time.clone(),
                                        name: stop_map
                                            .get(&stop_time.stop_id)
                                            .unwrap()
                                            .name
                                            .clone(),
                                    })
                                    .collect::<Vector<_>>();
                                stops.sort_by(|stop1, stop2| {
                                    stop1.stop_sequence.cmp(&stop2.stop_sequence)
                                });

                                // adding the RawTrip to MyTrip is the tipping point which kills performance. Maybe AppData should just be storing a u32 index of the items position in the original RawGtfs data
                                MyTrip {
                                    selected: false,
                                    // trip: Rc::new(trip.clone()),
                                    name: trip.id.clone(),
                                    stops,
                                }
                            })
                            .collect::<Vector<_>>(),
                    })
                    .collect::<Vector<_>>();
                routes.sort_by(|route1, route2| {
                    route1.route.short_name.cmp(&route2.route.short_name)
                });
                MyAgency {
                    selected: false,
                    agency: agency.clone(),
                    routes,
                }
            })
            .collect::<Vector<_>>(),
    };
    app_data
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    println!("reading raw gtfs");
    let gtfs = RawGtfs::new(&args.path.expect("must provide a path or url to a GTFS zip"))?;
    gtfs.print_stats();

    println!("making initial data");
    let initial_data = make_initial_data(gtfs);

    println!("making main window");
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((1000., 600.));

    println!("launching app");
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_data)?;
    Ok(())
}
