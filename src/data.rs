use chrono::Utc;
use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::kurbo::BezPath;
use druid::{Data, Lens, Point, Rect, Widget, WidgetExt};
use gtfs_structures::{
    Agency, Availability, BikesAllowedType, ContinuousPickupDropOff, DirectionType, Gtfs,
    LocationType, Pathway, PickupDropOffType, RawGtfs, RawStopTime, RawTrip, Route, RouteType,
    Shape, Stop, StopTime, StopTransfer, TimepointType, Trip,
};
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::rc::Rc;
use uuid::Uuid;

mod newtypes;
pub use newtypes::*;

pub trait ListItem {
    fn new_child(&mut self) -> String;
    fn update_all(&mut self, value: bool);
    fn id(&self) -> String;
    fn n_stops(&self) -> usize {
        99
    }
    fn show_editing(&self) -> bool {
        false
    }
    fn item_type(&self) -> String;
    // fn data_info(&self) -> String;
    fn data_info(&self) -> String {
        "not implemented".to_string()
    }
    fn selected(&self) -> bool;
    // fn name(&self) -> String;
}

#[derive(Clone, Data, Debug, Lens, Serialize, Deserialize)]
pub struct MyStop {
    pub live: bool,
    pub selected: bool,
    pub scroll_to_me: usize,

    pub id: String,
    pub code: Option<String>,
    pub name: String,
    pub description: String,
    pub location_type: MyLocationType,
    pub parent_station: Option<String>,
    pub zone_id: Option<String>,
    pub url: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub timezone: Option<String>,
    pub wheelchair_boarding: MyAvailability,
    pub level_id: Option<String>,
    pub platform_code: Option<String>,
    // pub transfers: Vec<StopTransfer>,
    // pub pathways: Vec<Pathway>,
    pub transfers: usize,
    pub pathways: usize,

    #[data(ignore)]
    #[lens(ignore)]
    pub stop: Option<Rc<Stop>>,
    // stop_time: RawStopTime,
    #[serde(skip)]
    pub latlong: Point,
}
impl MyStop {
    pub fn new(latlong: Point) -> MyStop {
        MyStop {
            live: true,
            selected: false,
            scroll_to_me: 0,
            id: Uuid::new_v4().to_string(),
            code: None,
            name: "new stop".to_string(),
            description: "".to_string(),
            location_type: MyLocationType(LocationType::StopPoint),
            parent_station: None,
            zone_id: None,
            url: None,
            longitude: Some(latlong.x),
            latitude: Some(latlong.y),
            timezone: None,
            wheelchair_boarding: MyAvailability(Availability::Available),
            level_id: None,
            platform_code: None,
            transfers: 0,
            pathways: 0,
            // pub stop: Option<Rc<Stop>>,
            stop: None,
            latlong,
        }
    }
}
impl ListItem for MyStop {
    fn new_child(&mut self) -> String {
        todo!()
    }
    fn update_all(&mut self, value: bool) {
        self.selected = value;
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn item_type(&self) -> String {
        "stop".to_string()
    }

    fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Data, Debug, Lens, Serialize, Deserialize)]
pub struct MyStopTime {
    pub live: bool,
    pub selected: bool,
    pub show_editing: bool,
    pub hovered: bool,
    pub edited: bool,

    pub trip_id: String,
    // pub arrival_time: Option<u32>,
    pub arrival_time: Option<String>,
    // pub departure_time: Option<u32>,
    pub departure_time: Option<String>,
    pub stop_id: String,
    pub stop_sequence: u16,
    pub stop_headsign: Option<String>,
    pub pickup_type: MyPickupDropOffType,
    pub drop_off_type: MyPickupDropOffType,
    pub continuous_pickup: MyContinuousPickupDropOff,
    pub continuous_drop_off: MyContinuousPickupDropOff,
    pub shape_dist_traveled: Option<f32>,
    pub timepoint: MyTimepointType,

