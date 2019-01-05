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

/// Position of a symbol against an amount
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Pos {
    Before,
    After,
}

/// Represent a currency like US Dollar or Euro, with its symbols
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Currency {
    /// Symbols, like ₿, ฿ or Ƀ for Bitcoin
    symbols: Vec<String>,
    /// ISO4217-ish symbol, like BTC or XBT for Bitcoin
    isos: Vec<String>,
    /// Human name(s)
    names: Vec<String>,
    /// Position to display symbols
    pos: Pos,
}

impl Currency {
    pub fn isos(&self) -> &Vec<String> {
        &self.isos
    }

    pub fn names(&self) -> &Vec<String> {
        &self.names
    }

    pub fn symbols(&self) -> &Vec<String> {
        &self.symbols
    }

    /// Constructor, copies the &str given
    pub fn new(symbols: Vec<String>, isos: Vec<String>, names: Vec<String>, pos: Pos) -> Currency {
        Currency {
            symbols,
            isos,
            names,
            pos,
        }
    }

    /// Simplified constructor, copies the &str given
    pub fn from(symbols: Vec<&str>, isos: Vec<&str>, names: Vec<&str>, pos: Pos) -> Currency {
        Self::new(
            symbols.into_iter().map(|s| String::from(s)).collect(),
            isos.into_iter().map(|s| String::from(s)).collect(),
            names.into_iter().map(|s| String::from(s)).collect(),
            pos)
    }

    /// Simplified construcor for currency with only one name, iso, symbol.
    pub fn from_simple(symbol: &str, iso: &str, name: &str, pos: Pos) -> Currency {
        Currency::new(
            vec![symbol.to_string()],
            vec![iso.to_string()],
            vec![name.to_string()],
            pos,
        )
    }
}

/// Some common currency
/// Symbols and ISO are take form Wikipedia
// TODO Use laziness to build only one of each currency

/// https://en.wikipedia.org/wiki/Bitcoin
pub fn btc() -> Currency {
    Currency::from(
        vec!["₿", "฿", "Ƀ"],
        vec!["BTC", "XBT"],
        vec!["Bitcoin"],
        Pos::After,
    )
}

/// https://en.wikipedia.org/wiki/United_States_dollar
pub fn usd() -> Currency {
    Currency::from_simple("$", "USD", "United States dollar", Pos::Before)
}

/// https://en.wikipedia.org/wiki/Euro
pub fn eur() -> Currency {
    Currency::from_simple("€", "EUR", "Euro", Pos::After)
}
