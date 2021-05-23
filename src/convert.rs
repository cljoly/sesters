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

use anyhow::{Context, Result};
use clap::ArgMatches;
use clap::Values as ClapValues;
use itertools::Itertools;
use log::{info, log_enabled, trace};
use std::io::{self, BufRead};
use ureq::Agent;

use crate::currency::PriceTag;
use crate::rate::Rate;
use crate::MainContext;
use crate::{api::RateApi, config::CurrencyConverterApiCom};

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
        let spaced_values = Itertools::intersperse(values, " ");
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

/// Parse arguments for convert subcommand and run it
pub(crate) fn run(ctxt: MainContext, matches: &ArgMatches) -> Result<()> {
    let txt;
    if matches.is_present("STDIN") {
        txt = stdin_buf();
    } else {
        txt = concat_or_stdin_1_line(matches.values_of("PLAIN_TXT"));
    }
    trace!("plain text: {}", &txt);
    let engine: crate::price_in_text::Engine = crate::price_in_text::Engine::new().unwrap();
    let price_tags = engine.all_price_tags(&txt);
    return match price_tags.get(0) {
        Some(price_tag) => handle_pricetag(ctxt, price_tag),
        None => {
            println!("No currency found.");
            Ok(())
        }
    };
}

fn handle_pricetag(ctxt: MainContext, price_tag: &PriceTag) -> Result<()> {
    let src_currency = price_tag.currency();
    trace!("src_currency: {}", &src_currency);

    let now = chrono::offset::Utc::now();

    // Get rate
    let endpoint = CurrencyConverterApiCom::new(&ctxt.cfg);
    trace!("Got API Endpoint");
    let rate_from_db = |dst_currency| -> Option<Rate> {
        // TODO Create transaction to keep outdated rates if the update to a new rate is unsucessful?
        trace!("Get rate from db");
        let uptodate_rates = ctxt
            .db
            .get_uptodate_rates(src_currency, dst_currency, &endpoint.provider_id(), now)
            .context("Failed to retrieve rates from the database")
            .ok()?;

        let rate = uptodate_rates.last();
        trace!("rate_from_db: {:?}", rate);
        rate.map(|r| r.clone())
    };

    let add_to_db = |rate: Rate| {
        trace!("Set rate to db");
        ctxt.db.set_rate(&rate).unwrap();
    };

    let rate_from_api = |dst_currency| -> Option<Rate> {
        info!("Retrieve rate online");
        let agent = Agent::new();
        endpoint.rate(&agent, &src_currency, dst_currency)
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
            // TODO Move this to the pricetag engine
            if price_tag.currency() == rate.dst() {
                continue;
            }
            println!("{} ➜ {}", &price_tag, &price_tag.convert(&rate).unwrap());
        }
    }

    for dst in ctxt.destination_currencies {
        ctxt.db
            .remove_outdated_rates(src_currency, dst, &endpoint.provider_id(), now)?;
    }

    Ok(())
}
