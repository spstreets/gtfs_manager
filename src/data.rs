use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::{Data, Lens, Widget, WidgetExt};
use gtfs_structures::{
    Agency, Availability, BikesAllowedType, ContinuousPickupDropOff, DirectionType, Gtfs, Pathway,
    PickupDropOffType, RawGtfs, RawStopTime, RawTrip, Route, RouteType, Stop, StopTime,
    StopTransfer, TimepointType, Trip,
};
use rgb::RGB8;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use uuid::Uuid;

mod newtypes;
pub use newtypes::*;

pub trait ListItem {
    fn new_child(&mut self) -> String;
    fn update_all(&mut self, value: bool);
    fn id(&self) -> String;
    fn item_type(&self) -> String;
    // fn data_info(&self) -> String;
    fn data_info(&self) -> String {
        "not implemented".to_string()
    }
    // fn name(&self) -> String;
}

#[derive(Clone, Data, Debug, Lens)]
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
    pub coord: (f64, f64),
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
}

#[derive(Clone, Data, Debug, Lens)]
pub struct MyStopTime {
    pub live: bool,
    pub selected: bool,

    pub trip_id: String,
    pub arrival_time: Option<u32>,
    pub departure_time: Option<u32>,
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
    // stop_time: RawStopTime,
    pub name: String,
    pub coord: (f64, f64),
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
}

