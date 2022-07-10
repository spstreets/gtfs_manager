## Table of Contents

1. [Roadmap/plan](#roadmap/plan)
1. [Todo](#todo)
1. [Questions](#questions)
1. [Features/design](#features/design)
    1. [datasets](#datasets)
        1. [Read/write directly vs Sqlite](#read/write-directly-vs-sqlite)
    1. [editing](#editing)
    1. [UI](#ui)
1. [Dustins message](#dustins-message)
    1. [Route variations](#route-variations)
1. [Map](#map)
    1. [Rendering](#rendering)
        1. [Vector](#vector)
    1. [Tiles](#tiles)
        1. [Vector](#vector)
            1. [MVT](#mvt)
    1. [other](#other)
    1. [servers](#servers)
    1. [OpenStreetMap](#openstreetmap)
1. [GTFS/transit Data](#gtfs/transit-data)
    1. [Relevant organisations](#relevant-organisations)
        1. [Overview](#overview)
        1. [Navitia](#navitia)
        1. [transit land](#transit-land)
        1. [mobilitydata](#mobilitydata)
        1. [google mobility data?](#google-mobility-data?)
    1. [Parsing](#parsing)
        1. [Rust](#rust)
        1. [Other langs](#other-langs)
    1. [manager/editor/viewer](#manager/editor/viewer)
    1. [validation](#validation)
        1. [Rust](#rust)
        1. [Other Langs](#other-langs)
    1. [Other Specs/Formats](#other-specs/formats)
        1. [TransXChange](#transxchange)
            1. [TransXChange to GTFS](#transxchange-to-gtfs)
    1. [Data catalogues](#data-catalogues)
    1. [Other](#other)
1. [Server applications](#server-applications)
1. [Exeter](#exeter)
1. [Commerical apps](#commerical-apps)

# Roadmap/plan

better to just get some UI working in the simplest way first, then discuss what an ideal UI would look like?

1. simple table based data viewer for GTFS data.  
   Either parse data directly using https://crates.io/crates/gtfs-structures or https://github.com/hove-io/transit_model, or import to sqlite and query directly from sqlite. Should also show schedule and headways.
2. Add static map for displaying routes and stops. Start off with raster images for ease, and look to replace with maplibre-rs as it matures.
3. Add tooltips to show route/stop names
4. Make routes/stops clickable to navigate to item in table.
5. Make map zoomable.

# Todo

-   [ ] "Some kind of UI to inspect and ask questions about a GTFS dataset"
-   [ ] "see all the variations of a route"
-   [ ] "editing -- maybe removing or moving stops, timetable changes -- either blindly or informed by data that operators might care about (like isochrone-based census calculations)"

# Questions

-   What features does the app need?
-   What is a list of typical questions that are asked when looking at GTFS data?
-   What is a typical workflow: find a dataset, download/open it and understand it, make changes to it, share changes with other people, keep track of older changes, etc.

# Features/design

remember KISS, do the simplest thing first

We are not interested in fares etc, just the geographic aspect of the data.

In order to encourage collaboration and contribution and help empower other projects, make use of existing crates where possible, and break any useful functionality into separate crates to allow it to be reused by others.

Make it easy to change/develop/build upon the app to make contributions easy, and make it easy for the app to be customised to specific use cases.

Cross platform and should also run on the web to make demoing the app easier.

## datasets

from http://cheesehead-techblog.blogspot.com/2014/03/importing-gtfs-files-into-sqlite.html:
"Each provider's GTFS file can include many optional fields, and may use different optional fields over time, so you are _likely_ to need to tweak this script a bit to get it to work:"
I'm assuming this is referring to optional fields that are in the spec. But I guess the could be extra fields not in the spec, also there seems to be GTFS "extensions".

provide list of downloaded GTFSs, their date and url. allow for deleting the data to preserve space on hd, but keep ghost record of dataset name, date, and url so if they want to use it again they can just click to download again, ie deleting data doesn't completing forget it.

gtfs data can be large, so better if we don't have to read it all in to memory?: https://stackoverflow.com/questions/26892634/splitting-gtfs-transit-data-into-smaller-ones

### Sample data

The spec repo contains some [sample data](https://github.com/google/transit/tree/master/gtfs/spec/en/examples/sample-feed-1) but for some reason the shapes.txt is empty. So instead trim down the Stagecoach South West GTFS to make some sample data.

### Read/write directly vs Sqlite

We can either read data directly from the GTFS zip, or we can copy this data into SQLite and only work with SQLite. Given GTFS data is essentially modelled as a relational db, reading it into a SQLite seems like it could be a quick win.

TLDR: don't bother with SQLite initially

SQLite would allow for fast persistance of edits
Would still then need to save/export to GTFS format. A problem with this is avoiding overwriting/loosing any additional tables/fields in the GTFS that were not read into SQLite. Could always handle this when converting from SQLite to GTFS by also reading the original file, which is essentially what we would need to do for saving in the direct approach anyway.

SQLite could allow faster app startup/data loading time
~100mb datasets loads can be parsed in a few seconds, so not a huge problem, but using SQLite we could save to a db file, cache this, and use it to load data instantly in the future. will need to ensure original file has not changed and cached db is still valid. Also could always serialize data into a faster format like bincode and cache it without SQLite which would probably make the load time negligible. SQLite would also be slower in most cases as unless the dataset has already been opened and copied to SQLite, will need need first parse and insert the data to SQLite, then read it from SQLite anyway.

SQLite could allow faster querying
It seems like the app is going to need the entire dataset loaded into memory anyway, given most of the size is from shapes.txt and we will also want to immediately display all the route on the map, so being able to store the data on disk and query a subset of it doesn't seem to be necessary.

SQLite would allow working with very large datasets
It doesn't seem like any individual datasets will be too large to read into memory, but this could be useful for working with multiple datasets. But again, either we display multiple datasets at the same time in which case they need to fit in memory, else we are switching between them and can just drop them then load the next one, which could be quick for switching back and forth if we serialize them.

## editing

won't necessarily want autosave/change on edit, might want to experiement with changes, edit undo, then save which also allows for slower saving.

each change should be recorded as a piece of data which appears in a list and can be deleted/undo-ed. then hit change to save your changes
update files in place: https://users.rust-lang.org/t/how-to-modify-content-inside-a-file-following-a-match/65135/6

## UI

as well as a tailored UI might be useful to be able to switch to a simple db/table view eg for checking raw data/data integrity etc

have four column lists, in order: agencies, routes, trips, stops. (multiple) selection of an item(s) in the list will filter the subsequent columns.
will also highlight the item on the map
can also select routes/stops by clicking on the map
would be nice to be able to click on a stop and it will highlight all the trips that use it
I think displaying all the lists simultaneously is going to take up too much space and obscure the map. Plus once an item is selected, the rest of the list is pretty redundant while investigating subsequent lists. better to do something like: first only show agencies, then once one is selected only show routes etc.

# Dustins message

The two openish GTFS-related things I've found are https://www.transit.land/routes/r-gcj8-2b and https://github.com/conveyal/taui (and Conveyal more generally). But to my knowledge, there's nothing that lets you just see all the variations of a route, click stops and see the schedule, headways, etc. Some kind of UI to inspect and ask questions about a GTFS dataset would be super useful, IMO. Then a next step could be editing -- maybe removing or moving stops, timetable changes -- either blindly or informed by data that operators might care about (like isochrone-based census calculations).

Commercial people in this space are https://www.podaris.com/ and https://www.remix.com/solutions/transit

podaris' free web app doesn't seem to have any GTFS support

isochrone is like a heat map showing travel times from a given point: https://en.wikipedia.org/wiki/Isochrone_map

## Route variations

-   https://groups.google.com/g/transit-developers/c/AOdm2RI--Uw
-   https://groups.google.com/g/gtfs-changes/c/7YrouCQqkjY/m/cabHu0MdAgAJ
-   https://github.com/google/transit/blob/master/gtfs/spec/en/style-guide.md
    seems different agencies record variations in different ways

# Map

Supports many different APIs and vector formats, useful for references:
https://github.com/openlayers/openlayers
https://openlayers.org/

## Rendering

### Vector

https://github.com/maplibre/maplibre-rs

## Tiles

This seems kind of good and won an OSM award but also has commerical backing and tries to sell stuff: https://openmaptiles.org/about/

leaflet does raster tiles, mapbox does vector: https://openmaptiles.org/docs/website/leaflet/

good overview: https://docs.mapbox.com/data/tilesets/guides/vector-tiles-standards/

### Vector

PostGIS to MVT: https://github.com/wyyerd/vectortile-rs

MBTiles for world is around 100TB

#### MVT

https://github.com/t-rex-tileserver/t-rex

encode MVT: https://github.com/DougLau/mvt
read and write (?) https://github.com/amandasaurus/rust-mapbox-vector-tile

Motivated by the 2020 transition to proprietary licensing for Mapbox GL JS
https://github.com/MapLibre

https://github.com/maplibre/maplibre-rs

##### Parsing

https://github.com/mapbox/vector-tile-js

## File formats

KML
GPX
GeoJson
TopoJson
Geobuf
PBF

### Conversion

https://github.com/OSGeo/gdal

## other

"Zero-Copy reading and writing of geospatial data."
https://github.com/georust/geozero
not 100% sure what it is, but flatgeobuf seems to be most popular implementation and is actually used by abstreets here: https://github.com/a-b-street/abstreet/blob/master/popdat/src/lib.rs
https://github.com/flatgeobuf/flatgeobuf
says it is inspired by https://github.com/mapbox/geobuf which basically seems like a binary geojson

https://github.com/a-b-street/abstreet/blob/master/popdat/src/lib.rs

## servers

-   https://github.com/urbica/martin/
-   https://github.com/mapbox/Hecate
-   https://github.com/maplibre/mbtileserver-rs

## OpenStreetMap

Seems like a well featured flutter app for managing OSM data
https://github.com/Zverik/every_door

https://crates.io/crates/osm-tile-downloader Download tiles from an OpenStreetMap tileserver to the file system

# GTFS/transit Data

## Sample Data

https://github.com/google/transit/tree/master/gtfs/spec/en/examples/sample-feed-1 has an empty shapes.txt for some reason, so will create our own sample data for the app.

## Spec

### Schemas

https://www.researchgate.net/figure/The-GTFS-Schema-for-the-data-from-JSP-Skopje_fig1_263853949
https://opentransportdata.swiss/en/cookbook/gtfs/

## Relevant organisations

### Overview

Google seems to own the spec, not gfts.org/MobilityData: https://developers.google.com/transit/gtfs/guides/changes-overview

interestingly, https://code.google.com/p/googletransitdatafeed/ now forwards to a mobilitydata repo

directly read GTFS: https://github.com/georust/transitfeed

https://github.com/orgs/georust/repositories
https://project-awesome.org/CUTR-at-USF/awesome-transit

http://www.gtfs-data-exchange.com/ is deprecated and recommends transitland (has corporate backing) or https://transitfeeds.com/ (is on github) as alternatives

### Navitia

https://github.com/hove-io/navitia

### transit land

seems to maintain a registry of data sources which it periodically checks for updates and saves the data into a postgres cluster and cloud object storage.

https://www.transit.land/feeds seems like a good source and they regularly check for updates. searching for "bus" brings up f-bus~dft~gov~uk.

https://www.transit.land/map#14.46/50.72963/-3.51308 is a map showing all the bus route but only lets you click on a route and show basic info like it's name, no info for stops, timetable, etc. Seems to get it GTFS data from https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/. Gives feeds a "Onestop ID"
https://www.transit.land/documentation/onestop-id-scheme/
https://github.com/openvenues/libpostal
https://crates.io/crates/rustpostal

### mobilitydata

maintains https://gtfs.org/

transitfeeds.com is a mess (basically also deprecated).  
its github repo changed name to openmobilitydata but is now deleted.  
the site has few uk datasets: https://openmobilitydata.org/l/178-england-uk.  
there is a mirror url https://openmobilitydata.org/, both of which point to https://github.com/MobilityData/mobility-database-catalogs  
It seems like https://database.mobilitydata.org/ is the current project, which is currently active, and seems like large and ambitious, but I haven't looked into it properly yet.

JSON and CSV files on GitHub that is a repository of 1300+ mobility datasets across the world. Contains contents of OpenMobilityData/TransitFeeds.com.

### google mobility data?

## Parsing

### Rust

-   https://github.com/nicomazz/fastgtfs parse and query
-   https://github.com/CommuteStream/tflgtfs fetches TfL data and converts it to GTFS
-   https://gitlab.com/jistr/townhopper cli for querying
-   https://crates.io/crates/gtfs-geojson GTFS to GeoJson
-   https://crates.io/crates/gtfs-structures parses GTFS data with serde
-   https://github.com/hove-io/transit_model converts between various formats, making use of the "NTFS model" used by Navitia

### Other langs

-   https://github.com/google/transitfeed A Python library for reading, validating, and writing transit schedule information in the GTFS format.
-   https://github.com/MobilityData/gtfs-validator Java validator
-   https://github.com/interline-io/transitland-lib golang Library and tool for reading, writing, and processing transit data
-   https://github.com/tyleragreen/gtfs-schema postgres schema
-   https://github.com/aytee17/gtfs-to-sqlite/blob/master/src/main/resources/GTFS_Specification.json java lib with json spec for sqlite types
-   https://github.com/4rterius/cgtfs c lib can parse into a sqlite db
-   https://github.com/BlinkTagInc/node-gtfs js lib can parse into a sqlite db
-   https://github.com/OpenTransitTools/gtfsdb python lib can parse into a sqlite db and query
-   https://github.com/CxAalto/gtfspy python lib can parse into a sqlite db and query
-   https://github.com/vasile/GTFS-viz ruby lib can parse into a sqlite db and geojson
-   https://github.com/atlregional/bus-router python lib creates a shapes.txt file for stop_times.txt using OSRM. creates valid GeoJSON from your shapes.txt very old - python 2.7
-   https://github.com/ad-freiburg/pfaedle c++ lib generate GTFS shapes from OSM data
-   https://github.com/HSLdevcom/gtfs_shape_mapfit python lib fits GTFS shape files and stops to a given OSM map file

## manager/editor/viewer

https://github.com/ibi-group/datatools-ui js front end for java backend: https://github.com/ibi-group/datatools-server successors to conveyal's https://github.com/conveyal/gtfs-editor
https://github.com/WRI-Cities/static-GTFS-manager for creating GTFS - has online tool but no dummy data. uploading south west GTFS errored. also failed with mobilitydata sample zip
https://github.com/google/transitfeed/wiki/ScheduleViewer is a python web app, use google maps.

## validation

### Rust

https://github.com/etalab/transport-validator/

### Other Langs

## Other Specs/Formats

### TransXChange

UK standard format
https://www.gov.uk/government/collections/transxchange

#### TransXChange to GTFS

https://itsleeds.github.io/UK2GTFS/articles/transxchange.html R code
https://github.com/danbillingsley/TransXChange2GTFS C#

## Data catalogues

https://github.com/transitland/transitland-atlas
Includes feeds in the following data specifications (specs):

-   GTFS
-   GTFS Realtime
-   GBFS - automatically synchronized from https://github.com/NABSA/gbfs/blob/master/systems.csv
    MDS - automatically synchronized from https://github.com/openmobilityfoundation/mobility-data-specification/blob/main/providers.csv

## Other

https://github.com/bliksemlabs/rrrr C-language implementation of the RAPTOR public transit routing algorithm

# Server applications

https://github.com/hove-io/navitia
https://api.navitia.io/
Navitia is a python webservice, with a seemingly free API, providing:

-   multi-modal journeys computation
-   line schedules
-   next departures
-   exploration of public transport data
-   search & autocomplete on places
-   things such as isochrones

https://github.com/Open-Transport/synthese
provides network modeling, passenger information, DRT reservation, CMS, real time data updating, and operations optimization

https://github.com/opentripplanner/OpenTripPlanner/ multi-modal trip planner, focusing on travel by scheduled public transportation in combination with bicycling, walking, and mobility services including bike share and ride hailing

# Exeter

-   https://data.bus-data.dft.gov.uk/timetable/dataset/2059/
-   https://www.stagecoachbus.com/open-data

# Commerical apps

-   Citymapper
-   https://transitapp.com/
