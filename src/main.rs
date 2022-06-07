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
use gtfs_structures::{Agency, Gtfs, Route, Stop, StopTime, Trip};
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
    #[data(ignore)]
    trip: Rc<Trip>,
    name: String,
    stops: Vector<MyStopTime>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyRoute {
    selected: bool,
    #[data(ignore)]
    route: Rc<Route>,
    name: String,
    trips: Vector<MyTrip>,
}

#[derive(Clone, Data, Default, Lens)]
struct MyAgency {
    selected: bool,
    #[data(ignore)]
    agency: Rc<Agency>,
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
    let gtfs = Gtfs::new(&args.path.expect("must provide a path or url to a GTFS zip"))?;
    let main_window = WindowDesc::new(main_widget())
        .title("Select")
        .window_size((1000., 600.));

    let app_data = AppData {
        agencies: gtfs
            .agencies
            .iter()
            .map(|agency| MyAgency {
                selected: false,
                agency: Rc::new(agency.clone()),
                name: agency.name.clone(),
                routes: gtfs
                    .routes
                    .iter()
                    .filter(|(_, route)| route.agency_id == agency.id)
                    .map(|(_, route)| MyRoute {
                        selected: false,
                        route: Rc::new(route.clone()),
                        name: route.short_name.clone(),
                        trips: gtfs
                            .trips
                            .iter()
                            .filter(|(_, trip)| trip.route_id == route.id)
                            .map(|(_, trip)| MyTrip {
                                selected: false,
                                trip: Rc::new(trip.clone()),
                                name: trip.id.clone(),
                                stops: trip
                                    .stop_times
                                    .iter()
                                    .map(|stop_time| MyStopTime {
                                        selected: false,
                                        // stop_time: Rc::new(stop_time.clone()),
                                        name: stop_time.stop.name.clone(),
                                    })
                                    .collect::<Vector<_>>(),
                            })
                            .collect::<Vector<_>>(),
                    })
                    .collect::<Vector<_>>(),
            })
            .collect::<Vector<_>>(),
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(app_data)?;
    Ok(())
}