#[derive(Clone, Data, Debug, Lens)]
pub struct MyTrip {
    pub live: bool,
    pub selected: bool,
    pub expanded: bool,

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
    pub stops: Vector<MyStopTime>,
}
impl ListItem for MyTrip {
    fn new_child(&mut self) -> String {
        todo!()
    }
    fn update_all(&mut self, value: bool) {
        // self.selected = value;
        self.stops.iter_mut().for_each(|stop| {
            stop.selected = value;
            // stop.update_all(value);
        });
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn item_type(&self) -> String {
        "trip".to_string()
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MyRGB8(pub RGB8);
impl Data for MyRGB8 {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MyContinuousPickupDropOff(pub ContinuousPickupDropOff);
impl MyContinuousPickupDropOff {
    pub fn radio_vec() -> Vec<(String, MyContinuousPickupDropOff)> {
        vec![
            (
                "Continuous".to_string(),
                MyContinuousPickupDropOff(ContinuousPickupDropOff::Continuous),
            ),
            (
                "NotAvailable".to_string(),
                MyContinuousPickupDropOff(ContinuousPickupDropOff::NotAvailable),
            ),
            (
                "ArrangeByPhone".to_string(),
                MyContinuousPickupDropOff(ContinuousPickupDropOff::ArrangeByPhone),
            ),
            (
                "CoordinateWithDriver".to_string(),
                MyContinuousPickupDropOff(ContinuousPickupDropOff::CoordinateWithDriver),
            ),
            (
                "Unknown(99)".to_string(),
                MyContinuousPickupDropOff(ContinuousPickupDropOff::Unknown(99)),
            ),
        ]
    }
}
impl Data for MyContinuousPickupDropOff {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Data, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Fruit {
    Apple,
    Pear,
    Orange,
}
impl Fruit {
    pub fn radio_vec() -> Vec<(String, Fruit)> {
        vec![
            ("Apple".to_string(), Fruit::Apple),
            ("Pear".to_string(), Fruit::Pear),
            ("Orange".to_string(), Fruit::Orange),
        ]
    }
}

#[derive(Clone, Data, Lens)]
pub struct MyRoute {
    pub new: bool,
    pub live: bool,
    pub selected: bool,
    pub expanded: bool,

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
    pub trips: Vector<MyTrip>,
}
impl ListItem for MyRoute {
    fn new_child(&mut self) -> String {
        let new_trip = MyTrip {
            live: true,
            selected: true,
            expanded: false,

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
            stops: Vector::new(),
        };
        let new_trip_id = new_trip.id();
        self.trips.push_front(new_trip);
        println!("added new trip");
        new_trip_id
    }
    fn update_all(&mut self, value: bool) {
        // self.selected = value;
        self.trips.iter_mut().for_each(|trip| {
            trip.selected = value;
            trip.update_all(value);
        });
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
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyAgency {
    pub show_deleted: bool,
    pub live: bool,
    pub selected: bool,
    pub expanded: bool,

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
    pub routes: Vector<MyRoute>,
}
impl ListItem for MyAgency {
    fn new_child(&mut self) -> String {
        let new_route = MyRoute {
            new: true,
            live: true,
            selected: true,
            expanded: false,
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
            trips: Vector::new(),
        };
        let new_route_id = new_route.id();
        self.routes.push_front(new_route);
        println!("added new route");
        new_route_id
    }
    fn update_all(&mut self, value: bool) {
        self.routes.iter_mut().for_each(|route| {
            route.selected = value;
            route.update_all(value);
        });
    }
    fn id(&self) -> String {
        // todo handle agency.id == None
        self.id.as_ref().unwrap().clone()
    }
    fn item_type(&self) -> String {
        "agency".to_string()
    }
}

#[derive(Clone, Data, Debug, PartialEq)]
pub enum EditType {
    Delete,
    Update,
    Create,
}
#[derive(Clone, Data, Lens)]
pub struct Action {
    pub id: usize,
    pub edit_type: EditType,
    pub item_type: String,
    pub item_id: String,
    // todo this of course means that the edit list won't get updated when eg a field name changes
    // #[data(ignore)]
    #[lens(ignore)]
    pub item_data: Option<Rc<dyn ListItem>>,
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

pub struct MyGtfs {
    pub agencies: Vec<Agency>,
    pub routes: Vec<Route>,
    pub trips: Vec<RawTrip>,
    pub stop_times: Vec<RawStopTime>,
    pub stops: Vec<Stop>,
}

#[derive(Clone, Data, Lens)]
pub struct Edit {
    id: usize,
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub show_deleted: bool,
    pub show_edits: bool,
    pub show_actions: bool,
    #[data(ignore)]
    #[lens(ignore)]
    pub gtfs: Rc<MyGtfs>,
    pub agencies: Vector<MyAgency>,
    pub stops: Vector<MyStop>,
    pub expanded: bool,
    pub actions: Vector<Action>,
    pub edits: Vector<Edit>,
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
}

pub fn make_initial_data(gtfs: RawGtfs) -> AppData {
    let mut agencies = gtfs.agencies.unwrap();
    let mut routes = gtfs.routes.unwrap();
    let mut trips = gtfs.trips.unwrap();
    let mut stop_times = gtfs.stop_times.unwrap();
    let mut stops = gtfs.stops.unwrap();

    let my_gtfs = MyGtfs {
        agencies: agencies.clone(),
        routes: routes.clone(),
        trips: trips.clone(),
        stop_times: stop_times.clone(),
        stops: stops.clone(),
    };

    // creates stop_time_range_from_trip_id which is a hashmap where each key is a trip_id pointing to the index range of it's stop times in the sorted stop_times below
    // need to be able to grab a slice of stop times by trip id to avoid doing the below loads of times:
    // stop_times
    //     .iter()
    //     .filter(|stop_time| stop_time.trip_id == trip.id);
    trips.sort_by(|x1, x2| x1.id.cmp(&x2.id));
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
    let mut filtered_stop_ids = Vec::new();

    let limited = true;
    let agencies = agencies
        .iter()
        // <limiting
        .enumerate()
        .filter(|(i, _)| if limited { *i < 5 } else { true })
        .map(|(_, x)| x)
        // limiting>
        .map(|agency| {
            let mut routes = routes
                .iter()
                .filter(|route| route.agency_id == agency.id)
                // <limiting
                .enumerate()
                .filter(|(i, _)| if limited { *i < 5 } else { true })
                .map(|(_, x)| x)
                // limiting>
                .map(|route| MyRoute {
                    new: false,
                    live: true,
                    selected: true,
                    expanded: false,

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
                    continuous_drop_off: MyContinuousPickupDropOff(
                        route.continuous_drop_off.clone(),
                    ),

                    route: Some(Rc::new(route.clone())),
                    trips: trips
                        .iter()
                        .enumerate()
                        .filter(|(i, trip)| trip.route_id == route.id)
                        // <limiting
                        .enumerate()
                        .filter(|(i, _)| if limited { *i < 5 } else { true })
                        .map(|(_, x)| x)
                        // limiting>
                        .map(|(i, trip)| {
                            let (start_index, end_index) =
                                stop_time_range_from_trip_id.get(&trip.id).unwrap().clone();
                            let mut stops = stop_times[start_index..end_index]
                                .iter()
                                // .filter(|stop_time| stop_time.trip_id == trip.id)
                                .map(|stop_time| {
                                    let stop = stop_map.get(&stop_time.stop_id).unwrap();
                                    filtered_stop_ids.push(stop.id.clone());
                                    MyStopTime {
                                        live: true,
                                        selected: true,

                                        trip_id: stop_time.trip_id.clone(),
                                        arrival_time: stop_time.arrival_time.clone(),
                                        departure_time: stop_time.departure_time.clone(),
                                        stop_id: stop_time.stop_id.clone(),
                                        stop_sequence: stop_time.stop_sequence.clone(),
                                        stop_headsign: stop_time.stop_headsign.clone(),
                                        pickup_type: MyPickupDropOffType(
                                            stop_time.pickup_type.clone(),
                                        ),
                                        drop_off_type: MyPickupDropOffType(
                                            stop_time.drop_off_type.clone(),
                                        ),
                                        continuous_pickup: MyContinuousPickupDropOff(
                                            stop_time.continuous_pickup.clone(),
                                        ),
                                        continuous_drop_off: MyContinuousPickupDropOff(
                                            stop_time.continuous_drop_off.clone(),
                                        ),
                                        shape_dist_traveled: stop_time.shape_dist_traveled.clone(),
                                        timepoint: MyTimepointType(stop_time.timepoint.clone()),

                                        stop_time: Some(Rc::new(stop_time.clone())),
                                        // stop_time: stop_time.clone(),
                                        name: stop.name.clone(),
                                        coord: (stop.longitude.unwrap(), stop.latitude.unwrap()),
                                    }
                                })
                                .collect::<Vector<_>>();
                            stops.sort_by(|stop1, stop2| {
                                stop1.stop_sequence.cmp(&stop2.stop_sequence)
                            });

                            // adding the RawTrip to MyTrip is the tipping point which kills performance. Maybe AppData should just be storing a u32 index of the items position in the original RawGtfs data
                            MyTrip {
                                live: true,
                                selected: true,
                                expanded: false,

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
                                stops,
                            }
                        })
                        .collect::<Vector<_>>(),
                })
                .collect::<Vector<_>>();
            routes.sort_by(|route1, route2| {
                route1
                    .route
                    .as_ref()
                    .unwrap()
                    .short_name
                    .cmp(&route2.route.as_ref().unwrap().short_name)
            });
            MyAgency {
                show_deleted: true,
                live: true,
                selected: true,
                expanded: false,

                id: agency.id.clone(),
                name: agency.name.clone(),
                url: agency.url.clone(),
                timezone: agency.timezone.clone(),
                lang: agency.lang.clone(),
                phone: agency.phone.clone(),
                fare_url: agency.fare_url.clone(),
                email: agency.email.clone(),

                agency: Some(Rc::new(agency.clone())),
                routes,
            }
        })
        .collect::<Vector<_>>();

    // let stops = stops
    //     .iter()
    //     .filter(|stop| filtered_stop_ids.contains(&stop.id))
    //     .collect::<Vec<_>>();

    let app_data = AppData {
        show_deleted: true,
        show_edits: false,
        show_actions: false,
        gtfs: Rc::new(my_gtfs),
        expanded: true,
        agencies,
        stops: stops
            .iter()
            // .enumerate()
            // .filter(|(i, _)| if limited { *i < 10 } else { true })
            // .map(|(_, x)| x)
            .filter(|stop| filtered_stop_ids.contains(&stop.id))
            .map(|stop| MyStop {
                live: true,
                selected: true,
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

                stop: Some(Rc::new(stop.clone())),
                coord: (stop.longitude.unwrap(), stop.latitude.unwrap()),
            })
            .collect::<Vector<_>>(),
        actions: Vector::new(),
        edits: Vector::new(),
    };
    app_data
}
