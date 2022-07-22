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
    fn new_child(&mut self) -> Self;
}

#[derive(Clone, Data, Debug, Lens)]
pub struct MyStop {
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
    pub transfers: usize,
    pub pathways: usize,
}

impl ListItem for MyStop {
    fn new_child(&mut self) -> Self {
        todo!()
    }
}

#[derive(Clone, Data, Debug, Lens)]
pub struct MyStopTime {
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

#[derive(Clone, Data, Debug, Lens)]
pub struct MyTrip {
    pub live: bool,
    pub visible: bool,
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
    pub n_stops: usize,
}
impl MyTrip {
    pub fn new(route_id: String) -> Self {
        MyTrip {
            live: true,
            visible: true,
            selected: false,
            expanded: false,

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
            stops: Vector::new(),
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

#[derive(Clone, Data, Lens)]
pub struct MyRoute {
    pub new: bool,
    pub live: bool,
    pub visible: bool,
    pub expanded: bool,
    pub selected: bool,

    #[lens(ignore)]
    #[data(ignore)]
    pub id: String,
    #[lens(ignore)]
    #[data(ignore)]
    pub short_name: String,
    #[lens(ignore)]
    #[data(ignore)]
    pub long_name: String,
    #[lens(ignore)]
    #[data(ignore)]
    pub desc: Option<String>,
    #[lens(ignore)]
    #[data(ignore)]
    pub route_type: MyRouteType,
    #[lens(ignore)]
    #[data(ignore)]
    pub url: Option<String>,
    #[lens(ignore)]
    #[data(ignore)]
    pub agency_id: Option<String>,
    #[lens(ignore)]
    #[data(ignore)]
    pub order: Option<u32>,
    #[lens(ignore)]
    #[data(ignore)]
    pub color: MyRGB8,
    #[lens(ignore)]
    #[data(ignore)]
    pub text_color: MyRGB8,
    #[lens(ignore)]
    #[data(ignore)]
    pub continuous_pickup: MyContinuousPickupDropOff,
    #[lens(ignore)]
    #[data(ignore)]
    pub continuous_drop_off: MyContinuousPickupDropOff,

    #[lens(ignore)]
    #[data(ignore)]
    pub route: Option<Rc<Route>>,
    pub trips: Vector<MyTrip>,
    pub n_stops: usize,
}
impl ListItem for MyRoute {
    fn new_child(&mut self) -> String {
        let new_trip = MyTrip {
            live: true,
            visible: true,
            selected: false,
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
            n_stops: 0,
        };
        let new_trip_id = new_trip.id();
        self.trips.push_front(new_trip);
        println!("added new trip");
        new_trip_id
    }
    fn update_all(&mut self, value: bool) {
        // self.selected = value;
        self.trips.iter_mut().for_each(|trip| {
            trip.visible = value;
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
    fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyAgency {
    pub show_deleted: bool,
    pub live: bool,
    pub visible: bool,
    pub expanded: bool,
    pub selected: bool,

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
    pub n_stops: usize,
}
impl ListItem for MyAgency {
    fn new_child(&mut self) -> String {
        let new_route = MyRoute {
            new: true,
            live: true,
            visible: true,
            expanded: false,
            selected: false,
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
            n_stops: 0,
        };
        let new_route_id = new_route.id();
        self.routes.push_front(new_route);
        println!("added new route");
        new_route_id
    }
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub agencies: Vector<MyAgency>,
    pub routes: Vector<MyRoute>,
    pub trips: Vector<MyTrip>,
}

pub fn make_initial_data(gtfs: RawGtfs) -> AppData {
    let mut agencies = gtfs.agencies.unwrap();
    let mut routes = gtfs.routes.unwrap();
    let mut trips = gtfs.trips.unwrap();
    let mut stop_times = gtfs.stop_times.unwrap();
    let mut stops = gtfs.stops.unwrap();

    let agencies = agencies
        .iter()
        .map(|agency| MyAgency {
            show_deleted: true,
            live: true,
            visible: true,
            expanded: false,
            selected: false,

            id: agency.id.clone(),
            name: agency.name.clone(),
            url: agency.url.clone(),
            timezone: agency.timezone.clone(),
            lang: agency.lang.clone(),
            phone: agency.phone.clone(),
            fare_url: agency.fare_url.clone(),
            email: agency.email.clone(),

            agency: Some(Rc::new(agency.clone())),
            routes: Vector::new(),
            n_stops: routes
                .iter()
                .filter(|route| route.agency_id == agency.id)
                .count(),
        })
        .collect::<Vector<_>>();

    let trips = trips
        .iter()
        .enumerate()
        .map(|(i, trip)| MyTrip {
            live: true,
            visible: true,
            selected: false,
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

            stops: Vector::new(),
            n_stops: stop_times
                .iter()
                .filter(|stop_time| stop_time.trip_id == trip.id)
                .count(),
        })
        .collect::<Vector<_>>();

    let mut routes = routes
        .iter()
        .map(|route| MyRoute {
            new: false,
            live: true,
            visible: true,
            expanded: false,
            selected: false,

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
            trips: Vector::new(),
            n_stops: trips
                .iter()
                .filter(|trip| trip.route_id == route.id)
                .count(),
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

    let app_data = AppData {
        show_deleted: true,
        show_edits: false,
        show_actions: false,
        gtfs: Rc::new(my_gtfs),
        trips_other2,
        stop_times_other2,
        stops_other2,
        stop_times_other: stop_times,

        selected_agency: None,
        selected_route: None,
        selected_trip: None,
        selected_stop_time: None,
        expanded: true,
        agencies,
        routes,
        trips,
        stop_times: Vector::new(),
        stops: stops
            .iter()
            // .enumerate()
            // .filter(|(i, _)| if limited { *i < 10 } else { true })
            // .map(|(_, x)| x)
            .filter(|stop| {
                if limited {
                    filtered_stop_ids.contains(&stop.id)
                } else {
                    true
                }
            })
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

                stop: Some(Rc::new(stop.clone())),
                coord: (stop.longitude.unwrap(), stop.latitude.unwrap()),
            })
            .collect::<Vector<_>>(),
        // stops: Vector::new(),
        actions: Vector::new(),
        edits: Vector::new(),
    };
    app_data
}
