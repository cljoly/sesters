/*
Sesters: easily convert one currency to another
Copyright (C) 2018-2019  Clément Joly <oss+sesters@131719.xyz>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use kv::{Config as KvConfig, Manager};
use log::info;

use std::path::Path;

mod currency;
mod config;
mod price_in_text;
mod rate_db;

use crate::config::Config;

fn main() {
    env_logger::init();
    info!("Starting up");

    let cfg = Config::get();

    // Manager for the database
    let mut mgr = Manager::new();
    info!("Initialize database");
    let mut cfg = KvConfig::default(Path::new(&cfg.db_path));
    let store_handle = mgr.open(cfg).unwrap();
    let store = store_handle.write().unwrap();
}
