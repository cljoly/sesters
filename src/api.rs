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

//! Access several API used by Sesters

use reqwest;

use crate::currency::Currency;
use crate::db::Rate;

use log::warn;
use std::collections::HashMap;

/// Client wrapper
pub struct Client(reqwest::Client);

impl Client {
    /// Create a new client for the API provider
    pub fn new() -> Self {
        Client(reqwest::Client::new())
    }

    /// Get rate, if it exists
    // TODO Adapt to something else than https://www.currencyconverterapi.com/
    pub fn rate<'c>(&self, src: &'c Currency, dst: &'c Currency) -> Option<Rate<'c>> {
        let pair = format!("{0}_{1}", src.get_main_iso(), dst.get_main_iso());
        let rate_err = |pair: &str| -> Result<Rate, Box<dyn std::error::Error>> {
            let client = &self.0;
            let mut res = client
                .get("https://free.currencyconverterapi.com/api/v6/convert")
                .query(&[("q", pair), ("compact", "ultra")])
                .send()?;
            // XXX Maybe HashMap is too long to build
            let rates: HashMap<String, f64> = res.json()?;
            Ok(Rate::now(src, dst, rates[pair]))
        };
        match rate_err(&pair) {
            Err(e) => {
                warn!("Error while performing request for {}: {}", pair, e);
                None
            }
            Ok(rate) => {
                Some(rate)
            }
        }
    }
}
