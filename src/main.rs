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
    // stop_sequence: u16,
    // #[data(ignore)]
    // #[lens(ignore)]
    // stop_time: Rc<RawStopTime>,
    // stop_time: RawStopTime,
    // name: String,
    index: usize,
}

#[derive(Clone, Data, Default, Lens)]
struct MyTrip {
    selected: bool,
    // #[data(ignore)]
    // trip: RawTrip,
    // name: String,
    index: usize,
    stops: Vector<MyStopTime>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyRoute {
    selected: bool,
    index: usize,
    // #[data(ignore)]
    // route: Route,
    trips: Vector<MyTrip>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyAgency {
    selected: bool,
    index: usize,
    // #[data(ignore)]
    // agency: Agency,
    routes: Vector<MyRoute>,
}

#[derive(Clone, Data, Default, Lens)]
struct AppData {
    agencies: Vector<MyAgency>,
}

fn stop_ui(gtfs: Rc<RawGtfs>) -> impl Widget<MyStopTime> {
    let gtfs2 = Rc::clone(&gtfs);
    Flex::row()
        .with_child(Checkbox::new("").lens(MyStopTime::selected))
        .with_child(Label::new(move |data: &MyStopTime, _env: &_| {
            format!("{}", gtfs2.stop_times.as_ref().unwrap()[data.index].stop_id)
        }))
        // .with_child(Either::new(
        //     |data: &Trip, _env: &Env| data.selected,
        //     List::new(stop_ui).lens(Trip::stops),
        //     Label::new(""),
        // ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}
fn trip_ui(gtfs: Rc<RawGtfs>) -> impl Widget<MyTrip> {
    let gtfs2 = Rc::clone(&gtfs);
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyTrip::selected))
        .with_child(Label::new(move |data: &MyTrip, _env: &_| {
            format!("{}", gtfs2.trips.as_ref().unwrap()[data.index].id)
        }));

    Flex::column()
        .with_child(title)
        .with_child(Either::new(
            |data: &MyTrip, _env: &Env| data.selected,
            List::new(move || stop_ui(Rc::clone(&gtfs))).lens(MyTrip::stops),
            Flex::row(),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}
fn route_ui(gtfs: Rc<RawGtfs>) -> impl Widget<MyRoute> {
    let gtfs2 = Rc::clone(&gtfs);
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyRoute::selected))
        .with_child(Label::new(move |data: &MyRoute, _env: &_| {
            format!("{}", gtfs2.routes.as_ref().unwrap()[data.index].short_name)
        }));

    Flex::column()
        .with_child(title)
        .with_child(Either::new(
            |data: &MyRoute, _env: &Env| data.selected,
            List::new(move || trip_ui(Rc::clone(&gtfs))).lens(MyRoute::trips),
            Flex::row(),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}
fn agency_ui(gtfs: Rc<RawGtfs>) -> impl Widget<MyAgency> {
    let gtfs2 = Rc::clone(&gtfs);
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyAgency::selected))
        .with_child(Label::new(move |data: &MyAgency, _env: &_| {
            format!("{}", gtfs2.agencies.as_ref().unwrap()[data.index].name)
        }));

    Flex::column()
        .with_child(title)
        .with_child(Either::new(
            |data: &MyAgency, _env: &Env| data.selected,
            List::new(move || route_ui(Rc::clone(&gtfs))).lens(MyAgency::routes),
            Flex::row(),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding((10., 0., 0., 0.))
}

fn main_widget(gtfs: Rc<RawGtfs>) -> impl Widget<AppData> {
    Scroll::new(List::new(move || agency_ui(Rc::clone(&gtfs))).lens(AppData::agencies))
}

fn make_initial_data(gtfs: Rc<RawGtfs>) -> AppData {
    let mut agencies = gtfs.agencies.as_ref().unwrap().clone();
    let mut routes = gtfs.routes.as_ref().unwrap().clone();
    let mut trips = gtfs.trips.as_ref().unwrap().clone();
    let mut stop_times = gtfs.stop_times.as_ref().unwrap().clone();
    let mut stops = gtfs.stops.as_ref().unwrap().clone();

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
            .enumerate()
            .map(|(agency_index, agency)| {
                let mut routes = routes
                    .iter()
                    .enumerate()
                    .filter(|(_, route)| route.agency_id == agency.id)
                    .map(|(index, route)| MyRoute {
                        selected: false,
                        index,
                        // route: route.clone(),
                        trips: trips
                            .iter()
                            .enumerate()
                            .filter(|(i, trip)| trip.route_id == route.id)
                            .map(|(i, trip)| {
                                let (start_index, end_index) =
                                    stop_time_range_from_trip_id.get(&trip.id).unwrap().clone();
                                let mut stops = stop_times[start_index..end_index]
                                    .iter()
                                    .enumerate()
                                    // .filter(|stop_time| stop_time.trip_id == trip.id)
                                    .map(|(st_index, stop_time)| MyStopTime {
                                        selected: false,
                                        // stop_sequence: stop_time.stop_sequence,
                                        // stop_time: Rc::new(stop_time.clone()),
                                        // stop_time: stop_time.clone(),
                                        // name: stop_map
                                        //     .get(&stop_time.stop_id)
                                        //     .unwrap()
                                        //     .name
                                        //     .clone(),
                                        index: st_index + start_index,
                                    })
                                    .collect::<Vector<_>>();
                                // stops.sort_by(|stop1, stop2| {
                                //     stop1.stop_sequence.cmp(&stop2.stop_sequence)
                                // });

                                // adding the RawTrip to MyTrip is the tipping point which kills performance. Maybe AppData should just be storing a u32 index of the items position in the original RawGtfs data
                                MyTrip {
                                    selected: false,
                                    // trip: Rc::new(trip.clone()),
                                    index: i,
                                    // name: trip.id.clone(),
                                    stops,
                                }
                            })
                            .collect::<Vector<_>>(),
                    })
                    .collect::<Vector<_>>();
                // routes.sort_by(|route1, route2| {
                //     route1.route.short_name.cmp(&route2.route.short_name)
                // });
                MyAgency {
                    selected: false,
                    index: agency_index,
                    // agency: agency.clone(),
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
    let gtfs = Rc::new(RawGtfs::new(
        &args.path.expect("must provide a path or url to a GTFS zip"),
    )?);
    gtfs.print_stats();

    println!("making initial data");
    let initial_data = make_initial_data(Rc::clone(&gtfs));

    println!("making main window");
    let main_window = WindowDesc::new(main_widget(Rc::clone(&gtfs)))
        .title("Select")
        .window_size((1000., 600.));

    println!("launching app");
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_data)?;
    Ok(())
}
