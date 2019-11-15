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

use log::info;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration file
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Version of the config file
    pub version: u8,
    /// Currencies to convert to
    pub currencies: Vec<String>,
    /// Path of the database (directory). Please note that ~ is not expanded
    pub db_path: PathBuf,
    /// API key per provider
    pub providers_keys: ProvidersKeys,
}

#[derive(Serialize, Deserialize)]
pub struct ProvidersKeys {
    pub currency_converter_api_com: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut db_path = dirs::data_dir().unwrap();
        db_path.push("sesters/db");
        Config {
            version: 0,
            currencies: vec!["EUR".to_string(), "USD".to_string(), "GBP".to_string()],
            db_path,
            providers_keys: ProvidersKeys {
                currency_converter_api_com: None,
            },
        }
    }
}

static APP_NAME: &'static str = "sesters";

impl Config {
    /// Get current configuration. Doesn’t handle errors, panics
    pub fn get() -> Config {
        info!("Reading configuration");
        confy::load(APP_NAME).unwrap()
    }

    /// Change current configuration. Doesn’t handle errors, panics
    pub fn set(c: Config) {
        info!("Writing configuration");
        confy::store(APP_NAME, c).unwrap();
    }
}
