//! BDCS API Server
//!
//!
//! Note: This requires the nightly Rust compiler and the following
//! packages on the host:
//!
//! * sqlite-devel
//! * openssl-devel
//!
//! # Overview
//!
//! ## BDCS API
//!
//! See the documentation on the [bdcs](../bdcs/index.html) crate for details about the library and the
//! API that this service provides.
//!
//! # Arguments
//!
//! * `--host` - IP to bind to, defaults to `127.0.0.1`
//! * `--port` - Port to use, defaults to `4000`
//! * `--log` - Path to logfile, which uses the slog JSON format. Defaults to `/var/log/bdcs-api.log`
//! * `--mockfiles` - Path to JSON files used for `/api/mock/` paths. Defaults to `/var/tmp/bdcs-mockfiles/`
//! * `DB` - Path to the metadata sqlite database created by the Haskell bdcs utility.
//! * `RECIPES` - Path to the directory holding the TOML formatted recipes.
//!

// Copyright (C) 2016-2017 Red Hat, Inc.
//
// This file is part of bdcs-api-server.
//
// bdcs-api-server is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// bdcs-api-server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with bdcs-api-server.  If not, see <http://www.gnu.org/licenses/>.

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate bdcs;
#[macro_use] extern crate clap;
extern crate rocket;
extern crate rusqlite;
extern crate rustc_serialize;
#[macro_use] extern crate slog;
extern crate slog_json;
#[macro_use] extern crate slog_scope;
extern crate slog_stream;
extern crate slog_term;
extern crate toml;


use std::fs::{File, OpenOptions};
use std::io::Write;

use bdcs::{RocketToml, RocketConfig};
use bdcs::api::{v0, mock, docs};
use bdcs::db::DBPool;
use bdcs::recipe::{self, RecipeRepo};
use clap::{Arg, App};
use slog::DrainExt;


/// Process Command Line Arguments and Serve the http API
fn main() {
    let matches = App::new("bdcs-api")
                            .about("A REST API on top of the BDCS")
                            .version(crate_version!())
                            .arg(Arg::with_name("host")
                                        .long("host")
                                        .value_name("HOSTNAME|IP")
                                        .help("Host or IP to bind to (127.0.0.1)")
                                        .takes_value(true))
                            .arg(Arg::with_name("port")
                                        .long("port")
                                        .value_name("PORT")
                                        .help("Port to bind to (4000)")
                                        .takes_value(true))
                            .arg(Arg::with_name("log")
                                        .long("log")
                                        .value_name("LOGFILE")
                                        .help("Path to JSON logfile")
                                        .takes_value(true))
                            .arg(Arg::with_name("mockfiles")
                                        .long("mockfiles")
                                        .value_name("MOCKFILES")
                                        .help("Path to JSON files used for /api/mock/ paths")
                                        .takes_value(true))
                            .arg(Arg::with_name("DB")
                                        .help("Path to the BDCS sqlite database")
                                        .required(true)
                                        .index(1))
                            .arg(Arg::with_name("RECIPES")
                                        .help("Path to the recipes")
                                        .required(true)
                                        .index(2))
                        .get_matches();

    // Write out the config to a Rocket.toml (this is easier than using rocket::custom)
    let rocket_config = RocketToml {
        global: RocketConfig {
            address: matches.value_of("host").unwrap_or("127.0.0.1").to_string(),
            port: matches.value_of("port").unwrap_or("").parse().unwrap_or(4000),
            db_path: matches.value_of("DB").unwrap().to_string(),
            recipe_path: matches.value_of("RECIPES").unwrap().to_string(),
            log_path: matches.value_of("log").unwrap_or("/var/log/bdcs-api.log").to_string(),
            mockfiles_path: matches.value_of("mockfiles").unwrap_or("/var/tmp/bdcs-mockfiles/").to_string()

        }
    };

    // Write out a Rocket.toml config with [global] settings
    let rocket_toml = toml::to_string(&rocket_config).unwrap();
    File::create("Rocket.toml").unwrap()
        .write_all(rocket_toml.as_bytes()).unwrap();

    // Setup logging
    let term_drain = slog_term::streamer().build();
    let log_file = OpenOptions::new()
                       .create(true)
                       .append(true)
                       .open(&rocket_config.global.log_path)
                       .expect("Error opening logfile for writing.");
    let file_drain = slog_stream::stream(log_file, slog_json::default());
    let log = slog::Logger::root(slog::duplicate(term_drain, file_drain).fuse(), o!());
    slog_scope::set_global_logger(log);

    info!(format!("BDCS API v{} started", crate_version!()));
    info!("Config:"; "rocket_config" => format!("{:?}", rocket_config));

    // Import the recipes from recipe_path into master branch of the git repository
    {
        let repo = recipe::init_repo(&rocket_config.global.recipe_path).unwrap();
        recipe::add_dir(&repo, &rocket_config.global.recipe_path, "master", false).unwrap();
    }

    rocket::ignite()
        .mount("/api/v0/", routes![ v0::test, v0::isos, v0::compose, v0::compose_types, v0::compose_cancel,
                                   v0::compose_status, v0::compose_status_id, v0::compose_log,
                                   v0::projects_list_default, v0::projects_list_filter,
                                   v0::projects_info,
                                   v0::projects_depsolve,
                                   v0::modules_info,
                                   v0::modules_list_default, v0::modules_list_filter,
                                   v0::modules_list_noargs_default, v0::modules_list_noargs_filter,
                                   v0::recipes_list_default, v0::recipes_list_filter,
                                   v0::recipes_info,
                                   v0::recipes_info_toml,
                                   v0::recipes_changes_default, v0::recipes_changes_filter,
                                   v0::recipes_diff,
                                   v0::options_recipes_new, v0::recipes_new_json, v0::recipes_new_toml,
                                   v0::options_recipes_delete, v0::recipes_delete,
                                   v0::recipes_undo,
                                   v0::recipes_depsolve])
        .mount("/api/mock/", routes![mock::static_route, mock::static_route_filter,
                                     mock::static_route_param, mock::static_route_param_filter,
                                     mock::static_route_action, mock::static_route_action_filter])
        .mount("/api/docs/", routes![docs::index, docs::files])
        .manage(DBPool::new(&rocket_config.global.db_path))
        .manage(RecipeRepo::new(&rocket_config.global.recipe_path))
        .launch();
}
