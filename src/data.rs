use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::{Data, Lens, Widget, WidgetExt};
use gtfs_structures::{Agency, Gtfs, RawGtfs, RawStopTime, RawTrip, Route, Stop, StopTime, Trip};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub trait ListItem {
    fn update_all(&mut self, value: bool);
    fn id(&self) -> String;
    fn item_type(&self) -> String;
    // fn data_info(&self) -> String;
    fn data_info(&self) -> String {
        "hello".to_string()
    }
    // fn name(&self) -> String;
}

#[derive(Clone, Data, Lens)]
pub struct MyStopTime {
    pub live: bool,
    pub selected: bool,
    pub stop_sequence: u16,
    #[data(ignore)]
    #[lens(ignore)]
    pub stop_time: Rc<RawStopTime>,
    // stop_time: RawStopTime,
    pub name: String,
    pub coord: (f64, f64),
}
impl ListItem for MyStopTime {
    fn update_all(&mut self, value: bool) {
        self.selected = value;
    }
    fn id(&self) -> String {
        self.stop_time.stop_id.clone()
    }
    fn item_type(&self) -> String {
        "stop_time".to_string()
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyTrip {
    pub live: bool,
    pub selected: bool,
    pub expanded: bool,
    #[data(ignore)]
    #[lens(ignore)]
    pub trip: Rc<RawTrip>,
    // #[data(ignore)]
    // trip: RawTrip,
    pub name: String,
    pub trip_headsign: String,
    pub stops: Vector<MyStopTime>,
}
impl ListItem for MyTrip {
    fn update_all(&mut self, value: bool) {
        self.selected = value;
        self.stops
            .iter_mut()
            .for_each(|stop| stop.update_all(value));
    }
    fn id(&self) -> String {
        self.trip.id.clone()
    }
    fn item_type(&self) -> String {
        "trip".to_string()
    }
    fn data_info(&self) -> String {
        format!(
            "{} -> {}",
            self.trip.trip_headsign.clone().unwrap(),
            self.trip_headsign
        )
    }
}

#[derive(Clone, Data, Default, Lens)]
pub struct MyRoute {
    pub selected: bool,
    pub expanded: bool,
    #[lens(ignore)]
    #[data(ignore)]
    pub route: Rc<Route>,
    pub trips: Vector<MyTrip>,
}
impl ListItem for MyRoute {
    fn update_all(&mut self, value: bool) {
        self.selected = value;
        self.trips
            .iter_mut()
            .for_each(|trip| trip.update_all(value));
    }
    fn id(&self) -> String {
        self.route.id.clone()
    }
    fn item_type(&self) -> String {
        "route".to_string()
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
    fn update_all(&mut self, value: bool) {
        self.selected = value;
        self.routes
            .iter_mut()
            .for_each(|route| route.update_all(value));
    }
    fn id(&self) -> String {
        // todo handle agency.id == None
        self.agency.id.as_ref().unwrap().clone()
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
pub struct Edit {
    pub id: usize,
    pub edit_type: EditType,
    pub item_type: String,
    pub item_id: String,
    #[data(ignore)]
    #[lens(ignore)]
    pub item_data: Option<Rc<dyn ListItem>>,
}
impl Debug for Edit {
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
pub struct AppData {
    #[data(ignore)]
    #[lens(ignore)]
    pub gtfs: Rc<MyGtfs>,
    pub agencies: Vector<MyAgency>,
    pub expanded: bool,
    pub edits: Vector<Edit>,
}
impl ListItem for AppData {
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
        gtfs: Rc::new(my_gtfs),
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
                        route: Rc::new(route.clone()),
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
                                            live: true,
                                            selected: true,
                                            stop_sequence: stop_time.stop_sequence,
                                            stop_time: Rc::new(stop_time.clone()),
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
                                    live: true,
                                    selected: true,
                                    expanded: false,
                                    trip: Rc::new(trip.clone()),
                                    name: trip.id.clone(),
                                    trip_headsign: trip.trip_headsign.clone().unwrap(),
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
        edits: Vector::new(),
    };
    app_data
}
