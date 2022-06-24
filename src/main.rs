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
    AppDelegate, AppLauncher, Color, Data, Env, Insets, Lens, LocalizedString, UnitPoint, Widget,
    WidgetExt, WindowDesc,
};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use gtfs_manager::{main_widget, make_initial_data, AppData, Delegate, ListItem, MapWidget};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Optional path to a GTFS zip. If missing demo data will be loaded
    pub path: Option<String>,
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
        .window_size((1400., 1000.));

    println!("launching app");
    AppLauncher::with_window(main_window)
        .delegate(Delegate {})
        // .log_to_console()
        .launch(initial_data)?;
    Ok(())
}
