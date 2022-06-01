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

use gtfs_manager::{DropdownSelect, ListSelect};

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

fn main_widget(gtfs: &Gtfs) -> impl Widget<AppData> {
    let mut row = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);
    row.add_flex_child(
        Scroll::new(
            ListSelect::new(
                gtfs.routes
                    .iter()
                    .map(|(_, route)| (route.short_name.clone(), route.id.clone())),
            )
            .on_select(|_, item, _| {}),
        )
        .lens(AppData::route),
        1.0,
    );
    row.add_default_spacer();

    row.add_flex_child(
        Scroll::new(ListSelect::new(gtfs.trips.iter().map(|(_, trip)| {
            // todo this will panic if id is less than 6 bytes
            (trip.id.clone(), trip.id.clone())
        })))
        .lens(AppData::trip),
        1.0,
    );
    row.add_default_spacer();

    row.add_flex_child(
        Scroll::new(ListSelect::new(
            gtfs.trips
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
                }),
        ))
        .lens(AppData::stop),
        1.0,
    );
    row.add_default_spacer();

    row.add_child(
        Label::new(|d: &AppData, _: &Env| {
            format!(
                "Map with Trip {:?} from Route {:?} highlighted",
                d.trip, d.route
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
