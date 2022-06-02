# gtfs_manager
A GUI for viewing and editing GTFS data

## Usage instructions

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