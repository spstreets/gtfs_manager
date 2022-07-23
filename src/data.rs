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
use std::sync::Arc;
use uuid::Uuid;

mod newtypes;
pub use newtypes::*;

// #[derive(Clone, Data, Debug, Lens)]
// pub struct MyStop {
//     pub id: String,
//     pub code: Option<String>,
//     pub name: String,
//     pub description: String,
//     pub location_type: MyLocationType,
//     pub parent_station: Option<String>,
//     pub zone_id: Option<String>,
//     pub url: Option<String>,
//     pub longitude: Option<f64>,
//     pub latitude: Option<f64>,
//     pub timezone: Option<String>,
//     pub wheelchair_boarding: MyAvailability,
//     pub level_id: Option<String>,
//     pub platform_code: Option<String>,
//     pub transfers: usize,
//     pub pathways: usize,
// }

#[derive(Clone, Data, Debug, Lens)]
pub struct MyStopTime {
    pub selected: bool,

    pub trip_id: Arc<String>,
    pub arrival_time: Option<u32>,
    pub departure_time: Option<u32>,
    pub stop_id: Arc<String>,
    pub stop_sequence: u16,
    pub stop_headsign: Option<Arc<String>>,
    pub pickup_type: MyPickupDropOffType,
    pub drop_off_type: MyPickupDropOffType,
    pub continuous_pickup: MyContinuousPickupDropOff,
    pub continuous_drop_off: MyContinuousPickupDropOff,
    pub shape_dist_traveled: Option<f32>,
    pub timepoint: MyTimepointType,
}

#[derive(Clone, Data, Debug, Lens)]
pub struct MyTrip {
    pub selected: bool,

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
    // #[data(ignore)]
    // #[lens(ignore)]
    // pub trip: Option<Rc<RawTrip>>,
    pub n_stops: usize,
}
impl MyTrip {
    pub fn new(route_id: String) -> Self {
        MyTrip {
            selected: false,

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

            n_stops: 0,
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct MyRoute {
    pub selected: bool,

    // #[lens(ignore)]
    // #[data(ignore)]
    // pub id: String,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub short_name: String,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub long_name: String,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub desc: Option<String>,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub route_type: MyRouteType,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub url: Option<String>,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub agency_id: Option<String>,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub order: Option<u32>,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub color: MyRGB8,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub text_color: MyRGB8,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub continuous_pickup: MyContinuousPickupDropOff,
    // #[lens(ignore)]
    // #[data(ignore)]
    // pub continuous_drop_off: MyContinuousPickupDropOff,
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

    pub n_stops: usize,
}
impl MyRoute {
    pub fn new(agency_id: Option<String>) -> MyRoute {
        MyRoute {
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

            n_stops: 0,
        }
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyAgency {
    pub selected: bool,

    pub id: Option<String>,
    pub name: String,
    pub url: String,
    pub timezone: String,
    pub lang: Option<String>,
    pub phone: Option<String>,
    pub fare_url: Option<String>,
    pub email: Option<String>,

    // #[lens(ignore)]
    // #[data(ignore)]
    // pub agency: Option<Rc<Agency>>,
    pub n_stops: usize,
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub agencies: Vector<MyAgency>,
    pub routes: Vector<MyRoute>,
    pub trips: Vector<MyTrip>,
    pub stop_times: Vector<MyStopTime>,
}

pub fn make_initial_data(gtfs: RawGtfs) -> AppData {
    let mut agencies = gtfs.agencies.unwrap();
    let mut routes = gtfs.routes.unwrap();
    let mut trips = gtfs.trips.unwrap();
    let mut stop_times = gtfs.stop_times.unwrap();
    // let mut stops = gtfs.stops.unwrap();

    let agencies = agencies
        .iter()
        .map(|agency| MyAgency {
            selected: false,

            id: agency.id.clone(),
            name: agency.name.clone(),
            url: agency.url.clone(),
            timezone: agency.timezone.clone(),
            lang: agency.lang.clone(),
            phone: agency.phone.clone(),
            fare_url: agency.fare_url.clone(),
            email: agency.email.clone(),

            n_stops: routes
                .iter()
                .filter(|route| route.agency_id == agency.id)
                .count(),
        })
        .collect::<Vector<_>>();

    let mut routes = routes
        .iter()
        .map(|route| MyRoute {
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

            n_stops: trips
                .iter()
                .filter(|trip| trip.route_id == route.id)
                .count(),
        })
        .collect::<Vector<_>>();
    routes.sort_by(|route1, route2| route1.short_name.cmp(&route2.short_name));

    let trips = trips
        .iter()
        .enumerate()
        .map(|(i, trip)| MyTrip {
            selected: false,

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

            // trip: Some(Rc::new(trip.clone())),
            n_stops: stop_times
                .iter()
                .filter(|stop_time| stop_time.trip_id == trip.id)
                .count(),
        })
        .collect::<Vector<_>>();

    // let mut stop_times = stop_times
    //     .iter()
    //     // .filter(|stop_time| stop_time.trip_id == trip.id)
    //     .map(|stop_time| MyStopTime {
    //         selected: false,

    //         trip_id: Arc::new(stop_time.trip_id.clone()),
    //         arrival_time: stop_time.arrival_time.clone(),
    //         departure_time: stop_time.departure_time.clone(),
    //         stop_id: Arc::new(stop_time.stop_id.clone()),
    //         stop_sequence: stop_time.stop_sequence.clone(),
    //         stop_headsign: stop_time
    //             .stop_headsign
    //             .clone()
    //             .map(|stop_headsign| Arc::new(stop_headsign)),
    //         pickup_type: MyPickupDropOffType(stop_time.pickup_type.clone()),
    //         drop_off_type: MyPickupDropOffType(stop_time.drop_off_type.clone()),
    //         continuous_pickup: MyContinuousPickupDropOff(stop_time.continuous_pickup.clone()),
    //         continuous_drop_off: MyContinuousPickupDropOff(stop_time.continuous_drop_off.clone()),
    //         shape_dist_traveled: stop_time.shape_dist_traveled.clone(),
    //         timepoint: MyTimepointType(stop_time.timepoint.clone()),
    //     })
    //     .collect::<Vector<_>>();
    // stop_times.sort_by(|stop1, stop2| stop1.stop_sequence.cmp(&stop2.stop_sequence));

    // let stops = stops
    //     .iter()
    //     .map(|stop| MyStop {
    //         live: true,
    //         selected: false,
    //         scroll_to_me: 0,

    //         id: stop.id.clone(),
    //         code: stop.code.clone(),
    //         name: stop.name.clone(),
    //         description: stop.description.clone(),
    //         location_type: MyLocationType(stop.location_type.clone()),
    //         parent_station: stop.parent_station.clone(),
    //         zone_id: stop.zone_id.clone(),
    //         url: stop.url.clone(),
    //         longitude: stop.longitude.clone(),
    //         latitude: stop.latitude.clone(),
    //         timezone: stop.timezone.clone(),
    //         wheelchair_boarding: MyAvailability(stop.wheelchair_boarding.clone()),
    //         level_id: stop.level_id.clone(),
    //         platform_code: stop.platform_code.clone(),
    //         transfers: stop.transfers.len(),
    //         pathways: stop.pathways.len(),

    //         stop: Some(Rc::new(stop.clone())),
    //         coord: (stop.longitude.unwrap(), stop.latitude.unwrap()),
    //     })
    //     .collect::<Vector<_>>();

    let app_data = AppData {
        agencies,
        routes,
        trips,
        stop_times: Vector::new(),
    };
    app_data
}
