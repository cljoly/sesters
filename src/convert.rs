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

//! Module for the convert subcommand

use clap::Values as ClapValues;
use clap::{value_t, ArgMatches};
use itertools::Itertools;
use log::{debug, info, log_enabled, trace};
use std::io::{self, BufRead};

use crate::api;
use crate::api::RateApi;
use crate::currency::PriceTag;
use crate::rate::Rate;
use crate::MainContext;

/// Concat the args with spaces, if args are not `None`. Read text from the
/// first line of stdin otherwise.
fn concat_or_stdin_1_line(arg_text: Option<ClapValues>) -> String {
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

/// Return content of stdin in a buffer
fn stdin_buf() -> String {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    stdin
        .fill_buf()
        .map(|bytes| String::from_utf8_lossy(bytes).into())
        .unwrap_or(String::new())
}

/// Perform and display conversion for a PriceTag
fn perform_display(ctxt: &MainContext, price_tag: &PriceTag) {
    let src_currency = price_tag.currency();
    trace!("src_currency: {}", &src_currency);

    // Get rate
    trace!("Get db handler");
    let sh = ctxt.db.store_handle().write().unwrap();
    trace!("Get rate bucket");
    let bucket = ctxt.db.bucket_rate(&sh);
    trace!("Got bucket");
    let endpoint = api::ExchangeRatesApiIo::new(&ctxt.cfg);
    trace!("Got API Endpoint");
    {
        let rate_from_db = |dst_currency| -> Option<Rate> {
            debug!("Create read transaction");
            let txn = sh.read_txn().unwrap();
            trace!("Get rate from db");
            let (uptodate_rates, outdated_rates) = ctxt.db.get_rates(
                &txn,
                &sh,
                src_currency,
                dst_currency,
                &endpoint.provider_id(),
            );

            // Remove outdated_rates
            let mut txnw = sh.write_txn().unwrap();
            for rate in outdated_rates {
                ctxt.db.del_rate(&mut txnw, &bucket, rate);
            }

            let rate = uptodate_rates.last();
            trace!("rate_from_db: {:?}", rate);
            rate.map(|r| r.clone())
        };

        let add_to_db = |rate: Rate| {
            debug!("Get write transaction");
            let mut txn = sh.write_txn().unwrap();
            trace!("Set rate to db");
            let r = ctxt.db.set_rate(&mut txn, &bucket, rate);
            trace!("Rate set, result: {:?}", &r);
            txn.commit().unwrap();
        };

        let rate_from_api = |dst_currency| -> Option<Rate> {
            info!("Retrieve rate online");
            let client = reqwest::Client::new();
            endpoint.rate(&client, &src_currency, dst_currency)
        };

        let rates = ctxt.destination_currencies.iter().map(|dst| {
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
                // Skip conversion that wouldn’t change currency (like 1 BTC -> 1 BTC)
                if price_tag.currency() == rate.dst() {
                    continue;
                }
                println!("{} ➜ {}", &price_tag, &price_tag.convert(&rate).unwrap());
            }
        }
    }
}

/// Parse arguments for convert subcommand and run it
pub(crate) fn run(ctxt: MainContext, matches: &ArgMatches) {
    let txt;
    if matches.is_present("STDIN") {
        txt = stdin_buf();
    } else {
        txt = concat_or_stdin_1_line(matches.values_of("PLAIN_TXT"));
    }
    trace!("plain text: {}", &txt);
    let engine: crate::price_in_text::Engine = crate::price_in_text::Engine::new().unwrap();
    let mut price_tags = engine.all_price_tags(&txt);
    if matches.is_present("FINDN") {
        price_tags = price_tags
            .into_iter()
            .take(value_t!(matches.value_of("FINDN"), usize).unwrap_or_else(|e| e.exit()))
            .collect();
    }
    let price_tags_iter = price_tags.iter();
    if price_tags.len() == 0 {
        println!("No currency found.");
        return;
    }
    for price_tag in price_tags_iter {
        perform_display(&ctxt, price_tag);
    }
}
