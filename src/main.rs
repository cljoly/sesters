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
use ::clap::Values as ClapValues;
use log::{error, info, trace};
use std::io::{self, BufRead};
use itertools::Itertools;

mod api;
mod config;
mod convert;
mod currency;
mod db;
mod price_in_text;
mod rate;
mod clap;

use crate::config::Config;
use crate::db::Db;
use crate::currency::Currency;

/// Main context to pass what is initiliazed in this module and what is parsed
/// in global tags
pub struct MainContext<'mc> {
    db: Db,
    destination_currencies: Vec<&'mc Currency>,
    cfg: Config,
}

/// Concat the args with spaces, if args are not `None`. Read text from stdin
/// otherwise.
fn concat_or_stdin(arg_text: Option<ClapValues>) -> String {
    fn read_stdin() -> String {
        info!("Reading stdin…");
        eprintln!("Enter the plain text on the first line");
        let stdin = io::stdin();
        let txt = stdin
            .lock()
            .lines()
            .next()
            .expect("Please provide some text on stdin")
            .unwrap();
        trace!("txt: {}", txt);
        txt
    }
    fn space_join(values: ClapValues) -> String {
        let mut txt = String::new();
        let spaced_values = values.intersperse(" ");
        for s in spaced_values {
            txt.push_str(s);
        }
        txt
    }
    arg_text.map_or_else(read_stdin, space_join)
}

fn main() {
    log::set_max_level(log::LevelFilter::Info);
    env_logger::init();
    info!("Starting up");

    let matches = crate::clap::get_app().get_matches();

    let cfg = Config::get();

    // Manager for the database
    let mut mgr = Manager::new();
    info!("Initialize database");
    let kcfg = KvConfig::default(&cfg.db_path);
    let db = Db::new(kcfg, &mut mgr);

    // Argument parsing
    let currency_iso_names_cfg: Vec<&str> = cfg.currencies.iter().map(|s| s.as_str()).collect();
    let currency_iso_names: Vec<&str> = matches.values_of("TO").map_or(currency_iso_names_cfg, |to| to.collect());
    let destination_currencies = currency_iso_names.iter().filter_map(|iso_name| {
        currency::existing_from_iso(&iso_name).or_else(|| {
            error!(
                "Invalid currency iso symbol '{}', ignored",
                iso_name
            );
            None
        })
    }).collect();

    let ctxt = MainContext { db, cfg, destination_currencies };

    if let Some(matches) = matches.subcommand_matches("convert") {
        let txt = concat_or_stdin(matches.values_of("PLAIN_TXT"));
        trace!("plain text: {}", &txt);
        crate::convert::run(ctxt, txt);
    }

    info!("Exiting");
}
