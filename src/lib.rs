#![warn(rust_2018_idioms)]
#![feature(duration_saturating_ops)]

#[cfg(test)]
pub mod test_helper;

pub mod app;
pub mod client;
pub mod constant;
pub mod event;
pub mod handler;
pub mod key_event_wrapper;
pub mod loader;
pub mod logevents;
pub mod loggroups;
pub mod state;
pub mod terminal;
pub mod ui;
pub mod utils;
