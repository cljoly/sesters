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

use crate::currency::Currency;
/// A module to find currency unit with amount in raw text
use regex::Regex;

mod tests;
/// An association between currency & amount, TODO with a position
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CurrencyAmount<'c> {
    currency: &'c Currency,
    amount: f64,
    // /// Position of the currency indicator against amount
    // position: Pos,
}

impl<'c> CurrencyAmount<'c> {
    fn new(currency: &'c Currency, amount: f64) -> Self {
        Self { currency, amount }
    }
    fn from_currency_match(cm: CurrencyMatch<'c>) -> Self {
        Self::new(cm.currency, cm.amount)
    }
}

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

// Find iso symbol with price tag in text. Before and TODO after
// Return a currency match
fn iso_for_currency<'c>(c: &'c Currency, text: &str) -> Option<CurrencyMatch<'c>> {
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
    for formatted_regex in formatted_regexes {
        let r = Regex::new(formatted_regex.as_str()).unwrap();
        for cap in r.captures_iter(text) {
            // TODO Implement distance, symbol order
            // Unwrap should not be an issue as we only have numbers and a dot
            println!(
                "---------------- Some {:?} ----------- {:?}",
                cap,
                cap.name("amount").unwrap().as_str()
            );
            return Some(CurrencyMatch::new(
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
    println!("---------------- None");
    None
}

// Find price with iso symbol for all given currency
// For price before and TODO after the iso symbol
fn iso<'c>(currencies: &'c Vec<Currency>, text: &str) -> Option<CurrencyAmount<'c>> {
    let mut cmatch_option = None;
    for c in currencies {
        if let Some(cmatch) = iso_for_currency(c, text) {
            cmatch_option = Some(cmatch);
            break;
        }
    }
    cmatch_option.map(|cm| CurrencyAmount::from_currency_match(cm))
}
