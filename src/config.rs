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

// Store and retrieve user configuration

use anyhow::Result;
use clap::crate_name;
use log::info;
use serde_derive::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn data_dir() -> PathBuf {
    let mut path = dirs_next::data_dir().unwrap();
    path.push(crate_name!());
    path
}

/// Configuration file
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Version of the config file
    version: u8,
    /// Currencies to convert to
    currencies: Vec<String>,
    /// Path of the database (directory). Please note that ~ is not expanded
    db_path: PathBuf,
    /// APIs used to get exchange rates
    pub apis: Apis,
}

impl Default for Config {
    fn default() -> Self {
        let mut db_path = data_dir();
        db_path.push("db.sqlite3");
        Config {
            version: 0,
            currencies: vec!["EUR".to_string(), "USD".to_string(), "GBP".to_string()],
            db_path,
            apis: Apis::default(),
        }
    }
}

impl Config {
    /// Init the configuration folders if they don’t already exist
    pub fn init() -> Result<()> {
        let data_dir = data_dir();
        if !Path::exists(&data_dir) {
            fs::create_dir_all(data_dir)?;
        }

        Ok(())
    }

    /// Get current configuration
    pub fn new() -> Result<Config, confy::ConfyError> {
        info!("Reading configuration");
        confy::load(crate_name!())
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    pub fn currencies(&self) -> &Vec<String> {
        &self.currencies
    }
}

#[derive(Serialize, Deserialize)]
pub struct Apis {
    pub currency_converter_api_com: CurrencyConverterApiCom,
    pub exchange_rates_api_io: ExchangeRatesApiIo,
}

impl Default for Apis {
    fn default() -> Self {
        Apis {
            currency_converter_api_com: CurrencyConverterApiCom::default(),
            exchange_rates_api_io: ExchangeRatesApiIo::default(),
        }
    }
}

/// For <https://www.currencyconverterapi.com/>
#[derive(Serialize, Deserialize)]
pub struct CurrencyConverterApiCom {
    pub key: String,
}

impl Default for CurrencyConverterApiCom {
    fn default() -> Self {
        CurrencyConverterApiCom {
            // TODO Find a solution to distribute this. It is fine for now to provide users with a
            // quick way to start
            key: "B260A0F748A54D96B69E".to_lowercase().to_owned(),
        }
    }
}

/// For <https://exchangeratesapi.io/>
#[derive(Serialize, Deserialize)]
pub struct ExchangeRatesApiIo {
    /// API key, if any
    key: String,
}

impl Default for ExchangeRatesApiIo {
    fn default() -> Self {
        ExchangeRatesApiIo { key: String::new() }
    }
}
