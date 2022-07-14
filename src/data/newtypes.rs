use deepsize::DeepSizeOf;
use druid::Data;
use gtfs_structures::{
    Agency, Availability, BikesAllowedType, ContinuousPickupDropOff, DirectionType, Gtfs,
    LocationType, PickupDropOffType, RawGtfs, RawStopTime, RawTrip, Route, RouteType, Stop,
    StopTime, TimepointType, Trip,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct MyLocationType(pub LocationType);
impl MyLocationType {
    pub fn radio_vec() -> Vec<(String, MyLocationType)> {
        vec![
            (
                "StopPoint".to_string(),
                MyLocationType(LocationType::StopPoint),
            ),
            (
                "StopArea".to_string(),
                MyLocationType(LocationType::StopArea),
            ),
            (
                "StationEntrance".to_string(),
                MyLocationType(LocationType::StationEntrance),
            ),
            (
                "GenericNode".to_string(),
                MyLocationType(LocationType::GenericNode),
            ),
            (
                "BoardingArea".to_string(),
                MyLocationType(LocationType::BoardingArea),
            ),
            (
                "Unknown(99)".to_string(),
                MyLocationType(LocationType::Unknown(99)),
            ),
        ]
    }
}
impl Data for MyLocationType {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct MyTimepointType(pub TimepointType);
impl MyTimepointType {
    pub fn radio_vec() -> Vec<(String, MyTimepointType)> {
        vec![
            (
                "Approximate".to_string(),
                MyTimepointType(TimepointType::Approximate),
            ),
            ("Exact".to_string(), MyTimepointType(TimepointType::Exact)),
        ]
    }
}
impl Data for MyTimepointType {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct MyPickupDropOffType(pub PickupDropOffType);
impl MyPickupDropOffType {
    pub fn radio_vec() -> Vec<(String, MyPickupDropOffType)> {
        vec![
            (
                "Regular".to_string(),
                MyPickupDropOffType(PickupDropOffType::Regular),
            ),
            (
                "NotAvailable".to_string(),
                MyPickupDropOffType(PickupDropOffType::NotAvailable),
            ),
            (
                "ArrangeByPhone".to_string(),
                MyPickupDropOffType(PickupDropOffType::ArrangeByPhone),
            ),
            (
                "CoordinateWithDriver".to_string(),
                MyPickupDropOffType(PickupDropOffType::CoordinateWithDriver),
            ),
            (
                "Unknown(99)".to_string(),
                MyPickupDropOffType(PickupDropOffType::Unknown(99)),
            ),
        ]
    }
}
impl Data for MyPickupDropOffType {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct MyDirectionType(pub DirectionType);
impl MyDirectionType {
    pub fn radio_vec() -> Vec<(String, MyDirectionType)> {
        vec![
            (
                "Outbound".to_string(),
                MyDirectionType(DirectionType::Outbound),
            ),
            (
                "Inbound".to_string(),
                MyDirectionType(DirectionType::Inbound),
            ),
        ]
    }
}
impl Data for MyDirectionType {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct MyAvailability(pub Availability);
impl MyAvailability {
    pub fn radio_vec() -> Vec<(String, MyAvailability)> {
        vec![
            (
                "InformationNotAvailable".to_string(),
                MyAvailability(Availability::InformationNotAvailable),
            ),
            (
                "Available".to_string(),
                MyAvailability(Availability::Available),
            ),
            (
                "NotAvailable".to_string(),
                MyAvailability(Availability::NotAvailable),
            ),
            (
                "Unknown(99)".to_string(),
                MyAvailability(Availability::Unknown(99)),
            ),
        ]
    }
}
impl Data for MyAvailability {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, DeepSizeOf)]
pub struct MyBikesAllowedType(pub BikesAllowedType);
impl MyBikesAllowedType {
    pub fn radio_vec() -> Vec<(String, MyBikesAllowedType)> {
        vec![
            (
                "NoBikeInfo".to_string(),
                MyBikesAllowedType(BikesAllowedType::NoBikeInfo),
            ),
            (
                "AtLeastOneBike".to_string(),
                MyBikesAllowedType(BikesAllowedType::AtLeastOneBike),
            ),
            (
                "NoBikesAllowed".to_string(),
                MyBikesAllowedType(BikesAllowedType::NoBikesAllowed),
            ),
            (
                "Unknown(99)".to_string(),
                MyBikesAllowedType(BikesAllowedType::Unknown(99)),
            ),
        ]
    }
}
impl Data for MyBikesAllowedType {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
