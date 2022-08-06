use std::rc::Rc;

use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::lens::{self, LensExt};
use druid::text::{EditableText, TextStorage};
use druid::widget::{
    Button, Checkbox, Container, Controller, CrossAxisAlignment, Either, Flex, FlexParams, Label,
    LabelText, LineBreaking, List, MainAxisAlignment, Painter, RadioGroup, Scroll, Stepper,
    TextBox, ViewSwitcher,
};
use druid::{
    AppDelegate, AppLauncher, Color, Data, Env, Event, EventCtx, FontDescriptor, FontFamily,
    FontWeight, Insets, Key, Lens, LocalizedString, PaintCtx, Point, RenderContext, Selector,
    UnitPoint, UpdateCtx, Widget, WidgetExt, WindowDesc, LifeCycle, LifeCycleCtx,
};
use gtfs_structures::ContinuousPickupDropOff;
use rgb::RGB8;

use crate::app_delegate::*;
use crate::data::*;
use crate::map::MapWidget;

mod dropdown;
mod expander;
mod filtered_list;
use dropdown::*;
use expander::Expander;
use filtered_list::FilteredList;

// parameters
const SPACING_1: f64 = 20.;
// const NARROW_LIST_WIDTH: f64 = 200.;
const NARROW_LIST_WIDTH: f64 = 600.;
const CORNER_RADIUS: f64 = 5.;
const HEADING_1: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::BOLD)
    .with_size(24.0);
const HEADING_2: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::BOLD)
    .with_size(20.0);
const ANNOTATION: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::THIN)
    .with_size(10.0);

const VARIABLE_SELECTED_ITEM_BORDER_COLOR: Key<Color> =
    Key::new("druid-help.list-item.background-color");
const SELECTED_ITEM_BORDER_COLOR: Color = Color::RED;
const VARIABLE_ITEM_BORDER_WIDTH: Key<f64> = Key::new("selected.stop_time.border");
const SELECTED_ITEM_BORDER_WIDTH: f64 = 1.;
const FIELD_SPACER_SIZE: f64 = 5.;

// const DARK_BLUE: Color = Color::rgb(54. / 255., 58. / 255., 74. / 255.);
// const DARK_GREEN: Color = Color::rgb(54. / 255., 74. / 255., 63. / 255.);

struct TextBoxOnChange;
impl<W: Widget<MyTrip>> Controller<MyTrip, W> for TextBoxOnChange {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &MyTrip,
        data: &MyTrip,
        env: &Env,
    ) {
        //  !bug
        if !old_data.trip_headsign.same(&data.trip_headsign) && old_data.id().same(&data.id()) {
            dbg!(&old_data.trip_headsign);
            dbg!(&data.trip_headsign);
            ctx.submit_command(ITEM_UPDATE.with((data.item_type(), data.id())));
        }
        child.update(ctx, old_data, data, env);
    }
}

fn delete_item_button<T: Data + ListItem>() -> impl Widget<T> {
    Button::new("delete").on_click(|ctx, data: &mut T, _| {
        ctx.submit_command(ITEM_DELETE.with((data.item_type(), data.id())));
    })
}
fn update_all_buttons<T: Data + ListItem>() -> impl Widget<T> {
    Flex::column()
        .with_child(Button::new("new child").on_click(|ctx, data: &mut T, _| {
            ctx.submit_command(ITEM_NEW_CHILD.with((data.item_type(), data.id())));
        }))
        .with_child(delete_item_button())
        .with_child(Button::new("select all").on_click(|_, data: &mut T, _| {
            data.update_all(true);
        }))
        .with_child(Button::new("deselect all").on_click(|_, data: &mut T, _| {
            data.update_all(false);
        }))
}
fn child_buttons<T: Data + ListItem>() -> impl Widget<T> {
    Flex::row()
        .with_child(Button::new("new child").on_click(|ctx, data: &mut T, _| {
            ctx.submit_command(ITEM_NEW_CHILD.with((data.item_type(), data.id())));
        }))
        .with_child(Button::new("select all").on_click(|_, data: &mut T, _| {
            data.update_all(true);
        }))
        .with_child(Button::new("deselect all").on_click(|_, data: &mut T, _| {
            data.update_all(false);
        }))
}

fn option_string_checkbox() -> impl Widget<Option<String>> {
    // "poo".to_string().
    Flex::row()
        .with_child(Checkbox::new("").lens(druid::lens::Map::new(
            |data: &Option<String>| match data {
                Some(_) => true,
                None => false,
            },
            |data: &mut Option<String>, inner: bool| {
                *data = if inner {
                    Some("nuttin".to_string())
                } else {
                    None
                };
            },
        )))
        .with_child(
            TextBox::new()
                .with_placeholder("hi")
                .lens(druid::lens::Map::new(
                    |data: &Option<String>| match data {
                        Some(text) => text.clone(),
                        None => "nutin".to_string(),
                    },
                    |data: &mut Option<String>, inner: String| {
                        match data {
                            Some(old_inner) => {
                                old_inner.clear();
                                old_inner.push_str(&inner);
                            }
                            None => {}
                        };
                    },
                ))
                .disabled_if(|data: &Option<String>, _| data.is_none()),
        )
}
fn option_string() -> impl Widget<Option<String>> {
    TextBox::new()
        .with_placeholder("empty")
        .lens(druid::lens::Map::new(
            |data: &Option<String>| match data {
                Some(text) => text.clone(),
                None => "".to_string(),
            },
            |data: &mut Option<String>, inner: String| {
                if inner.is_empty() {
                    *data = None;
                } else {
                    *data = Some(inner);
                }
            },
        ))
}
fn option_u32() -> impl Widget<Option<u32>> {
    Stepper::new().lens(druid::lens::Map::new(
        |data: &Option<u32>| match data {
            Some(num) => *num as f64,
            None => 0.,
        },
        |data: &mut Option<u32>, inner: f64| {
            if inner == 0. {
                *data = None;
            } else {
                *data = Some(inner as u32);
            }
        },
    ))
}
fn option_f32() -> impl Widget<Option<f32>> {
    Stepper::new().lens(druid::lens::Map::new(
        |data: &Option<f32>| match data {
            Some(num) => *num as f64,
            None => 0.,
        },
        |data: &mut Option<f32>, inner: f64| {
            if inner == 0. {
                *data = None;
            } else {
                *data = Some(inner as f32);
            }
        },
    ))
}
fn option_num<T>() -> impl Widget<Option<T>>
where
    T: Data + Copy + std::convert::Into<f64> + std::convert::From<f64>,
{
    Stepper::new().lens(druid::lens::Map::new(
        |data: &Option<T>| match data {
            Some(num) => (*num).into(),
            None => 0.,
        },
        |data: &mut Option<T>, inner: f64| {
            if inner == 0. {
                *data = None;
            } else {
                *data = Some(T::from(inner));
            }
        },
    ))
}

