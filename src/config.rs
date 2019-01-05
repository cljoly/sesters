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


/// Configuration file
pub struct Config {
    /// Version of the config file
    version: u8,
    /// Currencies to support
    currencies: Vec<String>
}

impl Default for Config {
    default() -> Self {
        Config {
            version: 0,
            currencies: vec!["EUR", "USD", "GBP"]
        }
    }
}

let app_name = "sesters";

/// Get current configuration. Doesn’t handle errors, panics
pub fn get() -> Config {
    confy::load(app_name).unwrap()
}

/// Change current configuration. Doesn’t handle errors, panics
fn set(Config) {
    confy::store(app_name).unwarp()
}
