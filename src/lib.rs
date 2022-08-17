// ignore unused warnings while prototyping
#![allow(unused)]

macro_rules! myprint {
    ($($args: expr),*) => {
        print!("{} [{}:{}] ", chrono::Utc::now().time(), file!(), line!());
        $(
            print!("{}", $args);
        )*
        println!("");
    }
}
// pub(crate) use myprint;

mod list_select;
pub use list_select::ListItem;

mod map;
pub use map::MapWidget;

mod data;
pub use data::*;

mod views;
pub use views::*;

mod app_delegate;
pub use app_delegate::*;
