// ignore unused warnings while prototyping
#![allow(unused)]
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use clap::Parser;
use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::lens::{self, LensExt};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, FlexParams, Label, List, MainAxisAlignment, Scroll,
};
use druid::{
    AppLauncher, Color, Data, Env, Insets, Lens, LocalizedString, UnitPoint, Widget, WidgetExt,
    WindowDesc,
};
use gtfs_structures::{Gtfs, Route};
use std::error::Error;

use gtfs_manager::ListItem;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Optional path to a GTFS zip. If missing demo data will be loaded
    pub path: Option<String>,
}

#[derive(Data, Clone, Lens)]
struct AppData {
    route: String,
    trip: String,
    stop: u16,
}

fn mylist<T: Data>(data: &Vec<(String, T)>) -> impl Widget<T> {
    let mut mycol = Flex::column().cross_axis_alignment(CrossAxisAlignment::Fill);
    data.iter().for_each(|(name, id)| {
        mycol.add_child(ListItem::new(name.clone(), id.clone()));
    });
    mycol
}

fn main_widget(gtfs: &Gtfs) -> impl Widget<AppData> {
    let mut row = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);
    let routes = gtfs
        .routes
        .iter()
        .map(|(_, route)| (route.short_name.clone(), route.id.clone()))
        .collect::<Vec<_>>();
    row.add_flex_child(Scroll::new(mylist(&routes)).lens(AppData::route), 1.0);

    let trips = gtfs
        .trips
        .iter()
        .map(|(_, trip)| (trip.id.clone(), trip.id.clone()))
        .collect::<Vec<_>>();
    row.add_flex_child(Scroll::new(mylist(&trips)).lens(AppData::trip), 1.0);

    let stops = gtfs
        .trips
        .iter()
        .next()
        .unwrap()
        .1
        .stop_times
        .iter()
        .map(|stop_time| {
            (
                stop_time.stop.as_ref().name.clone(),
                stop_time.stop_sequence,
            )
        })
        .collect::<Vec<_>>();
    row.add_flex_child(Scroll::new(mylist(&stops)).lens(AppData::stop), 1.0);

    let myroutes = gtfs.routes.clone();
    let mystoptimes = gtfs.trips.iter().next().unwrap().1.stop_times.clone();
    row.add_child(
        Label::new(move |data: &AppData, _: &Env| {
            format!(
                "Map highlighting\nStop {:?}\nfrom Trip {:?}\nfrom Route {:?}",
                mystoptimes
                    .iter()
                    .find(|stop| stop.stop_sequence == data.stop)
                    .unwrap()
                    .clone()
                    .stop
                    .as_ref()
                    .name
                    .clone(),
                data.trip,
                myroutes.get(&data.route).unwrap().clone().short_name
            )
        })
        .padding(Insets::uniform_xy(5., 5.)),
    );
    row
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    let gtfs = Gtfs::new(&args.path.expect("must provide a path or url to a GTFS zip"))?;
    let main_window = WindowDesc::new(main_widget(&gtfs))
        .title("Select")
        .window_size((1000., 600.));

    let app_data = AppData {
        route: gtfs.routes.iter().next().unwrap().1.id.clone(),
        trip: gtfs.trips.iter().next().unwrap().1.id.clone(),
        stop: gtfs.trips.iter().next().unwrap().1.stop_times[0].stop_sequence,
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(app_data)?;
    Ok(())
}