fn field_row<T: Data>(
    name: &str,
    update: impl Widget<T> + 'static,
    updated_flag: impl Fn(&T, &Env) -> bool + 'static,
) -> impl Widget<T> {
    Flex::column()
        .with_child(Label::new(name).with_font(ANNOTATION).fix_width(300.))
        .with_child(update.fix_width(300.))
        // .with_child(Either::new(
        //     updated_flag,
        //     Label::new("udpated").fix_width(100.),
        //     Label::new("").fix_width(100.),
        // ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

pub fn stop_ui() -> impl Widget<MyStop> {
    let title = Flex::row()
        .with_child(Checkbox::new("").lens(MyStop::selected))
        .with_default_spacer()
        .with_child(Either::new(
            |data: &MyStop, _env: &Env| data.live,
            Label::new(""),
            Label::new("deleted").with_text_color(Color::RED),
        ))
        .with_default_spacer()
        .with_child(Either::new(
            |data: &MyStop, _env: &Env| data.stop.is_none(),
            Label::new("new item").with_text_color(Color::RED),
            Label::new(""),
        ))
        .with_default_spacer()
        .with_child(delete_item_button());

    let fields = Flex::column()
        .with_child(field_row(
            "id",
            Label::new(|data: &MyStop, _: &_| format!("{:?}", data.id)),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.id != data.id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "code",
            option_string().lens(MyStop::code),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.code != data.code,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "name",
            TextBox::new()
                .with_placeholder("stop name")
                .lens(MyStop::name),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.name != data.name,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "description",
            TextBox::new()
                .with_placeholder("stop description")
                .lens(MyStop::description),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.description != data.description,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "location_type",
            Dropdown::new(
                Button::new(|data: &MyStop, _: &Env| format!("{:?}", data.location_type.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyLocationType::radio_vec())
                        // .fix_size(5., 10.)
                        .lens(druid::lens::Map::new(
                            |data: &MyStop| data.location_type.clone(),
                            |data: &mut MyStop, inner: MyLocationType| {
                                data.location_type = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.location_type != data.location_type.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "parent_station",
            option_string().lens(MyStop::parent_station),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.parent_station != data.parent_station,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "zone_id",
            option_string().lens(MyStop::zone_id),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.zone_id != data.zone_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "url",
            option_string().lens(MyStop::url),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.url != data.url,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "longitude",
            option_num().lens(MyStop::longitude),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.longitude != data.longitude,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "latitude",
            option_num().lens(MyStop::latitude),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.latitude != data.latitude,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "timezone",
            option_string().lens(MyStop::timezone),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.timezone != data.timezone,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "wheelchair_boarding",
            Dropdown::new(
                Button::new(|data: &MyStop, _: &Env| format!("{:?}", data.wheelchair_boarding.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyAvailability::radio_vec())
                        // .fix_size(5., 10.)
                        .lens(druid::lens::Map::new(
                            |data: &MyStop| data.wheelchair_boarding.clone(),
                            |data: &mut MyStop, inner: MyAvailability| {
                                data.wheelchair_boarding = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.wheelchair_boarding != data.wheelchair_boarding.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "level_id",
            option_string().lens(MyStop::level_id),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.level_id != data.level_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "platform_code",
            option_string().lens(MyStop::platform_code),
            |data: &MyStop, _: &_| match &data.stop {
                Some(stop) => stop.platform_code != data.platform_code,
                None => true,
            },
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start);

    item_container(
        Flex::column()
            .with_child(title)
            .with_spacer(SPACING_1)
            .with_child(fields)
            .cross_axis_alignment(CrossAxisAlignment::Start),
    )

    // Container::new(
    //     Flex::column()
    //         .with_child(title)
    //         .with_spacer(SPACING_1)
    //         .with_child(fields)
    //         .cross_axis_alignment(CrossAxisAlignment::Start)
    //         .padding((10., 10., 10., 10.)),
    // )
    // .rounded(CORNER_RADIUS)
    // // .background(Color::grey(0.16))
    // .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
    // // .expand_width()
    // .controller(ScrollToMeController {})
    // .fix_width(600.)
}

// pub fn stop_time_ui_small() -> impl Widget<MyStopTime> {
//     Container::new(
//         Label::new(|data: &MyStopTime, _env: &_| {
//             format!(
//                 "{}: {}",
//                 data.stop_sequence,
//                 data.stop.as_ref().unwrap().name
//             )
//         })
//         .padding((10., 10., 10., 10.)),
//     )
//     .rounded(CORNER_RADIUS)
//     .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
//     // .border(Color::RED, 0.)
//     // .background(BG_COLOR)
//     // .env_scope(|env, item| env.set(BG_COLOR, Color::RED))
//     // .env_scope(|env, item| env.set(BG_COLOR,  Color::BLUE.into()))
//     .border(SELECTED_ITEM_BORDER_COLOR, VARIABLE_ITEM_BORDER_WIDTH)
//     // EnvScope must wrap the border, else Missing Key panic
//     .env_scope(|env, stop_time| {
//         // dbg!("set env");
//         env.set(
//             VARIABLE_ITEM_BORDER_WIDTH,
//             if stop_time.selected {
//                 SELECTED_ITEM_BORDER_WIDTH
//             } else {
//                 0.
//             },
//         )
//     })
//     .fix_width(NARROW_LIST_WIDTH)
//     // can't determine why there is a big delay before click closure starts
//     .on_click(|ctx: &mut EventCtx, data: &mut MyStopTime, _: &_| {
//         dbg!("got click");
//         ctx.submit_command(SELECT_STOP_TIME.with((data.trip_id.clone(), data.stop_sequence)))
//     })
// }
pub fn selected_stop_time_ui_small() -> impl Widget<MyStopTime> {
    selected_item_container(
        Label::new(|data: &MyStopTime, _env: &_| {
            format!("{}: {}", data.stop_sequence, data.stop_name)
        })
        .with_line_break_mode(LineBreaking::Clip),
        Checkbox::new("fields").lens(MyStopTime::show_editing),
        Either::new(
            |data: &MyStopTime, _: &_| data.show_editing,
            stop_time_ui(),
            Flex::column(),
        ),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyStopTime, _: &_| {
        dbg!("got click");
        ctx.submit_command(SELECT_STOP_TIME.with((data.trip_id.clone(), data.stop_sequence)))
    })
}
pub fn stop_time_ui_small() -> impl Widget<MyStopTime> {
    unselected_item_container(
        Label::new(|data: &MyStopTime, _env: &_| {
            format!("{}: {}", data.stop_sequence, data.stop_name)
        })
        .with_line_break_mode(LineBreaking::Clip),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyStopTime, _: &_| {
        dbg!("got click");
        ctx.submit_command(SELECT_STOP_TIME.with((data.trip_id.clone(), data.stop_sequence)))
    })
}

pub fn unselected_item_container<T: Data>(child: impl Widget<T> + 'static) -> impl Widget<T> {
    Container::new(child.padding((10., 10., 10., 10.)))
        .rounded(CORNER_RADIUS)
        .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
        // .border(SELECTED_ITEM_BORDER_COLOR, VARIABLE_ITEM_BORDER_WIDTH)
        // .border(SELECTED_ITEM_BORDER_COLOR, SELECTED_ITEM_BORDER_WIDTH)
        .fix_width(NARROW_LIST_WIDTH)
}
// pub fn selected_item_container<T: Data>(child: impl Widget<T> + 'static) -> impl Widget<T> {
//     Container::new(child.padding((10., 10., 10., 10.)))
//         .rounded(CORNER_RADIUS)
//         .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
//         // .border(SELECTED_ITEM_BORDER_COLOR, VARIABLE_ITEM_BORDER_WIDTH)
//         .border(SELECTED_ITEM_BORDER_COLOR, SELECTED_ITEM_BORDER_WIDTH)
//         .fix_width(NARROW_LIST_WIDTH)
// }
pub fn selected_item_container<T: Data + ListItem>(
    child: impl Widget<T> + 'static,
    edit_checkbox: impl Widget<T> + 'static,
    either_fields: impl Widget<T> + 'static,
) -> impl Widget<T> {
    Container::new(
        Flex::column()
            .with_child(
                Flex::row()
                    .with_child(child)
                    .with_child(
                        Flex::column()
                            .with_child(edit_checkbox)
                            .with_child(delete_item_button()),
                    )
                    .must_fill_main_axis(true)
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween),
            )
            .with_child(either_fields)
            .padding((10., 10., 10., 10.)),
    )
    .rounded(CORNER_RADIUS)
    .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
    // .border(SELECTED_ITEM_BORDER_COLOR, VARIABLE_ITEM_BORDER_WIDTH)
    .border(SELECTED_ITEM_BORDER_COLOR, SELECTED_ITEM_BORDER_WIDTH)
    .fix_width(NARROW_LIST_WIDTH)
}
pub fn item_container<T: ListItem + Data>(child: impl Widget<T> + 'static) -> impl Widget<T> {
    Container::new(child.padding((10., 10., 10., 10.)))
        .rounded(CORNER_RADIUS)
        .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
        // .border(Color::RED, 0.)
        // .background(BG_COLOR)
        // .env_scope(|env, item| env.set(BG_COLOR, Color::RED))
        // .env_scope(|env, item| env.set(BG_COLOR,  Color::BLUE.into()))
        .border(SELECTED_ITEM_BORDER_COLOR, VARIABLE_ITEM_BORDER_WIDTH)
        // EnvScope must wrap the border, else Missing Key panic
        .env_scope(|env, stop_time| {
            // dbg!("set env");
            env.set(
                VARIABLE_ITEM_BORDER_WIDTH,
                if stop_time.selected() {
                    SELECTED_ITEM_BORDER_WIDTH
                } else {
                    0.
                },
            )
        })
        .fix_width(NARROW_LIST_WIDTH)
}
pub fn either_ui<T: ListItem + Data>(
    ui: impl Widget<T> + 'static,
    small_ui: impl Widget<T> + 'static,
) -> impl Widget<T> {
    item_container(Either::new(|data: &T, _: &_| data.selected(), ui, small_ui))
}

// todo make a custom checkbox which has data (String, bool) so the label value can be taken from the data AND be clickable
pub fn stop_time_ui() -> impl Widget<MyStopTime> {
    // let title = Flex::row()
    //     .with_child(Checkbox::new("").lens(MyStopTime::selected))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyStopTime, _env: &Env| data.live,
    //         Label::new(""),
    //         Label::new("deleted").with_text_color(Color::RED),
    //     ))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyStopTime, _env: &Env| data.stop_time.is_none(),
    //         Label::new("new item").with_text_color(Color::RED),
    //         Label::new(""),
    //     ))
    //     .with_default_spacer()
    //     .with_child(delete_item_button());

    let fields = Flex::column()
        .with_child(field_row(
            "trip_id",
            Label::new(|data: &MyStopTime, _: &_| format!("{:?}", data.trip_id)),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.trip_id != data.trip_id,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "arrival_time",
            option_string().lens(MyStopTime::arrival_time),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.arrival_time != data.arrival_time,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "departure_time",
            option_string().lens(MyStopTime::departure_time),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.departure_time != data.departure_time,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        // .with_child(field_row(
        //     "stop_id",
        //     TextBox::new()
        //         .with_placeholder("route stop_id")
        //         .lens(MyStopTime::stop_id),
        //     |data: &MyStopTime, _: &_| match &data.stop_time {
        //         Some(stop_time) => stop_time.stop_id != data.stop_id,
        //         None => true,
        //     },
        // ))
        .with_child(field_row(
            "stop_id",
            Button::new(|data: &MyStopTime, _: &_| data.stop_id.clone()).on_click(
                |ctx: &mut EventCtx, data: &mut MyStopTime, _| {
                    dbg!("submit sotp event");
                    ctx.submit_command(SELECT_STOP_LIST.with(data.stop_id.clone()));
                },
            ),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.stop_id != data.stop_id,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "stop_sequence",
            Stepper::new().lens(druid::lens::Map::new(
                |data: &MyStopTime| data.stop_sequence as f64,
                |data: &mut MyStopTime, inner: f64| data.stop_sequence = inner as u16,
            )),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.stop_sequence != data.stop_sequence,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "stop_headsign",
            option_string().lens(MyStopTime::stop_headsign),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.stop_headsign != data.stop_headsign,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "pickup_type",
            Dropdown::new(
                Button::new(|data: &MyStopTime, _: &Env| format!("{:?}", data.pickup_type.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyPickupDropOffType::radio_vec())
                        // .fix_size(5., 10.)
                        .lens(druid::lens::Map::new(
                            |data: &MyStopTime| data.pickup_type.clone(),
                            |data: &mut MyStopTime, inner: MyPickupDropOffType| {
                                data.pickup_type = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.pickup_type != data.pickup_type.0,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "drop_off_type",
            Dropdown::new(
                Button::new(|data: &MyStopTime, _: &Env| format!("{:?}", data.drop_off_type.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyPickupDropOffType::radio_vec())
                        // .fix_size(5., 10.)
                        .lens(druid::lens::Map::new(
                            |data: &MyStopTime| data.drop_off_type.clone(),
                            |data: &mut MyStopTime, inner: MyPickupDropOffType| {
                                data.drop_off_type = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.drop_off_type != data.drop_off_type.0,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "continuous_pickup",
            Dropdown::new(
                Button::new(|data: &MyStopTime, _: &Env| format!("{:?}", data.continuous_pickup.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyContinuousPickupDropOff::radio_vec()).lens(
                        druid::lens::Map::new(
                            |data: &MyStopTime| data.continuous_pickup.clone(),
                            |data: &mut MyStopTime, inner: MyContinuousPickupDropOff| {
                                data.continuous_pickup = inner;
                            },
                        ),
                    )
                },
            )
            .align_left(),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.continuous_pickup != data.continuous_pickup.0,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "continuous_drop_off",
            Dropdown::new(
                Button::new(|data: &MyStopTime, _: &Env| {
                    format!("{:?}", data.continuous_drop_off.0)
                })
                .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyContinuousPickupDropOff::radio_vec()).lens(
                        druid::lens::Map::new(
                            |data: &MyStopTime| data.continuous_drop_off.clone(),
                            |data: &mut MyStopTime, inner: MyContinuousPickupDropOff| {
                                data.continuous_drop_off = inner;
                            },
                        ),
                    )
                },
            )
            .align_left(),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.continuous_drop_off != data.continuous_drop_off.0,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "shape_dist_traveled",
            option_f32().lens(MyStopTime::shape_dist_traveled),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.shape_dist_traveled != data.shape_dist_traveled,
                None => true,
            },
        ))
        .with_spacer(FIELD_SPACER_SIZE)
        .with_child(field_row(
            "timepoint",
            Dropdown::new(
                Button::new(|data: &MyStopTime, _: &Env| format!("{:?}", data.timepoint.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyTimepointType::radio_vec()).lens(druid::lens::Map::new(
                        |data: &MyStopTime| data.timepoint.clone(),
                        |data: &mut MyStopTime, inner: MyTimepointType| {
                            data.timepoint = inner;
                        },
                    ))
                },
            )
            .align_left(),
            |data: &MyStopTime, _: &_| match &data.stop_time {
                Some(stop_time) => stop_time.timepoint != data.timepoint.0,
                None => true,
            },
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start);

    Flex::column()
        // .with_child(title)
        // .with_spacer(SPACING_1)
        .with_child(fields)
        // .with_child(
        //     FilteredList::new(
        //         List::new(stop_ui).with_spacing(10.),
        //         |item_data: &MyStopTime, filtered: &()| item_data.live,
        //     )
        //     .lens(druid::lens::Map::new(
        //         |data: &MyStopTime| (data.stops.clone(), ()),
        //         |data: &mut MyStopTime, inner: (Vector<MyStop>, ())| {
        //             data.stops = inner.0;
        //             // data.filter = inner.1;
        //         },
        //     )),
        // )
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

pub fn trip_ui_small() -> impl Widget<MyTrip> {
    unselected_item_container(
        Flex::column()
            .with_child(Label::new(|data: &MyTrip, _env: &_| {
                format!("{}", data.id())
            }))
            .with_child(Label::new(|data: &MyTrip, _env: &_| {
                format!("Stops: {}", data.n_stops)
            }))
            .cross_axis_alignment(CrossAxisAlignment::Start),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyTrip, _: &_| {
        ctx.submit_command(SELECT_TRIP.with(data.id.clone()))
    })
}
pub fn trip_ui_small_selected() -> impl Widget<MyTrip> {
    selected_item_container(
        Flex::column()
            .with_child(Label::new(|data: &MyTrip, _env: &_| {
                format!("{}", data.id())
            }))
            .with_child(Label::new(|data: &MyTrip, _env: &_| {
                format!("Stops: {}", data.n_stops)
            }))
            .cross_axis_alignment(CrossAxisAlignment::Start),
        Checkbox::new("fields").lens(MyTrip::show_editing),
        Either::new(
            |data: &MyTrip, _: &_| data.show_editing,
            trip_ui(),
            Flex::column(),
        ),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyTrip, _: &_| {
        ctx.submit_command(SELECT_TRIP.with(data.id.clone()))
    })
}
pub fn trip_ui() -> impl Widget<MyTrip> {
    // let title = Flex::row()
    //     .with_child(Checkbox::new("").lens(MyTrip::visible))
    //     .with_default_spacer()
    //     .with_child(Checkbox::new("select").lens(MyTrip::selected))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyTrip, _env: &Env| data.live,
    //         Label::new(""),
    //         Label::new("deleted").with_text_color(Color::RED),
    //     ))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyTrip, _env: &Env| data.trip.is_none(),
    //         Label::new("new item").with_text_color(Color::RED),
    //         Label::new(""),
    //     ))
    //     .with_default_spacer()
    //     .with_child(delete_item_button());

    let fields = Flex::column()
        .with_child(field_row(
            "id",
            Label::new(|data: &MyTrip, _: &_| format!("{:?}", data.id)),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.id != data.id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "service_id",
            Label::new(|data: &MyTrip, _: &_| format!("{:?}", data.service_id)),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.service_id != data.service_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "route_id",
            Label::new(|data: &MyTrip, _: &_| format!("{:?}", data.route_id)),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.route_id != data.route_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "shape_id",
            option_string().lens(MyTrip::shape_id),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.shape_id != data.shape_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "trip_headsign",
            option_string().lens(MyTrip::trip_headsign),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.trip_headsign != data.trip_headsign,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "trip_short_name",
            option_string().lens(MyTrip::trip_short_name),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.trip_short_name != data.trip_short_name,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "direction_id",
            Dropdown::new(
                Button::new(
                    |data: &MyTrip, _: &Env| match data.direction_id.map(|x| x.0) {
                        Some(val) => format!("{:?}", val),
                        None => "None".to_string(),
                    },
                )
                .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    Either::new(
                        |data: &MyTrip, _: &_| data.direction_id.is_some(),
                        RadioGroup::column(MyDirectionType::radio_vec())
                            .fix_size(100., 400.)
                            .lens(druid::lens::Map::new(
                                |data: &MyTrip| data.direction_id.unwrap().clone(),
                                |data: &mut MyTrip, inner: MyDirectionType| {
                                    data.direction_id = Some(inner);
                                },
                            )),
                        Label::new("None"),
                    )
                },
            )
            .align_left(),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.direction_id != data.direction_id.map(|x| x.0),
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "block_id",
            option_string().lens(MyTrip::block_id),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.block_id != data.block_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "wheelchair_accessible",
            Dropdown::new(
                Button::new(|data: &MyTrip, _: &Env| format!("{:?}", data.wheelchair_accessible.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyAvailability::radio_vec())
                        // .fix_size(5., 10.)
                        .lens(druid::lens::Map::new(
                            |data: &MyTrip| data.wheelchair_accessible.clone(),
                            |data: &mut MyTrip, inner: MyAvailability| {
                                data.wheelchair_accessible = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.wheelchair_accessible != data.wheelchair_accessible.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "bikes_allowed",
            Dropdown::new(
                Button::new(|data: &MyTrip, _: &Env| format!("{:?}", data.bikes_allowed.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyBikesAllowedType::radio_vec()).lens(druid::lens::Map::new(
                        |data: &MyTrip| data.bikes_allowed.clone(),
                        |data: &mut MyTrip, inner: MyBikesAllowedType| {
                            data.bikes_allowed = inner;
                        },
                    ))
                },
            )
            .align_left(),
            |data: &MyTrip, _: &_| match &data.trip {
                Some(trip) => trip.bikes_allowed != data.bikes_allowed.0,
                None => true,
            },
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start);

    // let children_header = Flex::row()
    //     .with_child(
    //         Flex::row()
    //             .with_child(Label::new(|data: &MyTrip, _: &_| {
    //                 data.stops.len().to_string()
    //             }))
    //             .with_child(Expander::new("Stop times").lens(MyTrip::expanded)),
    //     )
    //     .with_child(Either::new(
    //         |data: &MyTrip, _: &_| data.expanded,
    //         child_buttons(),
    //         Flex::row(),
    //     ))
    //     .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    //     .expand_width();

    // let children = Either::new(
    //     |data: &MyTrip, _env: &Env| data.expanded,
    //     FilteredList::new(
    //         List::new(stop_time_ui).with_spacing(10.),
    //         |item_data: &MyStopTime, filtered: &()| item_data.live,
    //     )
    //     .lens(druid::lens::Map::new(
    //         |data: &MyTrip| (data.stops.clone(), ()),
    //         |data: &mut MyTrip, inner: (Vector<MyStopTime>, ())| {
    //             data.stops = inner.0;
    //             // data.filter = inner.1;
    //         },
    //     )),
    //     Flex::row(),
    // );

    Flex::column()
        // .with_child(title)
        // .with_spacer(SPACING_1)
        .with_child(fields)
        // .with_spacer(SPACING_1)
        // .with_child(children_header)
        // .with_default_spacer()
        // .with_child(children)
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

pub fn route_ui_small_selected() -> impl Widget<MyRoute> {
    selected_item_container(
        Flex::column()
            .with_child(Label::new(|data: &MyRoute, _env: &_| {
                format!("{}", data.short_name)
            }))
            .with_child(
                Label::new(|data: &MyRoute, _env: &_| format!("{}", data.long_name))
                    .with_line_break_mode(LineBreaking::Clip),
            )
            .with_child(Label::new(|data: &MyRoute, _env: &_| {
                format!("Trips: {}", data.n_stops)
            }))
            .cross_axis_alignment(CrossAxisAlignment::Start),
        Checkbox::new("fields").lens(MyRoute::show_editing),
        Either::new(
            |data: &MyRoute, _: &_| data.show_editing,
            route_ui(),
            Flex::column(),
        ),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyRoute, _: &_| {
        ctx.submit_command(SELECT_ROUTE.with(data.id.clone()))
    })
}
pub fn route_ui_small() -> impl Widget<MyRoute> {
    unselected_item_container(
        Flex::column()
            .with_child(Label::new(|data: &MyRoute, _env: &_| {
                format!("{}", data.short_name)
            }))
            .with_child(
                Label::new(|data: &MyRoute, _env: &_| format!("{}", data.long_name))
                    .with_line_break_mode(LineBreaking::Clip),
            )
            .with_child(Label::new(|data: &MyRoute, _env: &_| {
                format!("Trips: {}", data.n_stops)
            }))
            .cross_axis_alignment(CrossAxisAlignment::Start),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyRoute, _: &_| {
        ctx.submit_command(SELECT_ROUTE.with(data.id.clone()))
    })
}
// pub fn route_ui_small() -> impl Widget<MyRoute> {
//     selected_item_container(Label::new(|data: &MyRoute, _env: &_| {
//         format!("{}", data.short_name)
//     }))
//     .on_click(|ctx: &mut EventCtx, data: &mut MyRoute, _: &_| {
//         ctx.submit_command(SELECT_ROUTE.with(data.id.clone()))
//     })
// }
pub fn route_ui() -> impl Widget<MyRoute> {
    // let title = Flex::row()
    //     .with_child(Checkbox::new("").lens(MyRoute::visible))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyRoute, _env: &Env| data.live,
    //         Label::new(""),
    //         Label::new("deleted").with_text_color(Color::RED),
    //     ))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyRoute, _env: &Env| data.new,
    //         Label::new("new item").with_text_color(Color::RED),
    //         Label::new(""),
    //     ))
    //     .with_default_spacer()
    //     .with_child(delete_item_button())
    //     .with_default_spacer()
    //     .with_child(
    //         Button::new("Add Trip").on_click(|ctx, data: &mut MyRoute, _| {
    //             ctx.submit_command(NEW_TRIP.with(data.id.clone()));
    //         }),
    //     );

    let fieldsdfa = Flex::column()
        .with_child(field_row(
            "id",
            Label::new(|data: &MyRoute, _: &_| format!("{:?}", data.id)),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.id != data.id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "short_name",
            Label::new(|data: &MyRoute, _: &_| data.short_name.clone()),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.short_name != data.short_name,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "long_name",
            Label::new(|data: &MyRoute, _: &_| data.long_name.clone()),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.long_name != data.long_name,
                None => true,
            },
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let fields = Flex::column()
        .with_child(field_row(
            "id",
            Label::new(|data: &MyRoute, _: &_| format!("{:?}", data.id)),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.id != data.id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "short_name",
            TextBox::new()
                .with_placeholder("route short_name")
                .lens(MyRoute::short_name),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.short_name != data.short_name,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "long_name",
            TextBox::new()
                .with_placeholder("route long_name")
                .lens(MyRoute::long_name),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.long_name != data.long_name,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "desc",
            option_string().lens(MyRoute::desc),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.desc != data.desc,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "route_type",
            Dropdown::new(
                Button::new(|data: &MyRoute, _: &Env| format!("{:?}", data.route_type.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyRouteType::radio_vec())
                        .fix_size(100., 400.)
                        .lens(druid::lens::Map::new(
                            |data: &MyRoute| data.route_type.clone(),
                            |data: &mut MyRoute, inner: MyRouteType| {
                                data.route_type = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.route_type != data.route_type.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "url",
            option_string().lens(MyRoute::url),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.url != data.url,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "agency_id",
            Label::new(|data: &MyRoute, _: &_| format!("{:?}", data.agency_id)),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.agency_id != data.agency_id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "order",
            Label::new(|data: &MyRoute, _: &_| format!("{:?}", data.order)),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.order != data.order,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "color",
            Painter::new(|ctx: &mut PaintCtx, data: &MyRoute, _: &Env| {
                let rect = ctx.size().to_rect();
                let RGB8 { r, g, b } = data.color.0;
                ctx.fill(rect, &Color::rgb8(r, g, b));
            })
            .fix_size(50., 10.),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.color != data.color.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "text_color",
            Painter::new(|ctx: &mut PaintCtx, data: &MyRoute, _: &Env| {
                let rect = ctx.size().to_rect();
                let RGB8 { r, g, b } = data.text_color.0;
                ctx.fill(rect, &Color::rgb8(r, g, b));
            })
            .fix_size(50., 10.),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.text_color != data.text_color.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "continuous_pickup",
            Dropdown::new(
                Button::new(|data: &MyRoute, _: &Env| format!("{:?}", data.continuous_pickup.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyContinuousPickupDropOff::radio_vec())
                        // .fix_size(5., 10.)
                        .lens(druid::lens::Map::new(
                            |data: &MyRoute| data.continuous_pickup.clone(),
                            |data: &mut MyRoute, inner: MyContinuousPickupDropOff| {
                                data.continuous_pickup = inner;
                            },
                        ))
                },
            )
            .align_left(),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.continuous_pickup != data.continuous_pickup.0,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "continuous_drop_off",
            Dropdown::new(
                Button::new(|data: &MyRoute, _: &Env| format!("{:?}", data.continuous_drop_off.0))
                    .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
                |_, _| {
                    RadioGroup::column(MyContinuousPickupDropOff::radio_vec()).lens(
                        druid::lens::Map::new(
                            |data: &MyRoute| data.continuous_drop_off.clone(),
                            |data: &mut MyRoute, inner: MyContinuousPickupDropOff| {
                                data.continuous_drop_off = inner;
                            },
                        ),
                    )
                },
            )
            .align_left(),
            |data: &MyRoute, _: &_| match &data.route {
                Some(route) => route.continuous_drop_off != data.continuous_drop_off.0,
                None => true,
            },
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start);

    // let children_header = Flex::row()
    //     .with_child(Expander::new("Trip variants").lens(MyRoute::expanded))
    //     .with_child(Either::new(
    //         |data: &MyRoute, _: &_| data.expanded,
    //         child_buttons(),
    //         Flex::row(),
    //     ))
    //     .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    //     .expand_width();

    // let children = Either::new(
    //     |data: &MyRoute, _env: &Env| data.expanded,
    //     FilteredList::new(
    //         List::new(trip_ui).with_spacing(10.),
    //         |item_data: &MyTrip, filtered: &()| item_data.live,
    //     )
    //     .lens(druid::lens::Map::new(
    //         |data: &MyRoute| (data.trips.clone(), ()),
    //         |data: &mut MyRoute, inner: (Vector<MyTrip>, ())| {
    //             data.trips = inner.0;
    //             // data.filter = inner.1;
    //         },
    //     )),
    //     Flex::row(),
    // );

    Flex::column()
        // .with_child(title)
        // .with_spacer(SPACING_1)
        .with_child(fields)
        // .with_spacer(SPACING_1)
        // .with_child(children_header)
        // .with_default_spacer()
        // .with_child(children)
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

pub fn agency_ui_small() -> impl Widget<MyAgency> {
    unselected_item_container(
        Flex::column()
            .with_child(Label::new(|data: &MyAgency, _env: &_| {
                format!("{}", data.name)
            }))
            .with_child(Label::new(|data: &MyAgency, _env: &_| {
                format!("Routes: {}", data.n_stops)
            }))
            .cross_axis_alignment(CrossAxisAlignment::Start),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyAgency, _: &_| {
        ctx.submit_command(SELECT_AGENCY.with(data.id.clone()))
    })
}
pub fn agency_ui_small_selected() -> impl Widget<MyAgency> {
    selected_item_container(
        Flex::column()
            .with_child(Label::new(|data: &MyAgency, _env: &_| {
                format!("{}", data.name)
            }))
            .with_child(Label::new(|data: &MyAgency, _env: &_| {
                format!("Routes: {}", data.n_stops)
            }))
            .cross_axis_alignment(CrossAxisAlignment::Start),
        Checkbox::new("fields").lens(MyAgency::show_editing),
        Either::new(
            |data: &MyAgency, _: &_| data.show_editing,
            agency_ui(),
            Flex::column(),
        ),
    )
    .on_click(|ctx: &mut EventCtx, data: &mut MyAgency, _: &_| {
        ctx.submit_command(SELECT_AGENCY.with(data.id.clone()))
    })
}
pub fn agency_ui() -> impl Widget<MyAgency> {
    // let title = Flex::row()
    //     .with_child(Either::new(
    //         |data: &MyAgency, _env: &Env| data.live,
    //         Label::new(""),
    //         Label::new("deleted").with_text_color(Color::RED),
    //     ))
    //     .with_default_spacer()
    //     .with_child(Either::new(
    //         |data: &MyAgency, _env: &Env| data.agency.is_none(),
    //         Label::new("new item").with_text_color(Color::RED),
    //         Label::new(""),
    //     ))
    //     .with_default_spacer()
    //     .with_child(delete_item_button());

    let fields = Flex::column()
        .with_child(field_row(
            "id",
            option_string().lens(MyAgency::id),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.id != data.id,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "name",
            TextBox::new()
                .with_placeholder("agency name")
                .lens(MyAgency::name),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.name != data.name,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "url",
            TextBox::new()
                .with_placeholder("agency url")
                .lens(MyAgency::url),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.url != data.url,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "timezone",
            TextBox::new()
                .with_placeholder("agency timezone")
                .lens(MyAgency::timezone),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.timezone != data.timezone,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "lang",
            option_string().lens(MyAgency::lang),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.lang != data.lang,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "phone",
            option_string().lens(MyAgency::phone),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.phone != data.phone,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "fare_url",
            option_string().lens(MyAgency::fare_url),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.fare_url != data.fare_url,
                None => true,
            },
        ))
        .with_default_spacer()
        .with_child(field_row(
            "email",
            option_string().lens(MyAgency::email),
            |data: &MyAgency, _: &_| match &data.agency {
                Some(agency) => agency.email != data.email,
                None => true,
            },
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start);

    // let children_header = Flex::row()
    //     .with_child(Expander::new("Routes").lens(MyAgency::expanded))
    //     .with_child(Either::new(
    //         |data: &MyAgency, _: &_| data.expanded,
    //         child_buttons(),
    //         Flex::row(),
    //     ))
    //     .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    //     .expand_width();

    // let children = Either::new(
    //     |data: &MyAgency, _env: &Env| data.expanded,
    //     FilteredList::new(
    //         List::new(route_ui).with_spacing(10.),
    //         |item_data: &MyRoute, filtered: &()| item_data.live,
    //     )
    //     .lens(druid::lens::Map::new(
    //         |data: &MyAgency| (data.routes.clone(), ()),
    //         |data: &mut MyAgency, inner: (Vector<MyRoute>, ())| {
    //             data.routes = inner.0;
    //             // data.filter = inner.1;
    //         },
    //     )),
    //     Flex::row(),
    // );

    Flex::column()
        // .with_child(title)
        // .with_spacer(SPACING_1)
        .with_child(fields)
        // .with_spacer(SPACING_1)
        // .with_child(children_header)
        // .with_default_spacer()
        // .with_child(children)
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

fn edit() -> impl Widget<Edit> {
    Container::new(Flex::row())
        .padding((10., 10., 10., 10.))
        // .background(Color::grey(0.1))
        .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
        .rounded(CORNER_RADIUS)
}
// fn action() -> impl Widget<Action> {
//     Container::new(
//         Flex::row()
//             .with_child(Label::new(|data: &Action, _: &_| match data.edit_type {
//                 EditType::Delete => format!(
//                     "Delete: {} {}. {}",
//                     data.item_type,
//                     data.item_id,
//                     data.item_data.clone().unwrap().data_info()
//                 ),
//                 EditType::Create => format!(
//                     "Create: {} {}. {}",
//                     data.item_type,
//                     data.item_id,
//                     data.item_data.clone().unwrap().data_info()
//                 ),
//                 EditType::Update => format!(
//                     "Update: {} {}. {}",
//                     data.item_type,
//                     data.item_id,
//                     data.item_data.clone().unwrap().data_info()
//                 ),
//             }))
//             .with_child(
//                 Button::new("x").on_click(|ctx: &mut EventCtx, data: &mut Action, _| {
//                     ctx.submit_command(EDIT_DELETE.with(data.id));
//                 }),
//             ),
//     )
//     .padding((10., 10., 10., 10.))
//     // .background(Color::grey(0.1))
//     .background(Color::rgb(54. / 255., 58. / 255., 74. / 255.))
//     .rounded(CORNER_RADIUS)
// }

fn agency_selected() -> Box<dyn Widget<AppData>> {
    Box::new(
        Flex::column()
            .with_child(
                FilteredList::new(
                    List::new(agency_ui_small_selected).with_spacing(10.),
                    |agency: &MyAgency, filtered: &Option<Option<String>>| {
                        filtered.as_ref().map_or(false, |id| &agency.id == id)
                    },
                )
                .lens(druid::lens::Map::new(
                    |data: &AppData| (data.agencies.clone(), data.selected_agency_id.clone()),
                    |data: &mut AppData, inner: (Vector<MyAgency>, Option<Option<String>>)| {
                        data.agencies = inner.0;
                        data.selected_agency_id = inner.1;
                    },
                )),
            )
            .with_default_spacer()
            .with_flex_child(
                Scroll::new(
                    FilteredList::new(
                        List::new(route_ui_small).with_spacing(10.),
                        |route: &MyRoute, filtered: &Option<Option<String>>| {
                            filtered.as_ref().map_or(false, |id| &route.agency_id == id)
                        },
                    )
                    .lens(druid::lens::Map::new(
                        |data: &AppData| (data.routes.clone(), data.selected_agency_id.clone()),
                        |data: &mut AppData, inner: (Vector<MyRoute>, Option<Option<String>>)| {
                            data.routes = inner.0;
                            data.selected_agency_id = inner.1;
                        },
                    )),
                ),
                1.,
            )
            .fix_width(NARROW_LIST_WIDTH),
    )
}

fn parent_agency_selected() -> impl Widget<AppData> {
    FilteredList::new(
        List::new(agency_ui_small).with_spacing(10.),
        |agency: &MyAgency, filtered: &Option<Option<String>>| {
            filtered.as_ref().map_or(false, |id| &agency.id == id)
        },
    )
    .lens(druid::lens::Map::new(
        |data: &AppData| (data.agencies.clone(), data.selected_agency_id.clone()),
        |data: &mut AppData, inner: (Vector<MyAgency>, Option<Option<String>>)| {
            data.agencies = inner.0;
            data.selected_agency_id = inner.1;
        },
    ))
}
fn parent_route_selected() -> impl Widget<AppData> {
    FilteredList::new(
        List::new(route_ui_small).with_spacing(10.),
        |route: &MyRoute, filtered: &Option<String>| {
            filtered.as_ref().map_or(false, |id| &route.id == id)
        },
    )
    .lens(druid::lens::Map::new(
        |data: &AppData| (data.routes.clone(), data.selected_route_id.clone()),
        |data: &mut AppData, inner: (Vector<MyRoute>, Option<String>)| {
            data.routes = inner.0;
            data.selected_route_id = inner.1;
        },
    ))
}
fn parent_trip_selected() -> impl Widget<AppData> {
    FilteredList::new(
        List::new(trip_ui_small).with_spacing(10.),
        |trip: &MyTrip, filtered: &Option<String>| {
            filtered.as_ref().map_or(false, |id| &trip.id == id)
        },
    )
    .lens(druid::lens::Map::new(
        |data: &AppData| (data.trips.clone(), data.selected_trip_id.clone()),
        |data: &mut AppData, inner: (Vector<MyTrip>, Option<String>)| {
            data.trips = inner.0;
            data.selected_trip_id = inner.1;
        },
    ))
}
fn parent_stop_time_selected() -> impl Widget<AppData> {
    FilteredList::new(
        List::new(stop_time_ui_small).with_spacing(10.),
        |stop_time: &MyStopTime, filtered: &Option<(String, u16)>| {
            filtered.as_ref().map_or(false, |id| {
                stop_time.trip_id == id.0 && stop_time.stop_sequence == id.1
            })
        },
    )
    .lens(druid::lens::Map::new(
        |data: &AppData| (data.stop_times.clone(), data.selected_stop_time_id.clone()),
        |data: &mut AppData, inner: (Vector<MyStopTime>, Option<(String, u16)>)| {
            data.stop_times = inner.0;
            data.selected_stop_time_id = inner.1;
        },
    ))
}

fn route_selected() -> Box<dyn Widget<AppData>> {
    Box::new(
        Flex::column()
            .with_child(parent_agency_selected())
            .with_child(
                FilteredList::new(
                    List::new(route_ui_small_selected).with_spacing(10.),
                    |route: &MyRoute, filtered: &Option<String>| {
                        filtered.as_ref().map_or(false, |id| &route.id == id)
                    },
                )
                .lens(druid::lens::Map::new(
                    |data: &AppData| (data.routes.clone(), data.selected_route_id.clone()),
                    |data: &mut AppData, inner: (Vector<MyRoute>, Option<String>)| {
                        data.routes = inner.0;
                        data.selected_route_id = inner.1;
                    },
                )),
            )
            .with_default_spacer()
            .with_flex_child(
                Scroll::new(
                    FilteredList::new(
                        List::new(trip_ui_small).with_spacing(10.),
                        |trip: &MyTrip, filtered: &Option<String>| {
                            filtered.as_ref().map_or(false, |id| &trip.route_id == id)
                        },
                    )
                    .lens(druid::lens::Map::new(
                        |data: &AppData| (data.trips.clone(), data.selected_route_id.clone()),
                        |data: &mut AppData, inner: (Vector<MyTrip>, Option<String>)| {
                            data.trips = inner.0;
                            data.selected_route_id = inner.1;
                        },
                    )),
                ),
                1.,
            )
            .fix_width(NARROW_LIST_WIDTH),
    )
}

fn trip_selected() -> Box<dyn Widget<AppData>> {
    Box::new(
        Flex::column()
            .with_child(parent_agency_selected())
            .with_child(parent_route_selected())
            .with_child(
                FilteredList::new(
                    List::new(trip_ui_small_selected).with_spacing(10.),
                    |trip: &MyTrip, filtered: &Option<String>| {
                        filtered.as_ref().map_or(false, |id| &trip.id == id)
                    },
                )
                .lens(druid::lens::Map::new(
                    |data: &AppData| (data.trips.clone(), data.selected_trip_id.clone()),
                    |data: &mut AppData, inner: (Vector<MyTrip>, Option<String>)| {
                        data.trips = inner.0;
                        data.selected_trip_id = inner.1;
                    },
                )),
            )
            .with_default_spacer()
            .with_flex_child(
                Scroll::new(
                    FilteredList::new(
                        List::new(stop_time_ui_small).with_spacing(10.),
                        |stop_time: &MyStopTime, filtered: &Option<String>| {
                            filtered
                                .as_ref()
                                .map_or(false, |id| &stop_time.trip_id == id)
                        },
                    )
                    .lens(druid::lens::Map::new(
                        |data: &AppData| (data.stop_times.clone(), data.selected_trip_id.clone()),
                        |data: &mut AppData, inner: (Vector<MyStopTime>, Option<String>)| {
                            data.stop_times = inner.0;
                            data.selected_trip_id = inner.1;
                        },
                    )),
                ),
                1.,
            )
            .fix_width(NARROW_LIST_WIDTH),
    )
}

fn stop_time_selected() -> Box<dyn Widget<AppData>> {
    Box::new(
        Flex::column()
            .with_child(parent_agency_selected())
            .with_child(parent_route_selected())
            .with_child(parent_trip_selected())
            .with_child(
                FilteredList::new(
                    List::new(selected_stop_time_ui_small).with_spacing(10.),
                    |stop_time: &MyStopTime, filtered: &Option<(String, u16)>| {
                        filtered.as_ref().map_or(false, |id| {
                            stop_time.trip_id == id.0 && stop_time.stop_sequence == id.1
                        })
                    },
                )
                .lens(druid::lens::Map::new(
                    |data: &AppData| (data.stop_times.clone(), data.selected_stop_time_id.clone()),
                    |data: &mut AppData, inner: (Vector<MyStopTime>, Option<(String, u16)>)| {
                        data.stop_times = inner.0;
                        data.selected_stop_time_id = inner.1;
                    },
                )),
            )
            .fix_width(NARROW_LIST_WIDTH),
    )
}

fn list_stop_selected() -> Box<dyn Widget<AppData>> {
    Box::new(
        Flex::column()
            .with_child(parent_agency_selected())
            .with_child(parent_route_selected())
            .with_child(parent_trip_selected())
            .with_child(parent_stop_time_selected())
            .with_child(
                FilteredList::new(
                    List::new(stop_ui).with_spacing(10.),
                    |stop: &MyStop, filtered: &Option<String>| {
                        filtered.as_ref().map_or(false, |id| &stop.id == id)
                    },
                )
                .lens(druid::lens::Map::new(
                    |data: &AppData| (data.stops.clone(), data.selected_stop_id.clone()),
                    |data: &mut AppData, inner: (Vector<MyStop>, Option<String>)| {
                        data.stops = inner.0;
                        data.selected_stop_id = inner.1;
                    },
                )),
            )
            .fix_width(NARROW_LIST_WIDTH),
    )
}

pub fn main_widget() -> impl Widget<AppData> {
    // todo what's the difference between Point::ZERO and Point::ORIGIN?
    println!("make main widget");

    // let agencies_header = Flex::row()
    //     .with_child(Expander::new("Agencies").lens(AppData::expanded))
    //     .with_child(Either::new(
    //         |data: &AppData, _: &_| data.expanded,
    //         child_buttons(),
    //         Flex::row(),
    //     ))
    //     .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    //     .expand_width();

    // let agencies = Scroll::new(
    //     Flex::column().with_child(
    //         List::new(|| either_ui(agency_ui(), agency_ui_small()))
    //             .with_spacing(10.)
    //             .lens(AppData::agencies),
    //     ),
    // )
    // .fix_width(NARROW_LIST_WIDTH);

    let all_together_now = ViewSwitcher::new(
        |data: &AppData, _: &_| {
            (
                data.selected_agency_id.clone(),
                data.selected_route_id.clone(),
                data.selected_trip_id.clone(),
                data.selected_stop_time_id.clone(),
                data.selected_stop_id.clone(),
            )
        },
        |selector: &(
            Option<Option<String>>,
            Option<String>,
            Option<String>,
            Option<(String, u16)>,
            Option<String>,
        ),
         data: &AppData,
         _: &_| {
            // TODO want parents to become unselected when children are selected
            // if nothing selected: agency_list
            // if agency selected: selected agency and trip list
            // ...
            // if trip selected: unselected parents, selected trip, and stop_time list
            // if stop_time selected: unselected parents, selected stop_time
            // if stop selected: unselected parents (if they are already selected, else nothing since we don't know what the parents are), selected stop

            if *selector == (None, None, None, None, None) {
                Box::new(
                    List::new(agency_ui_small)
                        .with_spacing(10.)
                        .lens(AppData::agencies),
                )
            } else if data.selected_agency_id.is_some() && data.selected_route_id == None {
                agency_selected()
            } else if data.selected_route_id.is_some() && data.selected_trip_id.is_none() {
                route_selected()
            } else if data.selected_trip_id.is_some() && data.selected_stop_time_id.is_none() {
                trip_selected()
            } else if data.selected_stop_time_id.is_some() && data.selected_stop_id.is_none() {
                stop_time_selected()
            } else if data.selected_stop_id.is_some() {
                list_stop_selected()
            } else {
                panic!("unexpected item selection combination")
            }
        },
    )
    .fix_width(NARROW_LIST_WIDTH);

    let zoom_level = RadioGroup::row(ZoomLevel::radio_group_vec()).lens(AppData::map_zoom_level);
    // let map_widget = (MapWidget::new(1., 1., Point::ZERO).on_added(
    let map_widget = (MapWidget::new(1., Point::ZERO).on_added(
        |map: &mut MapWidget, life_cycle_ctx: &mut LifeCycleCtx, data: &AppData, _: &Env| {
            map.trips_coords = data.trips_coords();
            // life_cycle_ctx.
        },
    ))
    .expand();

    Flex::row()
        .with_flex_child(all_together_now, 1.)
        .with_default_spacer()
        // .with_spacer(20.)
        .with_flex_child(
            Flex::column().with_child(zoom_level).with_child(map_widget),
            FlexParams::new(1.0, CrossAxisAlignment::Start),
        )
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .padding(20.)
}

struct ScrollToMeController;
impl<W: Widget<MyStop>> Controller<MyStop, W> for ScrollToMeController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut MyStop,
        env: &Env,
    ) {
        child.event(ctx, event, data, env);
        if let Event::MouseDown(_) = event {
            ctx.scroll_to_view();
        }
    }

    // fn lifecycle(
    //     &mut self,
    //     child: &mut W,
    //     ctx: &mut druid::LifeCycleCtx,
    //     event: &druid::LifeCycle,
    //     data: &T,
    //     env: &Env,
    // ) {
    //     child.lifecycle(ctx, event, data, env)
    // }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &MyStop,
        data: &MyStop,
        env: &Env,
    ) {
        if data.scroll_to_me > old_data.scroll_to_me {
            dbg!("scroll_to_view");
            ctx.scroll_to_view();
        }
        child.update(ctx, old_data, data, env)
    }
}
