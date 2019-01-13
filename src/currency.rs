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

use serde_derive::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    // Test static currencies
    #[test]
    fn static_currency() {
        assert!(EUR.check());
        assert!(USD.check());
        assert!(BTC.check());
    }
}

/// Position of a symbol against an amount
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub enum Pos {
    Before,
    After,
}

impl Default for Pos {
    fn default() -> Pos {
        Pos::After
    }
}

/// Represent a currency like US Dollar or Euro, with its symbols
// TODO Improve serialization/deserialization
#[derive(Debug, Default, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Currency {
    /// Symbols, like ₿, ฿ or Ƀ for Bitcoin. Slice must not be empty
    #[serde(skip)]
    symbols: &'static [&'static str],
    /// ISO4217-ish symbol, like BTC or XBT for Bitcoin. Slice must not be empty
    #[serde(skip)]
    isos: &'static [&'static str],
    /// Human name(s). Slice must not be empty
    #[serde(skip)]
    names: &'static [&'static str],
    /// Position to display symbols
    pos: Pos,
}

impl Currency {
    pub fn isos(&self) -> &'static [&'static str] {
        &self.isos
    }

    /// Main iso symbol for a currency, USD for instance
    pub fn get_main_iso(&self) -> &str {
        return &self.isos[0];
    }

    pub fn names(&self) -> &'static [&'static str] {
        &self.names
    }

    pub fn symbols(&self) -> &'static [&'static str] {
        &self.symbols
    }

    /// Constructor, copies the &str given. Panics if vectors are empty TODO Use Result type instead
    pub fn new(
        symbols: &'static [&'static str],
        isos: &'static [&'static str],
        names: &'static [&'static str],
        pos: Pos,
    ) -> Currency {
        let c = Currency {
            symbols,
            isos,
            names,
            pos,
        };
        assert!(c.check());
        c
    }

    /// Check if a currency is conform to the constraints listed in the definition of the structure
    pub fn check(&self) -> bool {
        self.symbols.len() >= 1 && self.isos.len() >= 1 && self.names.len() >= 1
    }
}

/// Some common currency
/// Symbols and ISO are from Wikipedia

/// https://en.wikipedia.org/wiki/Bitcoin
// TODO Use Currency::new once const fn is in stable
pub const BTC: Currency = Currency {
    symbols: &["₿", "฿", "Ƀ"],
    isos: &["BTC", "XBT"],
    names: &["Bitcoin"],
    pos: Pos::After,
};

/// https://en.wikipedia.org/wiki/United_States_dollar
pub const USD: Currency = Currency {
    symbols: &["$"],
    isos: &["USD"],
    names: &["United States dollar"],
    pos: Pos::Before,
};

/// https://en.wikipedia.org/wiki/Euro
pub const EUR: Currency = Currency {
    symbols: &["€"],
    isos: &["EUR"],
    names: &["Euro"],
    pos: Pos::After,
};

/// Get an existing currency from ISO code
pub fn existing_from_iso(code: &str) -> Option<&'static Currency> {
    match code {
        "EUR" => Some(&EUR),
        "BTC" => Some(&BTC),
        "USD" => Some(&USD),
        _ => None,
    }
}
