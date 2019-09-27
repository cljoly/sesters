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

use log::{trace, debug, info, log_enabled};

use crate::api;
use crate::api::RateApi;
use crate::rate::Rate;
use crate::MainContext;
 
pub fn run(ctxt: MainContext, txt: String) {
    let engine: crate::price_in_text::Engine = crate::price_in_text::Engine::new().unwrap();
    let price_tags = engine.all_price_tags(&txt);
    if let Some(price_tag) = price_tags.get(0) {
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
                let rate = uptodate_rates.last();
                trace!("rate_from_db: {:?}", rate);
                rate.map(|r| r.clone())
            };

            let add_to_db = |rate: Rate| {
                debug!("Get write transaction");
                let mut txn = sh.write_txn().unwrap();
                trace!("Set rate to db");
                let r = ctxt.db.set_rate(&mut txn, &sh, &bucket, rate);
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
}

