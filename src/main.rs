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
use std::io::{self, BufRead};

mod api;
mod config;
mod currency;
mod db;
mod price_in_text;

use crate::config::Config;
use crate::currency::{EUR, USD};
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
        info!("stdin: {}", txt);
    }
    let currency_amounts = price_in_text::iso(&currency::ALL_CURRENCIES, &txt);

    if let Some(currency_amount) = currency_amounts.get(0) {
        let src_currency = currency_amount.currency();
        // TODO Use config instead of &USD
        let dst_currency = &USD;
        dbg!(&src_currency);

        // Get rate
        {
            info!("Get db handler");
            let sh = db.store_handle().read().unwrap();
            info!("Get db transaction");
            // let txn = sh.write_txn().unwrap();
            let txn = sh.read_txn().unwrap();

            info!("Get rate from db");
            let rate_from_db: Option<db::Rate> = db.get_rate(&txn, &sh, src_currency, dst_currency);
            info!("rate_from_db: {:?}", rate_from_db);
            let rate_from_api = || {
                use crate::api::RateApi;
                let client = reqwest::Client::new();
                let ccac = crate::api::CurrencyConverterApiCom::new(cfg);
                ccac.rate(&client, &src_currency, dst_currency).unwrap()
            };
            let rate = rate_from_db.unwrap_or_else(rate_from_api);
            // TODO Add to db

            dbg!(currency_amount.convert(&rate));
        }
    } else {
        println!("No currency found.")
    }
}