    #[data(ignore)]
    #[lens(ignore)]
    pub stop_time: Option<Rc<RawStopTime>>,
    #[data(ignore)]
    #[lens(ignore)]
    pub stop: Option<Rc<Stop>>,
    // stop_time: RawStopTime,
    pub stop_name: String,
    // (lon, lat)
    #[serde(skip)]
    pub latlong: Point,
}
impl MyStopTime {
    pub fn new(trip_id: String, stop_id: String, stop_sequence: u16) -> MyStopTime {
        MyStopTime {
            live: true,
            selected: false,
            show_editing: false,
            hovered: false,
            edited: true,

            trip_id,
            arrival_time: None,
            departure_time: None,
            stop_id,
            stop_sequence,
            stop_headsign: None,
            pickup_type: MyPickupDropOffType(PickupDropOffType::Regular),
            drop_off_type: MyPickupDropOffType(PickupDropOffType::Regular),
            continuous_pickup: MyContinuousPickupDropOff(ContinuousPickupDropOff::Continuous),
            continuous_drop_off: MyContinuousPickupDropOff(ContinuousPickupDropOff::Continuous),
            shape_dist_traveled: None,
            timepoint: MyTimepointType(TimepointType::Approximate),

            //  stop_time: Option<Rc<RawStopTime>>,
            //  stop: Option<Rc<Stop>>,
            stop_time: None,
            stop: None,
            stop_name: "stop name".to_string(),
            // (lon, lat)
            latlong: Point::ORIGIN,
        }
    }
}
impl ListItem for MyStopTime {
    fn new_child(&mut self) -> String {
        todo!()
    }
    fn update_all(&mut self, value: bool) {
        self.selected = value;
    }
    fn id(&self) -> String {
        self.stop_id.clone()
    }
    fn item_type(&self) -> String {
        "stop_time".to_string()
    }
    fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Data, Debug, Lens, Serialize, Deserialize)]
pub struct MyTrip {
    pub live: bool,
    pub visible: bool,
    pub selected: bool,
    pub expanded: bool,
    pub show_editing: bool,
    pub edited: bool,

