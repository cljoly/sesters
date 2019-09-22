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

use serde_derive::Serialize;

use crate::currency::{Currency, CurrencyAmount};
use crate::currency;
/// A module to find currency unit with amount (a **price tag**) in raw text
use regex::Regex;

mod tests;

// Information about a currency match, will be used to compute the probability
// of assocation between an amount and a currency
#[derive(Debug, PartialEq, Clone, Serialize)]
struct CurrencyMatch<'c> {
    // Amount of the currency
    amount: f64,
    // Currency matching
    currency: &'c Currency,
    // Absolute distance between symbol and amount
    distance: i32,
    // Whether the order between amount and symbol is conform to currency property
    correct_symbol_order: bool,
}

impl<'c> CurrencyMatch<'c> {
    fn new(
        amount: f64,
        currency: &'c Currency,
        distance: i32,
        correct_symbol_order: bool,
    ) -> CurrencyMatch {
        CurrencyMatch {
            amount,
            correct_symbol_order,
            currency,
            distance,
        }
    }
}

impl<'c> From<CurrencyMatch<'c>> for CurrencyAmount<'c> {
    fn from(cm: CurrencyMatch<'c>) -> Self {
        Self::new(cm.currency, cm.amount)
    }
}

// Find iso symbol with price tag in text. Before and TODO after
// Return a currency match
fn iso_for_currency<'c>(c: &'c Currency, text: &str) -> Vec<CurrencyMatch<'c>> {
    let mut formatted_regexes = Vec::new();
    for iso in c.isos() {
        // TODO Use lazy_static here
        formatted_regexes.push(format!(
            r"(?x)
            (?P<sym_before>{})
            (?P<length_before>.*?)
            (?P<amount>-?\d+(\.\d*)?)
            ",
            iso
        ));
    }
    let mut currency_matches = Vec::new();
    for formatted_regex in formatted_regexes {
        let r = Regex::new(formatted_regex.as_str()).unwrap();
        for cap in r.captures_iter(text) {
            // TODO Implement distance, symbol order
            // Unwrap should not be an issue as we only have numbers and a dot
            currency_matches.push(CurrencyMatch::new(
                cap.name("amount")
                    .unwrap()
                    .as_str()
                    .parse()
                    .expect("Float impossible to parse"),
                c,
                1,
                true,
            ));
        }
    }
    currency_matches
}

/// Find price with iso symbol for all given currency
/// For price before and TODO after the iso symbol
pub fn iso<'c>(currencies: &'c [Currency], text: &str) -> Vec<CurrencyAmount<'c>> {
    let matches_iterator = currencies.iter().map(|c| iso_for_currency(c, text));
    matches_iterator.flatten().map(|cm| cm.into()).collect()
}

/// Price tag engine, used to extract price tags in plain text
/// It proceeds in 3 steps:
/// 1. Find positions of all number (possibly with various separator)
/// 2. Find positions of currencies looked for, and for each, look for number, forward and backward in a certain distance (name *window*). A probability of “matching” is computed for each.
/// 3. Return N topmost matches
pub struct Engine<'c> {
    options: EngineOptions<'c>,
}

impl<'c> Engine<'c> {
    fn new() -> Engine<'c> {
        EngineBuilder::new().fire()
    }

    fn all_price_tag() -> Vec<CurrencyAmount<'c>> {
        unreachable!();
    }
}

pub struct EngineOptions<'c> {
    window_size: usize,
    currencies: Vec<&'c Currency>
}

impl Default for EngineOptions<'_> {
    fn default() -> EngineOptions<'static> {
        EngineOptions {
            window_size: 10,
            currencies: vec![], // TODO Use ALL_CURRENCIES instead
        }
    }
}

pub struct EngineBuilder<'c>(EngineOptions<'c>);

impl<'c> EngineBuilder<'c> {
    /// Create a builder with default option, to be custumized
    fn new() -> EngineBuilder<'static> {
        EngineBuilder(Default::default())
    }

    /// Consume Builder and fire the Engine, so that it can match text
    fn fire(self) -> Engine<'c> {
        unimplemented!();
    }

    fn window(&mut self, size: usize) -> &mut EngineBuilder<'c> {
        self.0.window_size = size;
        self
    }
}
