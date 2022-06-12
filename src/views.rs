use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::lens::{self, LensExt};
use druid::widget::{
    Button, Checkbox, Container, CrossAxisAlignment, Either, Flex, FlexParams, Label, List,
    MainAxisAlignment, Scroll,
};
use druid::{
    AppLauncher, Color, Data, Env, EventCtx, FontDescriptor, FontFamily, FontWeight, Insets, Lens,
    LocalizedString, Point, UnitPoint, Widget, WidgetExt, WindowDesc,
};

use crate::data::*;
use crate::map::MapWidget;

const CORNER_RADIUS: f64 = 10.;
const HEADING_1: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::BOLD)
    .with_size(24.0);
const HEADING_2: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::BOLD)
    .with_size(20.0);

// todo make a custom checkbox which has data (String, bool) so the label value can be taken from the data AND be clickable
pub fn stop_ui() -> impl Widget<MyStopTime> {
    Container::new(
        Flex::row()
            .with_child(Label::new(|data: &MyStopTime, _env: &_| {
                format!("{}", data.name)
            }))
            .with_child(Checkbox::new("").lens(MyStopTime::selected))
            // .with_child(Either::new(
            //     |data: &Trip, _env: &Env| data.selected,
            //     List::new(stop_ui).lens(Trip::stops),
            //     Label::new(""),
            // ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding((10., 10., 10., 10.)),
    )
    .rounded(CORNER_RADIUS)
    .background(Color::grey(0.16))
    .expand_width()
}

