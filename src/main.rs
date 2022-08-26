// ignore unused warnings while prototyping
// #![allow(unused)]
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use chrono::Utc;
use clap::Parser;
use druid::{AppLauncher, Color, WindowDesc};
use gtfs_structures::RawGtfs;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use gtfs_manager::{
    main_widget, make_initial_data, AppData, Delegate, VARIABLE_STOP_TIME_BORDER_COLOR,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Optional path to a GTFS zip. If missing demo data will be loaded
    pub path: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    // experimenting with storing a demo dataset as bincode but couldn't get the data to deserialize to bincode using gtfs_structures' custom serde implementations
    let path = Path::new("sao-paulo-sptrans.bincode");
    // let path = Path::new("sao-paulo-sptrans.json");
    let initial_data = if path.exists() {
        println!("{:?} json: read from disk", Utc::now());
        // let file = fs::File::open(path)?;

        println!("{:?} json: deserialize", Utc::now());
        let input = File::open(path)?;
        let buffered = BufReader::new(input);
        let initial_data: AppData = bincode::deserialize_from(buffered)?;
        // let initial_data: AppData = serde_json::from_reader(file)?;
        initial_data
    } else {
        println!("reading raw gtfs");
        let mut gtfs = if let Some(path) = &args.path {
            RawGtfs::new(path)?
        } else {
            panic!("sfad")
        };
        gtfs.print_stats();

        println!("making initial data");
        let initial_data = make_initial_data(&mut gtfs);

        // // bincode
        // let bincode_path = "sao-paulo-sptrans.bincode";
        // {
        //     println!("{:?} bincode: serialize", Utc::now());
        //     let se_bincode_vec: Vec<u8> = bincode::serialize(&initial_data).unwrap();

        //     println!("{:?} bincode: write to disk", Utc::now());
        //     fs::write(bincode_path, se_bincode_vec).expect("write failed");
        // }

        // println!("{:?} bincode: read from disk", Utc::now());
        // let read_bincode_string = fs::read(bincode_path).unwrap();

        // println!("{:?} bincode: deserialize", Utc::now());
        // let initial_data: AppData = bincode::deserialize(&read_bincode_string[..]).unwrap();

        // json
        {
            // println!("{:?} json: serialize", Utc::now());
            // // let serialized_data = serde_json::to_string(&initial_data)?;
            // let serialized_data: Vec<u8> = bincode::serialize(&initial_data)?;

            // println!("{:?} json: write to disk", Utc::now());
            // fs::write(path, serialized_data)?;
        }
        initial_data
    };

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
        .configure_env(|env, _state| {
            env.set(
                VARIABLE_STOP_TIME_BORDER_COLOR,
                Color::rgb(54. / 255., 58. / 255., 74. / 255.),
            );
        })
        .delegate(Delegate {})
        // .log_to_console()
        .launch(initial_data)?;
    Ok(())
}