    pub id: String,
    pub service_id: String,
    pub route_id: String,
    pub shape_id: Option<String>,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<MyDirectionType>,
    pub block_id: Option<String>,
    pub wheelchair_accessible: MyAvailability,
    pub bikes_allowed: MyBikesAllowedType,
    #[data(ignore)]
    #[lens(ignore)]
    pub trip: Option<Rc<RawTrip>>,
    // #[data(ignore)]
    // trip: RawTrip,
    pub name: String,
    // pub stops: Vector<MyStopTime>,
    pub n_stops: usize,
}
impl MyTrip {
    pub fn new(route_id: String) -> Self {
        MyTrip {
            live: true,
            visible: true,
            selected: false,
            expanded: false,
            show_editing: false,
            edited: false,

            id: Uuid::new_v4().to_string(),
            service_id: "needtogetcalendar.serviceid".to_string(),
            route_id,
            shape_id: None,
            trip_headsign: None,
            trip_short_name: None,
            direction_id: None,
            block_id: None,
            wheelchair_accessible: MyAvailability(Availability::InformationNotAvailable),
            bikes_allowed: MyBikesAllowedType(BikesAllowedType::NoBikeInfo),

            trip: None,
            name: "new trip name".to_string(),
            // stops: Vector::new(),
            n_stops: 0,
        }
    }
}
impl ListItem for MyTrip {
    fn new_child(&mut self) -> String {
        todo!()
    }
    fn update_all(&mut self, value: bool) {
        // self.selected = value;
        // self.stops.iter_mut().for_each(|stop| {
        //     stop.selected = value;
        //     // stop.update_all(value);
        // });
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn n_stops(&self) -> usize {
        self.n_stops
    }
    fn show_editing(&self) -> bool {
        self.show_editing
    }
    fn item_type(&self) -> String {
        "trip".to_string()
    }
    fn selected(&self) -> bool {
        self.selected
    }
    // fn data_info(&self) -> String {
    //     format!(
    //         "{} -> {}",
    //         self.trip
    //             .trip_headsign
    //             .clone()
    //             .unwrap_or("no headsign".to_string()),
    //         self.trip_headsign
    //     )
    // }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyRouteType(pub RouteType);
impl MyRouteType {
    pub fn radio_vec() -> Vec<(String, MyRouteType)> {
        vec![
            ("Tramway".to_string(), MyRouteType(RouteType::Tramway)),
            ("Subway".to_string(), MyRouteType(RouteType::Subway)),
            ("Rail".to_string(), MyRouteType(RouteType::Rail)),
            ("Bus".to_string(), MyRouteType(RouteType::Bus)),
            ("Ferry".to_string(), MyRouteType(RouteType::Ferry)),
            ("CableCar".to_string(), MyRouteType(RouteType::CableCar)),
            ("Gondola".to_string(), MyRouteType(RouteType::Gondola)),
            ("Funicular".to_string(), MyRouteType(RouteType::Funicular)),
            ("Coach".to_string(), MyRouteType(RouteType::Coach)),
            ("Air".to_string(), MyRouteType(RouteType::Air)),
            ("Taxi".to_string(), MyRouteType(RouteType::Taxi)),
            ("Other(99)".to_string(), MyRouteType(RouteType::Other(99))),
        ]
    }
}
impl Data for MyRouteType {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct MyRoute {
    pub new_item: bool,
    pub live: bool,
    pub visible: bool,
    pub expanded: bool,
    pub selected: bool,
    pub show_editing: bool,

    pub id: String,
    pub short_name: String,
    pub long_name: String,
    pub desc: Option<String>,
    pub route_type: MyRouteType,
    pub url: Option<String>,
    pub agency_id: Option<String>,
    pub order: Option<u32>,
    pub color: MyRGB8,
    pub text_color: MyRGB8,
    pub continuous_pickup: MyContinuousPickupDropOff,
    pub continuous_drop_off: MyContinuousPickupDropOff,

    #[lens(ignore)]
    #[data(ignore)]
    pub route: Option<Rc<Route>>,
    // pub trips: Vector<MyTrip>,
    pub n_trips: usize,
}
impl MyRoute {
    pub fn new(agency_id: Option<String>) -> MyRoute {
        MyRoute {
            new_item: true,
            live: true,
            visible: true,
            expanded: false,
            selected: false,
            show_editing: false,

            id: Uuid::new_v4().to_string(),
            short_name: "Short name".to_string(),
            long_name: "Long name".to_string(),
            desc: None,
            route_type: MyRouteType(RouteType::Bus),
            url: None,
            agency_id,
            order: None,
            color: MyRGB8(RGB8::new(255, 255, 255)),
            text_color: MyRGB8(RGB8::new(0, 0, 0)),
            continuous_pickup: MyContinuousPickupDropOff(ContinuousPickupDropOff::Continuous),
            continuous_drop_off: MyContinuousPickupDropOff(ContinuousPickupDropOff::Continuous),
            route: None,
            n_trips: 0,
        }
    }
}
impl ListItem for MyRoute {
    fn new_child(&mut self) -> String {
        let new_trip = MyTrip {
            live: true,
            visible: true,
            selected: false,
            expanded: false,
            show_editing: false,
            edited: false,

            id: Uuid::new_v4().to_string(),
            service_id: "needtogetcalendar.serviceid".to_string(),
            route_id: self.id(),
            shape_id: None,
            trip_headsign: None,
            trip_short_name: None,
            direction_id: None,
            block_id: None,
            wheelchair_accessible: MyAvailability(Availability::InformationNotAvailable),
            bikes_allowed: MyBikesAllowedType(BikesAllowedType::NoBikeInfo),

            trip: None,
            name: "new trip name".to_string(),
            // stops: Vector::new(),
            n_stops: 0,
        };
        let new_trip_id = new_trip.id();
        // self.trips.push_front(new_trip);
        println!("added new trip");
        new_trip_id
    }
    fn update_all(&mut self, value: bool) {
        // self.selected = value;
        // self.trips.iter_mut().for_each(|trip| {
        //     trip.visible = value;
        //     trip.update_all(value);
        // });
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn item_type(&self) -> String {
        "route".to_string()
    }
    fn data_info(&self) -> String {
        format!(
            "{} -> {}",
            self.route.as_ref().unwrap().short_name.clone(),
            self.short_name
        )
    }
    fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Data, Default, Lens, Serialize, Deserialize)]
pub struct MyAgency {
    pub show_deleted: bool,
    pub live: bool,
    pub visible: bool,
    pub expanded: bool,
    pub selected: bool,
    pub show_editing: bool,

    pub id: Option<String>,
    pub name: String,
    pub url: String,
    pub timezone: String,
    pub lang: Option<String>,
    pub phone: Option<String>,
    pub fare_url: Option<String>,
    pub email: Option<String>,

    #[lens(ignore)]
    #[data(ignore)]
    pub agency: Option<Rc<Agency>>,
    pub n_stops: usize,
}
impl ListItem for MyAgency {
    fn new_child(&mut self) -> String {
        let new_route = MyRoute {
            new_item: true,
            live: true,
            visible: true,
            expanded: false,
            selected: false,
            show_editing: false,
            id: Uuid::new_v4().to_string(),

            short_name: "new route short name".to_string(),
            long_name: "new route long name".to_string(),
            desc: None,
            route_type: MyRouteType(RouteType::Bus),
            url: None,
            agency_id: None,
            order: None,
            color: MyRGB8(RGB8::new(0, 0, 0)),
            text_color: MyRGB8(RGB8::new(0, 0, 0)),
            continuous_pickup: MyContinuousPickupDropOff(ContinuousPickupDropOff::NotAvailable),
            continuous_drop_off: MyContinuousPickupDropOff(ContinuousPickupDropOff::NotAvailable),

            route: None,
            // trips: Vector::new(),
            n_trips: 0,
        };
        let new_route_id = new_route.id();
        // self.routes.push_front(new_route);
        println!("added new route");
        new_route_id
    }
    fn update_all(&mut self, value: bool) {
        // self.routes.iter_mut().for_each(|route| {
        //     route.visible = value;
        //     route.update_all(value);
        // });
    }
    fn id(&self) -> String {
        // todo handle agency.id == None
        self.id.as_ref().unwrap().clone()
    }
    fn item_type(&self) -> String {
        "agency".to_string()
    }
    fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Data, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditType {
    Delete,
    Update,
    Create,
}
#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct Action {
    pub id: usize,
    pub edit_type: EditType,
    pub item_type: String,
    pub item_id: String,
    // todo this of course means that the edit list won't get updated when eg a field name changes
    // #[data(ignore)]

    // not sure how to derive Serialize for trait object
    // #[lens(ignore)]
    // pub item_data: Option<Rc<dyn ListItem>>,
}
impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Edit")
            .field("id", &self.id)
            .field("edit_type", &self.edit_type)
            .field("item_type", &self.item_type)
            .field("item_id", &self.item_id)
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MyGtfs {
    pub agencies: Vec<Agency>,
    pub routes: Vec<Route>,
    pub trips: Vec<RawTrip>,
    pub stop_times: Vec<RawStopTime>,
    pub stops: Vec<Stop>,
    pub shapes: Vec<Shape>,
}

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct Edit {
    id: usize,
}

#[derive(Debug, Copy, Clone, Data, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ZoomLevel {
    One,
    Two,
    Five,
    Ten,
    Twenty,
    Fifty,
}
impl ZoomLevel {
    pub fn radio_group_vec() -> Vec<(String, ZoomLevel)> {
        vec![
            ("1x".to_string(), ZoomLevel::One),
            ("2x".to_string(), ZoomLevel::Two),
            ("5x".to_string(), ZoomLevel::Five),
            ("10x".to_string(), ZoomLevel::Ten),
            ("20x".to_string(), ZoomLevel::Twenty),
            ("50x".to_string(), ZoomLevel::Fifty),
        ]
    }
    pub fn to_usize(&self) -> usize {
        match self {
            ZoomLevel::One => 1,
            ZoomLevel::Two => 2,
            ZoomLevel::Five => 5,
            ZoomLevel::Ten => 10,
            ZoomLevel::Twenty => 20,
            ZoomLevel::Fifty => 50,
        }
    }
    pub fn to_f64(&self) -> f64 {
        self.to_usize() as f64
    }
    /// canvas_size / (zoom_f64 * 300.)
    pub fn path_width(&self, canvas_size: f64) -> f64 {
        // canvas_size / (self.to_f64() * 300.)
        canvas_size / (self.to_f64() * 300.)
    }
}

// #[derive(Clone, Data, Lens)]
#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct AppData {
    /// if true insert before else after
    pub insert_stop_time_before: Option<bool>,

    pub show_deleted: bool,
    pub show_edits: bool,
    pub show_actions: bool,
    #[data(ignore)]
    #[lens(ignore)]
    pub gtfs: Rc<MyGtfs>,
    // #[data(ignore)]
    // #[lens(ignore)]
    // pub trips_other2: Vec<RawTrip>,
    // #[data(ignore)]
    // #[lens(ignore)]
    // pub stop_times_other2: Vec<RawStopTime>,
    // #[data(ignore)]
    // #[lens(ignore)]
    // pub stops_other2: Vec<Stop>,
    // #[data(ignore)]
    // #[lens(ignore)]
    // pub stop_times_other: Vec<MyStopTime>,
    pub selected_agency_id: Option<Option<String>>,
    pub selected_route_id: Option<String>,
    // (index, id)
    pub selected_trip_id: Option<(usize, String)>,
    pub selected_stop_time_id: Option<(String, u16)>,
    pub hovered_stop_time_id: Option<(String, u16)>,
    pub selected_stop_id: Option<String>,

    // pub all_trip_paths_bitmap_grouped: Vector<(Rect, Vector<usize>)>,
    #[data(ignore)]
    #[lens(ignore)]
    pub hovered_trip_paths: Vector<usize>,
    // #[data(ignore)]
    // #[lens(ignore)]
    // pub selected_trip_path: Option<usize>,
    #[data(ignore)]
    #[lens(ignore)]
    pub stop_time_range_from_trip_id: HashMap<String, (usize, usize)>,
    #[data(ignore)]
    #[lens(ignore)]
    pub stop_index_from_id: HashMap<String, usize>,
    #[data(ignore)]
    #[lens(ignore)]
    pub shapes_range_from_shape_id: HashMap<String, Range<usize>>,

    pub agencies: Vector<MyAgency>,
    pub routes: Vector<MyRoute>,
    pub trips: Vector<MyTrip>,
    pub stop_times: Vector<MyStopTime>,
    pub stops: Vector<MyStop>,
    pub expanded: bool,
    pub actions: Vector<Action>,
    pub edits: Vector<Edit>,

    pub map_zoom_level: ZoomLevel,
    pub map_stop_selection_mode: bool,
}
impl ListItem for AppData {
    fn new_child(&mut self) -> String {
        todo!()
    }
    fn update_all(&mut self, value: bool) {
        self.agencies
            .iter_mut()
            .for_each(|agency| agency.update_all(value));
    }
    fn id(&self) -> String {
        "null".to_string()
    }
    fn item_type(&self) -> String {
        "null".to_string()
    }
    fn selected(&self) -> bool {
        false
    }
}
// vector of trips (selected, vector of stop coords)
impl AppData {
    pub fn trip_coords_from_stop_coords(&self, trip_id: String) -> Vec<Point> {
        dbg!("make trip coords");
        let trip = self.trips.iter().find(|trip| trip.id == trip_id).unwrap();
        let (start_index, end_index) = self
            .stop_time_range_from_trip_id
            .get(&trip.id)
            .unwrap()
            .clone();
        let mut points = Vec::new();
        for i in start_index..end_index {
            let stop_time = self.stop_times.get(i).unwrap();
            let stop = self
                .stops
                .get(*self.stop_index_from_id.get(&stop_time.stop_id).unwrap())
                .unwrap();

            points.push(Point::new(stop.longitude.unwrap(), stop.latitude.unwrap()));
        }
        points
    }