pub fn trip_ui() -> impl Widget<MyTrip> {
    // let label = Label::new(|data: &bool, env: &Env| "hi");
    let title = Flex::row()
        .with_child(Label::new(|data: &MyTrip, _env: &_| {
            format!("{}", data.name)
        }))
        .with_child(Checkbox::new("").lens(MyTrip::selected).on_click(
            |ctx: &mut EventCtx, data: &mut MyTrip, env: &Env| {
                if data.selected {
                    data.selected = false;
                    data.stops.iter_mut().for_each(|stop| stop.selected = false);
                } else {
                    data.selected = true;
                    data.stops.iter_mut().for_each(|stop| stop.selected = true);
                }
            },
        ));

    Container::new(
        Flex::column()
            .with_child(title)
            .with_child(Checkbox::new("Stops >").lens(MyTrip::expanded))
            .with_child(Either::new(
                |data: &MyTrip, _env: &Env| data.expanded,
                Flex::column()
                    .with_child(
                        Flex::row()
                            .with_child(Button::new("select all").on_click(
                                |_, data: &mut MyTrip, _| {
                                    data.stops.iter_mut().for_each(|stop| stop.selected = true)
                                },
                            ))
                            .with_child(Button::new("clear all").on_click(
                                |_, data: &mut MyTrip, _| {
                                    data.stops.iter_mut().for_each(|stop| stop.selected = false)
                                },
                            )),
                    )
                    .with_child(List::new(stop_ui).with_spacing(10.).lens(MyTrip::stops)),
                Flex::row(),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding((10., 10., 10., 10.)),
    )
    .rounded(CORNER_RADIUS)
    // .background(Color::grey(0.1))
    .background(Color::rgb(54. / 255., 74. / 255., 63. / 255.))
    .expand_width()
}

pub fn route_ui() -> impl Widget<MyRoute> {
    let title = Flex::row()
        .with_child(
            Label::new(|data: &MyRoute, _env: &_| format!("{}", data.route.short_name))
                .with_font(HEADING_2),
        )
        .with_child(Checkbox::new("").lens(MyRoute::selected));

    Container::new(
        Flex::column()
            .with_child(title)
            .with_default_spacer()
            .with_child(Checkbox::new("Trips >").lens(MyRoute::expanded))
            .with_child(Either::new(
                |data: &MyRoute, _env: &Env| data.expanded,
                Flex::column()
                    .with_child(
                        Flex::row()
                            .with_child(Button::new("select all").on_click(
                                |_, data: &mut MyRoute, _| {
                                    data.trips.iter_mut().for_each(|trip| {
                                        trip.selected = true;
                                        trip.stops.iter_mut().for_each(|stop| stop.selected = true)
                                    })
                                },
                            ))
                            .with_child(Button::new("clear all").on_click(
                                |_, data: &mut MyRoute, _| {
                                    data.trips.iter_mut().for_each(|trip| {
                                        trip.selected = false;
                                        trip.stops.iter_mut().for_each(|stop| stop.selected = false)
                                    })
                                },
                            )),
                    )
                    .with_child(List::new(trip_ui).with_spacing(10.).lens(MyRoute::trips)),
                Flex::row(),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding((10., 10., 10., 10.)),
    )
    .rounded(CORNER_RADIUS)
    .background(Color::grey(0.16))
    .expand_width()
}

pub fn agency_ui() -> impl Widget<MyAgency> {
    let title = Flex::row()
        .with_child(
            Label::new(|data: &MyAgency, _env: &_| format!("{}", data.agency.name))
                .with_font(HEADING_2),
        )
        .with_child(Checkbox::new("").lens(MyAgency::selected));

    Container::new(
        Flex::column()
            .with_child(title)
            .with_spacer(20.)
            .with_child(
                Flex::row()
                    .with_child(Checkbox::new("Routes >").lens(MyAgency::expanded))
                    .with_child(
                        Flex::row()
                            .with_child(Button::new("select all").on_click(
                                |_, data: &mut MyAgency, _| {
                                    data.routes.iter_mut().for_each(|route| {
                                        route.selected = true;
                                        route.trips.iter_mut().for_each(|trip| {
                                            trip.selected = true;
                                            trip.stops
                                                .iter_mut()
                                                .for_each(|stop| stop.selected = true)
                                        })
                                    })
                                },
                            ))
                            .with_child(Button::new("clear all").on_click(
                                |_, data: &mut MyAgency, _| {
                                    data.routes.iter_mut().for_each(|route| {
                                        route.selected = false;
                                        route.trips.iter_mut().for_each(|trip| {
                                            trip.selected = false;
                                            trip.stops
                                                .iter_mut()
                                                .for_each(|stop| stop.selected = false)
                                        })
                                    })
                                },
                            )),
                    )
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .expand_width(),
            )
            .with_default_spacer()
            .with_child(Either::new(
                |data: &MyAgency, _env: &Env| data.expanded,
                List::new(route_ui).with_spacing(10.).lens(MyAgency::routes),
                Flex::row(),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start),
    )
    .padding((10., 10., 10., 10.))
    // .background(Color::grey(0.1))
    .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
    .rounded(CORNER_RADIUS)
    .fix_width(800.)
}

pub fn main_widget() -> impl Widget<AppData> {
    // todo what's the difference between Point::ZERO and Point::ORIGIN?
    println!("make main widget");
    let map_widget = (MapWidget::new(1., 1., Point::ZERO)).expand();
    Flex::row()
        .with_child(
            Flex::column()
                .with_child(
                    Checkbox::new("Agencies >")
                        .align_left()
                        .lens(AppData::expanded),
                )
                .with_default_spacer()
                .with_flex_child(
                    Either::new(
                        |data: &AppData, _env: &Env| data.expanded,
                        Scroll::new(
                            Flex::column()
                                .with_child(
                                    Flex::row()
                                        .with_child(Button::new("select all").on_click(
                                            |_, data: &mut AppData, _| {
                                                data.agencies
                                                    .iter_mut()
                                                    .for_each(|trip| trip.selected = true)
                                            },
                                        ))
                                        .with_child(Button::new("clear all").on_click(
                                            |_, data: &mut AppData, _| {
                                                data.agencies
                                                    .iter_mut()
                                                    .for_each(|trip| trip.selected = false)
                                            },
                                        )),
                                )
                                .with_child(
                                    List::new(agency_ui)
                                        .with_spacing(10.)
                                        .lens(AppData::agencies),
                                ),
                        )
                        .fix_width(800.),
                        Flex::row().fix_width(800.),
                    ),
                    1.,
                ),
        )
        .with_spacer(20.)
        .with_flex_child(map_widget, FlexParams::new(1.0, CrossAxisAlignment::Start))
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .padding(20.)
}
