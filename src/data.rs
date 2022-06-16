use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::{Data, Lens, Widget, WidgetExt};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use std::collections::HashMap;

pub trait ListItem {
    fn update_selection(&mut self, value: bool);
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyStopTime {
    pub selected: bool,
    pub stop_sequence: u16,
    // #[data(ignore)]
    // #[lens(ignore)]
    // stop_time: Rc<RawStopTime>,
    // stop_time: RawStopTime,
    pub name: String,
    pub coord: (f64, f64),
}
impl ListItem for MyStopTime {
    fn update_selection(&mut self, value: bool) {
        self.selected = value;
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyTrip {
    pub selected: bool,
    pub expanded: bool,
    // #[data(ignore)]
    // trip: RawTrip,
    pub name: String,
    pub stops: Vector<MyStopTime>,
}
impl ListItem for MyTrip {
    fn update_selection(&mut self, value: bool) {
        self.selected = value;
        self.stops
            .iter_mut()
            .for_each(|stop| stop.update_selection(value));
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyRoute {
    pub selected: bool,
    pub expanded: bool,
    #[data(ignore)]
    pub route: Route,
    pub trips: Vector<MyTrip>,
}
impl ListItem for MyRoute {
    fn update_selection(&mut self, value: bool) {
        self.selected = value;
        self.trips
            .iter_mut()
            .for_each(|trip| trip.update_selection(value));
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyAgency {
    pub selected: bool,
    pub expanded: bool,
    #[data(ignore)]
    pub agency: Agency,
    pub routes: Vector<MyRoute>,
}
impl ListItem for MyAgency {
    fn update_selection(&mut self, value: bool) {
        self.selected = value;
        self.routes
            .iter_mut()
            .for_each(|route| route.update_selection(value));
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct AppData {
    pub agencies: Vector<MyAgency>,
    pub expanded: bool,
}
impl AppData {
    pub fn trip_coords(&self) -> Vec<Vec<(f64, f64)>> {
        self.agencies
            .iter()
            .filter(|agency| agency.selected)
            .map(|agency| {
                agency
                    .routes
                    .iter()
                    .filter(|route| route.selected && route.trips.len() > 0)
                    .map(|route| {
                        route
                            .trips
                            .iter()
                            .filter(|trip| trip.selected)
                            .map(|trip| {
                                trip.stops
                                    .iter()
                                    .filter(|stop| stop.selected)
                                    .map(|stop| stop.coord)
                                    .collect::<Vec<_>>()
                            })
                            .flatten()
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    }
}
impl ListItem for AppData {
    fn update_selection(&mut self, value: bool) {
        self.agencies
            .iter_mut()
            .for_each(|agency| agency.update_selection(value));
    }
}

pub fn make_initial_data(gtfs: RawGtfs) -> AppData {
    let mut agencies = gtfs.agencies.unwrap();
    let mut routes = gtfs.routes.unwrap();
    let mut trips = gtfs.trips.unwrap();
    let mut stop_times = gtfs.stop_times.unwrap();
    let mut stops = gtfs.stops.unwrap();

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

    // hash map for getting a top by stop_id
    let mut stop_map = HashMap::new();
    let stops2 = stops.clone();
    stops2.iter().for_each(|stop| {
        stop_map.insert(stop.id.clone(), stop.clone());
    });

    agencies.sort_by(|x1, x2| x1.name.cmp(&x2.name));

    let app_data = AppData {
        expanded: true,
        agencies: agencies
            .iter()
            .map(|agency| {
                let mut routes = routes
                    .iter()
                    .filter(|route| route.agency_id == agency.id)
                    .map(|route| MyRoute {
                        selected: true,
                        expanded: false,
                        route: route.clone(),
                        trips: trips
                            .iter()
                            .enumerate()
                            .filter(|(i, trip)| trip.route_id == route.id)
                            .map(|(i, trip)| {
                                let (start_index, end_index) =
                                    stop_time_range_from_trip_id.get(&trip.id).unwrap().clone();
                                let mut stops = stop_times[start_index..end_index]
                                    .iter()
                                    // .filter(|stop_time| stop_time.trip_id == trip.id)
                                    .map(|stop_time| {
                                        let stop = stop_map.get(&stop_time.stop_id).unwrap();
                                        MyStopTime {
                                            selected: true,
                                            stop_sequence: stop_time.stop_sequence,
                                            // stop_time: Rc::new(stop_time.clone()),
                                            // stop_time: stop_time.clone(),
                                            name: stop.name.clone(),
                                            coord: (
                                                stop.longitude.unwrap(),
                                                stop.latitude.unwrap(),
                                            ),
                                        }
                                    })
                                    .collect::<Vector<_>>();
                                stops.sort_by(|stop1, stop2| {
                                    stop1.stop_sequence.cmp(&stop2.stop_sequence)
                                });

                                // adding the RawTrip to MyTrip is the tipping point which kills performance. Maybe AppData should just be storing a u32 index of the items position in the original RawGtfs data
                                MyTrip {
                                    selected: true,
                                    expanded: false,
                                    // trip: Rc::new(trip.clone()),
                                    name: trip.id.clone(),
                                    stops,
                                }
                            })
                            .collect::<Vector<_>>(),
                    })
                    .collect::<Vector<_>>();
                routes.sort_by(|route1, route2| {
                    route1.route.short_name.cmp(&route2.route.short_name)
                });
                MyAgency {
                    selected: true,
                    expanded: false,
                    agency: agency.clone(),
                    routes,
                }
            })
            .collect::<Vector<_>>(),
    };
    app_data
}
