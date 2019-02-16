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
use log::{info, trace, debug};
use std::io::{self, BufRead};

mod api;
mod config;
mod currency;
mod db;
mod price_in_text;
mod rate;

use crate::config::Config;
use crate::currency::{Currency, EUR, USD};
use crate::rate::Rate;
use crate::db::Db;

fn main() {
    env_logger::init();
    info!("Starting up");

    let cfg = Config::get();

    // Manager for the database
    let mut mgr = Manager::new();
    info!("Initialize database");
    let mut kcfg = KvConfig::default(&cfg.db_path);
    let db = Db::new(kcfg, &mut mgr);

    let mut txt;
    // Acquire text to extract conversion instruction
    {
        info!("Reading stdin…");
        let stdin = io::stdin();
        txt = stdin
            .lock()
            .lines()
            .next()
            .expect("Please provide some text on stdin")
            .unwrap();
        debug!("stdin: {}", txt);
    }
    let currency_amounts = price_in_text::iso(&currency::ALL_CURRENCIES, &txt);

    if let Some(currency_amount) = currency_amounts.get(0) {
        let src_currency = currency_amount.currency();
        // TODO Use config instead of &USD
        let dst_currency = &USD;
        trace!("src_currency: {}", &src_currency);

        // Get rate
        trace!("Get db handler");
        let sh = db.store_handle().write().unwrap();
        trace!("Get rate bucket");
        let bucket = db.bucket_rate(&sh);
        trace!("Got bucket");
        {
            let rate_from_db = || -> Option<Rate> {
                debug!("Create read transaction");
                let txn = sh.read_txn().unwrap();
                trace!("Get rate from db");
                let rate = db.get_rate(&txn, &sh, src_currency, dst_currency);
                trace!("rate_from_db: {:?}", rate);
                rate
            };

            let add_to_db = |rate: Rate| {
                debug!("Get write transaction");
                let mut txn = sh.write_txn().unwrap();
                trace!("Set rate to db");
                let r = db.set_rate(&mut txn, &sh, &bucket, rate);
                trace!("Rate set, result: {:?}", &r);
                txn.commit().unwrap();
            };

            let rate_from_api = || -> Option<Rate> {
                use crate::api::RateApi;
                let client = reqwest::Client::new();
                let endpoint = crate::api::ExchangeRatesApiIo::new(&cfg);
                endpoint.rate(&client, &src_currency, dst_currency)
            };

            let rate = rate_from_db()
                .or_else(rate_from_api);

            if let Some(rate) = &rate {
                info!("Rate retrieved: {}", &rate);
            } else {
                info!("No rate retrieved");
            }
            trace!("Final rate: {:?}", &rate);
            if let Some(rate) = rate {
                dbg!(currency_amount.convert(&rate));
                debug!("Set rate to db");
                add_to_db(rate)
            }
        }
    } else {
        println!("No currency found.")
    }
}
