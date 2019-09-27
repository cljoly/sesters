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
use log::{debug, error, info, log_enabled, trace};
use std::io::{self, BufRead};
use clap::{clap_app,crate_version,crate_authors,crate_description};
use itertools::Itertools;

mod api;
mod config;
mod currency;
mod db;
mod price_in_text;
mod rate;

use crate::api::RateApi;
use crate::config::Config;
use crate::db::Db;
use crate::rate::Rate;

/// Concat the args with spaces, if args are not `None`. Read text from stdin
/// otherwise.
fn concat_or_stdin(arg_text: Option<clap::Values>) -> String {
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
    fn space_join(values: clap::Values) -> String {
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

    let matches = clap_app!(sesters =>
        // (@setting DontCollapseArgsInUsage)
        (version: crate_version!())
        (author: crate_authors!())
        (about: concat!(crate_description!(), "\n", "https://seste.rs"))
        // TODO Implement -c
        // (@arg CONFIG: -c --config +global +takes_value "Sets a custom config file")
        // TODO Add flag for verbosity, for preferred currency
        (@arg TO: -t --to +takes_value +global +multiple "Currency to convert to, uses defaults from the configuration file if not set")
        (@subcommand convert =>
            (@setting TrailingVarArg)
            // (@setting DontDelimitTrailingValues)
            (about: "Perform currency conversion to your preferred currency, from a price tag found in plain text")
            (visible_alias: "c")
            (@arg PLAIN_TXT: +multiple !use_delimiter "Plain text to extract a price tag from. If not set, plain text will be read from stdin")
        )
    ).get_matches();

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
    });
    let mut txt;
    if let Some(matches) = matches.subcommand_matches("convert") {
        txt = concat_or_stdin(matches.values_of("PLAIN_TXT"));
        trace!("plain text: {}", &txt);
    } else {
        // TODO This is available here only because the rest of the code is dependant of the text to convert and becase it is executed whether the convert subcommand is used or not. Thus, to actually use subcommand, the code below should be called in the if branch of this else, allowing to remove the else.
        txt = String::new();
    }

    let engine: crate::price_in_text::Engine = crate::price_in_text::Engine::new().unwrap();
    let price_tags = engine.all_price_tags(&txt);
    if let Some(price_tag) = price_tags.get(0) {
        let src_currency = price_tag.currency();
        trace!("src_currency: {}", &src_currency);

        // Get rate
        trace!("Get db handler");
        let sh = db.store_handle().write().unwrap();
        trace!("Get rate bucket");
        let bucket = db.bucket_rate(&sh);
        trace!("Got bucket");
        let endpoint = api::ExchangeRatesApiIo::new(&cfg);
        trace!("Got API Endpoint");
        {
            let rate_from_db = |dst_currency| -> Option<Rate> {
                debug!("Create read transaction");
                let txn = sh.read_txn().unwrap();
                trace!("Get rate from db");
                let (uptodate_rates, outdated_rates) = db.get_rates(
                    &txn,
                    &sh,
                    src_currency,
                    dst_currency,
                    &endpoint.provider_id(),
                );
                let rate = uptodate_rates.last();
                trace!("rate_from_db: {:?}", rate);
                rate.map(|r| r.clone())
            };

            let add_to_db = |rate: Rate| {
                debug!("Get write transaction");
                let mut txn = sh.write_txn().unwrap();
                trace!("Set rate to db");
                let r = db.set_rate(&mut txn, &sh, &bucket, rate);
                trace!("Rate set, result: {:?}", &r);
                txn.commit().unwrap();
            };

            let rate_from_api = |dst_currency| -> Option<Rate> {
                info!("Retrieve rate online");
                let client = reqwest::Client::new();
                endpoint.rate(&client, &src_currency, dst_currency)
            };

            let rates = destination_currencies.map(|dst| {
                rate_from_db(&dst).or_else(|| {
                    let rate = rate_from_api(&dst);
                    if let Some(rate) = &rate {
                        info!("Set rate to db");
                        add_to_db(rate.clone());
                    }
                    rate
                })
            });

            for rate in rates {
                if log_enabled!(log::Level::Info) {
                    if let Some(rate) = &rate {
                        info!("Rate retrieved: {}", &rate);
                    } else {
                        info!("No rate retrieved");
                    }
                }
                trace!("Final rate: {:?}", &rate);
                if let Some(rate) = rate {
                    println!(
                        "{} ➜ {}",
                        &price_tag,
                        &price_tag.convert(&rate).unwrap()
                    );
                }
            }
        }
    } else {
        println!("No currency found.")
    }
    info!("Exiting");
}
