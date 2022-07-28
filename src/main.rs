// ignore unused warnings while prototyping
#![allow(unused)]
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use chrono::Utc;
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
use std::fs::{self, File};
use std::io::BufReader;
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
    let mut gtfs = if let Some(path) = &args.path {
        RawGtfs::new(path)?
    } else {
        panic!("sfad")
    };
    gtfs.print_stats();

    println!("making initial data");
    let initial_data = make_initial_data(&mut gtfs);

    // bincode
    let bincode_path = "sao-paulo-sptrans.bincode";
    {
        println!("{:?} bincode: serialize", Utc::now());
        // println!("{:?} bincode: serialize", start.elapsed());
        let se_bincode_vec: Vec<u8> = bincode::serialize(&initial_data).unwrap();

        println!("{:?} bincode: write to disk", Utc::now());
        // println!("{:?} bincode: write to disk", start.elapsed());
        fs::write(bincode_path, se_bincode_vec).expect("write failed");
    }

    println!("{:?} bincode: read from disk", Utc::now());
    // println!("{:?} bincode: read from disk", start.elapsed());
    let read_bincode_string = fs::read(bincode_path).unwrap();

    println!("{:?} bincode: deserialize", Utc::now());
    // println!("{:?} bincode: deserialize", start.elapsed());
    let initial_data: AppData = bincode::deserialize(&read_bincode_string[..]).unwrap();

    // println!("{:?} bincode: deserialize directly", Utc::now());
    // // println!("{:?} bincode: deserialize directly", start.elapsed());
    // let input = File::open(bincode_path).unwrap();
    // let buffered = BufReader::new(input);
    // let de_bincode: AppData = bincode::deserialize_from(buffered).unwrap();

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