    pub fn trips_coords_from_stop_coords(&self) -> Vec<Vec<Point>> {
        dbg!("make trips coords");
        self.trips
            .iter()
            .map(|trip| {
                let (start_index, end_index) = self
                    .stop_time_range_from_trip_id
                    .get(&trip.id)
                    .unwrap()
                    .clone();
                let mut stop_time_coords = self.gtfs.stop_times[start_index..end_index]
                    .iter()
                    .map(|stop_time| {
                        let stop = self
                            .stops
                            .get(*self.stop_index_from_id.get(&stop_time.stop_id).unwrap())
                            .unwrap();

                        // (
                        //     stop_time.stop_sequence.clone(),
                        //     (stop.longitude.unwrap(), stop.latitude.unwrap()),
                        // )

                        Point::new(stop.longitude.unwrap(), stop.latitude.unwrap())
                    })
                    .collect::<Vec<_>>();
                // stop_time_coords.sort_by(|stop_time1, stop_time2| stop_time1.0.cmp(&stop_time2.0));

                // stop_time_coords
                //     .iter()
                //     .map(|(_stop_sequence, coords)| *coords)
                //     .collect::<Vec<_>>()
                stop_time_coords
            })
            .collect::<Vec<_>>()
    }
    // TODO don't need to construct MyStopTime here
    pub fn trips_coords_from_shapes(&self) -> Vec<Vec<Point>> {
        dbg!("make trip coords");
        self.trips
            .iter()
            .map(|trip| {
                // let (start_index, end_index) = self
                //     .stop_time_range_from_trip_id
                //     .get(&trip.id)
                //     .unwrap()
                //     .clone();
                // let mut stop_time_coords = self.gtfs.stop_times[start_index..end_index]
                //     .iter()
                //     .map(|stop_time| {
                //         let stop = self
                //             .stops
                //             .get(*self.stop_index_from_id.get(&stop_time.stop_id).unwrap())
                //             .unwrap();

                //         (
                //             stop_time.stop_sequence.clone(),
                //             (stop.longitude.unwrap(), stop.latitude.unwrap()),
                //         )
                //     })
                //     .collect::<Vector<_>>();
                // stop_time_coords.sort_by(|stop_time1, stop_time2| stop_time1.0.cmp(&stop_time2.0));

                // stop_time_coords
                //     .iter()
                //     .map(|(_stop_sequence, coords)| *coords)
                //     .collect::<Vec<_>>()

                // let mut shapes = self
                //     .gtfs
                //     .shapes
                //     .iter()
                //     .filter(|shape| &shape.id == trip.shape_id.as_ref().unwrap())
                //     .map(|shape| (shape.sequence, (shape.longitude, shape.latitude)))
                //     .collect::<Vec<_>>();

                if let Some(shape_id) = &trip.shape_id {
                    let range = self.shapes_range_from_shape_id.get(shape_id).unwrap();
                    let mut shapes = self.gtfs.shapes[range.start..range.end]
                        .iter()
                        .map(|shape| (shape.sequence, Point::new(shape.longitude, shape.latitude)))
                        .collect::<Vec<_>>();
                    shapes.sort_by(|shape1, shape2| shape1.0.cmp(&shape2.0));
                    shapes
                        .iter()
                        .map(|(_sequence, coords)| *coords)
                        .collect::<Vec<_>>()
                } else {
                    // TODO handle new trips which don't have shapes
                    Vec::new()
                }
            })
            .collect::<Vec<_>>()
    }
    // TODO shouldn't do filtering here, should include flags with coords so can do filtering later
    // TODO should be recording whether agencies/routes are visible/selected here...
    pub fn flat_trips(&self) -> Vec<(bool, MyTrip)> {
        dbg!("make trip coords");
        todo!()
    }
}

pub fn make_initial_data(gtfs: &mut RawGtfs) -> AppData {
    // NOTE: must pay attention to when Vector<x> and gtfs.x are being sorted and ensure they are the same
    println!("{:?} start make_initial_data", Utc::now());
    let agencies = gtfs.agencies.as_mut().unwrap();
    let routes = gtfs.routes.as_mut().unwrap();
    let trips = gtfs.trips.as_mut().unwrap();
    let stop_times = gtfs.stop_times.as_mut().unwrap();
    let stops = gtfs.stops.as_mut().unwrap();
    let shapes = gtfs.shapes.as_mut().unwrap().as_ref().unwrap();

    println!("{:?} do stop_times stuff", Utc::now());
    // creates stop_time_range_from_trip_id which is a hashmap where each key is a trip_id pointing to the index range of it's stop times in the sorted stop_times below
    // need to be able to grab a slice of stop times by trip id to avoid doing the below loads of times:
    // stop_times
    //     .iter()
    //     .filter(|stop_time| stop_time.trip_id == trip.id);
    trips.sort_by(|x1, x2| x1.id.cmp(&x2.id));
    stop_times.sort_by(|stop1, stop2| stop1.stop_sequence.cmp(&stop2.stop_sequence));
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

    // hash map for getting a stop by stop_id
    let mut stop_map = HashMap::new();
    let stops2 = stops.clone();
    stops2.iter().for_each(|stop| {
        stop_map.insert(stop.id.clone(), stop.clone());
    });

    agencies.sort_by(|x1, x2| x1.name.cmp(&x2.name));

    // let mut filtered_stop_ids = Vec::new();

    // let stop_times_other2 = stop_times.clone();
    // let trips_other2 = trips.clone();
    // let stops_other2 = stops.clone();

    // TODO should proabably just store stops in an im hashmap not vector, since below is brittle and will break is a stop is added/removed from stops vector.
    // hash map for getting a stop by stop_id
    let mut stop_index_from_id = HashMap::new();
    // let stops2 = self.gtfs.stops.clone();
    stops.iter().enumerate().for_each(|(i, stop)| {
        stop_index_from_id.insert(stop.id.clone(), i);
    });

    let mut shapes_from_trip_id = HashMap::new();
    let mut start: usize = 0;
    let mut end: usize = 0;
    if let Some(first_item) = shapes.get(0) {
        let mut current_id = first_item.id.clone();
        // insert bound if entered new id section or is last item
        for item in shapes {
            if current_id != item.id {
                shapes_from_trip_id.insert(current_id.clone(), Range { start, end });
                current_id = item.id.clone();
                start = end;
            }
            end += 1;
        }
        shapes_from_trip_id.insert(current_id.clone(), Range { start, end });
    }

    routes.sort_by(|route1, route2| route1.short_name.cmp(&route2.short_name));

    println!("{:?} create my_gtfs", Utc::now());
    // NOTE must make my_gtfs after doing sorting otherwise indexes/mappings/lookups will not be correct
    let my_gtfs = MyGtfs {
        agencies: agencies.clone(),
        routes: routes.clone(),
        trips: trips.clone(),
        stop_times: stop_times.clone(),
        stops: stops.clone(),
        shapes: shapes.clone(),
    };

    // let limited = true;
    println!("{:?} make agencies", Utc::now());
    let agencies = agencies
        .iter()
        // <limiting
        // .enumerate()
        // .filter(|(i, _)| if limited { *i < 5 } else { true })
        // .map(|(_, x)| x)
        // limiting>
        .map(|agency| MyAgency {
            show_deleted: true,
            live: true,
            visible: true,
            expanded: false,
            selected: false,
            show_editing: false,

            id: agency.id.clone(),
            name: agency.name.clone(),
            url: agency.url.clone(),
            timezone: agency.timezone.clone(),
            lang: agency.lang.clone(),
            phone: agency.phone.clone(),
            fare_url: agency.fare_url.clone(),
            email: agency.email.clone(),

            agency: Some(Rc::new(agency.clone())),
            // routes: Vector::new(),
            n_stops: routes
                .iter()
                .filter(|route| route.agency_id == agency.id)
                .count(),
        })
        .collect::<Vector<_>>();

    // let stops = stops
    //     .iter()
    //     .filter(|stop| filtered_stop_ids.contains(&stop.id))
    //     .collect::<Vec<_>>();

    // let (start_index, end_index) = stop_time_range_from_trip_id.get(&trip.id).unwrap().clone();
    // let mut stops = stop_times[start_index..end_index]
    println!("{:?} make stop_times", Utc::now());
    let mut stop_times = stop_times
        .iter()
        // .filter(|stop_time| stop_time.trip_id == trip.id)
        .map(|stop_time| {
            let stop = stop_map.get(&stop_time.stop_id).unwrap();
            // filtered_stop_ids.push(stop.id.clone());
            MyStopTime {
                live: true,
                selected: false,
                show_editing: false,
                hovered: false,
                edited: false,

                trip_id: stop_time.trip_id.clone(),
                arrival_time: stop_time.arrival_time.clone(),
                departure_time: stop_time.departure_time.clone(),
                stop_id: stop_time.stop_id.clone(),
                stop_sequence: stop_time.stop_sequence.clone(),
                stop_headsign: stop_time.stop_headsign.clone(),
                pickup_type: MyPickupDropOffType(stop_time.pickup_type.clone()),
                drop_off_type: MyPickupDropOffType(stop_time.drop_off_type.clone()),
                continuous_pickup: MyContinuousPickupDropOff(stop_time.continuous_pickup.clone()),
                continuous_drop_off: MyContinuousPickupDropOff(
                    stop_time.continuous_drop_off.clone(),
                ),
                shape_dist_traveled: stop_time.shape_dist_traveled.clone(),
                timepoint: MyTimepointType(stop_time.timepoint.clone()),

                // stop_time: Some(Rc::new(stop_time.clone())),
                // stop: Some(Rc::new(
                //     stops
                //         .iter()
                //         .find(|stop| stop.id == stop_time.stop_id)
                //         .unwrap()
                //         .clone(),
                // )),
                stop_time: Some(Rc::new(stop_time.clone())),
                stop: None,
                // stop_time: stop_time.clone(),
                stop_name: stop.name.clone(),
                latlong: Point::new(stop.longitude.unwrap(), stop.latitude.unwrap()),
            }
        })
        .collect::<Vector<_>>();

    println!("{:?} make trips", Utc::now());
    let trips = trips
        .iter()
        .enumerate()
        // <limiting
        // .enumerate()
        // .filter(|(i, _)| if limited { *i < 5 } else { true })
        // .map(|(_, x)| x)
        // limiting>
        .map(|(i, trip)| {
            // adding the RawTrip to MyTrip is the tipping point which kills performance. Maybe AppData should just be storing a u32 index of the items position in the original RawGtfs data
            MyTrip {
                live: true,
                visible: true,
                selected: false,
                expanded: false,
                show_editing: false,
                edited: false,

                id: trip.id.clone(),
                service_id: trip.service_id.clone(),
                route_id: trip.route_id.clone(),
                shape_id: trip.shape_id.clone(),
                trip_headsign: trip.trip_headsign.clone(),
                trip_short_name: trip.trip_short_name.clone(),
                direction_id: trip.direction_id.map(|x| MyDirectionType(x)),
                block_id: trip.block_id.clone(),
                wheelchair_accessible: MyAvailability(trip.wheelchair_accessible),
                bikes_allowed: MyBikesAllowedType(trip.bikes_allowed),

                trip: Some(Rc::new(trip.clone())),
                name: trip.id.clone(),

                // stops: Vector::new(),
                // n_stops: stop_times
                //     .iter()
                //     .filter(|stop_time| stop_time.trip_id == trip.id)
                //     .count(),
                n_stops: {
                    let (start, end) = stop_time_range_from_trip_id.get(&trip.id).unwrap();
                    end - start
                },
                // n_stops: 99,
            }
        })
        .collect::<Vector<_>>();

    println!("{:?} make routes", Utc::now());
    let mut routes = routes
        .iter()
        // <limiting
        .enumerate()
        // .filter(|(i, _)| if limited { *i < 40 } else { true })
        .map(|(_, x)| x)
        // limiting>
        .map(|route| MyRoute {
            new_item: false,
            live: true,
            visible: true,
            expanded: false,
            selected: false,
            show_editing: false,

            id: route.id.clone(),
            short_name: route.short_name.clone(),
            long_name: route.long_name.clone(),
            desc: route.desc.clone(),
            route_type: MyRouteType(route.route_type.clone()),
            url: route.url.clone(),
            agency_id: route.agency_id.clone(),
            order: route.order.clone(),
            color: MyRGB8(route.color.clone()),
            text_color: MyRGB8(route.text_color.clone()),
            continuous_pickup: MyContinuousPickupDropOff(route.continuous_pickup.clone()),
            continuous_drop_off: MyContinuousPickupDropOff(route.continuous_drop_off.clone()),

            route: Some(Rc::new(route.clone())),
            // trips: Vector::new(),
            n_trips: trips
                .iter()
                .filter(|trip| trip.route_id == route.id)
                .count(),
        })
        .collect::<Vector<_>>();

    println!("{:?} make app_data with stops", Utc::now());
    let app_data = AppData {
        insert_stop_time_before: None,

        show_deleted: true,
        show_edits: false,
        show_actions: false,
        gtfs: Rc::new(my_gtfs),
        // trips_other2: Vec::new(),
        // stop_times_other2: Vec::new(),
        // stops_other2: Vec::new(),
        // stop_times_other: Vec::new(),
        selected_agency_id: None,
        selected_route_id: None,
        selected_trip_id: None,
        selected_stop_time_id: None,
        hovered_stop_time_id: None,
        selected_stop_id: None,
        expanded: true,

        // all_trip_paths_bitmap_grouped: Vector::new(),
        hovered_trip_paths: Vector::new(),
        // selected_trip_path: None,
        stop_time_range_from_trip_id,
        stop_index_from_id,
        shapes_range_from_shape_id: shapes_from_trip_id,

        agencies,
        routes,
        trips,
        stop_times,
        stops: stops
            .iter()
            .map(|stop| MyStop {
                live: true,
                selected: false,
                scroll_to_me: 0,

                id: stop.id.clone(),
                code: stop.code.clone(),
                name: stop.name.clone(),
                description: stop.description.clone(),
                location_type: MyLocationType(stop.location_type.clone()),
                parent_station: stop.parent_station.clone(),
                zone_id: stop.zone_id.clone(),
                url: stop.url.clone(),
                longitude: stop.longitude.clone(),
                latitude: stop.latitude.clone(),
                timezone: stop.timezone.clone(),
                wheelchair_boarding: MyAvailability(stop.wheelchair_boarding.clone()),
                level_id: stop.level_id.clone(),
                platform_code: stop.platform_code.clone(),
                transfers: stop.transfers.len(),
                pathways: stop.pathways.len(),

                // this adds 4 seconds
                // stop: Some(Rc::new(stop.clone())),
                stop: None,
                latlong: Point::new(stop.longitude.unwrap(), stop.latitude.unwrap()),
            })
            .collect::<Vector<_>>(),
        // stops: Vector::new(),
        actions: Vector::new(),
        edits: Vector::new(),
        map_zoom_level: ZoomLevel::One,
        // map_zoom_level: ZoomLevel::Two,
        map_stop_selection_mode: false,
    };
    println!("{:?} finish make_initial_data", Utc::now());
    app_data
}
