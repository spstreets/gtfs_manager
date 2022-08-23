# Gtfs Manager
A desktop GUI app for viewing and editing GTFS data.  
Built with [Druid](https://github.com/linebender/druid), a rust native UI toolkit.

## Status

The current state of the app is a rough prototype and further work is required.  
Contributions and collaborators are welcome.


### Current features
* Display GTFS data on a map with panning and zooming
* Select individual trips and stops and display their metadata
* Add new routes, trips, and stops to the map

### Future improvements

* Managing Edits: Edit/undo, easily view changes that have been made to the dataset, compare two different datasets.
* File management: Open, save, and close GTFS datasets from the file dialog.
* Web version: host a compiled to wasm and rendered in web canvas (as supported by Druid) version in order to provide a convenient demo.
* Database of GTFS data: Support discovering and importing datasets from https://www.transit.land/feeds.
* Map background: Currently only the GTFS routes themselves are displayed on the map. Add a background map to provide context.
Add much more...

## Usage instructions

We suggest you update to the latest version of `rustc` before trying to install Gtfs Manager, to ensure you meet the minimum supported rust version:
```bash
rustup update
```

Install gtfs_manager:
```bash
cargo install --git https://github.com/spstreets/gtfs_manager
```

Now we can open a GTFS zip file by providing the path or URL of a file to gtfs_manager. The below example opens a Sao Paulo GTFS file which is stored on Github.
```bash
gtfs_manager https://github.com/spstreets/gtfs_manager/releases/download/v0.1.0/sao-paulo-sptrans.zip
```

Alternatively you can clone the repository and build and run gtfs_manager using `cargo run`. Larger datasets will take a long time to load on debug builds, in which case it is recommended to build with the `--release` flag. For example:
```bash
cargo run --release https://github.com/spstreets/gtfs_manager/releases/download/v0.1.0/sao-paulo-sptrans.zip
```