use std::rc::Rc;

use chrono::Utc;
use druid::im::{ordmap, vector, OrdMap, Vector};
use druid::image::buffer::Rows;
use druid::lens::{self, LensExt};
use druid::text::{EditableText, TextStorage};
use druid::widget::{
    Button, Checkbox, Container, Controller, CrossAxisAlignment, Either, Flex, FlexParams, Label,
    LabelText, LineBreaking, List, MainAxisAlignment, Painter, RadioGroup, Scroll, Stepper,
    TextBox, ViewSwitcher,
};
use druid::{
    AppDelegate, AppLauncher, Color, Data, Env, Event, EventCtx, FontDescriptor, FontFamily,
    FontWeight, Insets, Key, Lens, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, Point,
    RenderContext, Selector, UnitPoint, UpdateCtx, Widget, WidgetExt, WindowDesc,
};
use gtfs_structures::ContinuousPickupDropOff;
use rgb::RGB8;

// parameters
pub const FIELDS_TOP_PADDING: f64 = 20.;
pub const SPACING_1: f64 = 20.;
// const NARROW_LIST_WIDTH: f64 = 200.;
pub const NARROW_LIST_WIDTH: f64 = 600.;
pub const CORNER_RADIUS: f64 = 5.;
pub const HEADING_1: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::BOLD)
    .with_size(24.0);
pub const HEADING_2: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::BOLD)
    .with_size(20.0);
pub const ANNOTATION: FontDescriptor = FontDescriptor::new(FontFamily::SYSTEM_UI)
    .with_weight(FontWeight::THIN)
    .with_size(10.0);

pub const VARIABLE_STOP_TIME_BORDER_COLOR: Key<Color> =
    Key::new("druid-help.stop-time.border-color");
pub const VARIABLE_SELECTED_ITEM_BORDER_COLOR: Key<Color> =
    Key::new("druid-help.list-item.background-color");
pub const SELECTED_ITEM_BORDER_COLOR: Color = Color::RED;
pub const VARIABLE_ITEM_BORDER_WIDTH: Key<f64> = Key::new("selected.stop_time.border");
pub const SELECTED_ITEM_BORDER_WIDTH: f64 = 1.;
pub const FIELD_SPACER_SIZE: f64 = 5.;
pub const CHILD_LIST_SPACING: f64 = 5.;
