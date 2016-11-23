extern crate flate2;
extern crate glob;
extern crate hyper;
#[macro_use] extern crate nickel;
extern crate nickel_sqlite;
extern crate rusqlite;
extern crate rustc_serialize;
extern crate toml;

pub mod api;
pub mod db;
mod config;
pub use config::BDCSConfig;
