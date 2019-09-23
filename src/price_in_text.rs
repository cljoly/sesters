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

use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::{Included, Excluded};
use std::convert::TryInto;
use std::cmp::Ordering;
use log::{trace, debug};
use itertools::Itertools;
use serde_derive::Serialize;

use crate::currency::{Currency, PriceTag};
use crate::currency;
/// A module to find currency unit with amount (a **price tag**) in raw text
use regex::Regex;

#[cfg(test)]
mod tests;

// Information about a price tag match, will be used to compute the probability
// of assocation between an amount and a currency
#[derive(Debug, Clone, Serialize)]
pub struct PriceTagMatch<'c> {
    // Amount of the currency
    amount: f64,
    // Currency matching
    currency: &'c Currency,
    // Absolute distance between symbol and amount
    distance: i32,
    // Whether the order between amount and symbol is conform to currency property
    correct_symbol_order: bool,
}

/// A PriceTagMatch is better than another if the distance between amount and
/// symbol is shorter. In case where the distances are the same, the
/// PriceTagMatch which has the correct_symbol_order is better.
///
/// With this ordering, we try to make the best match the smallest: as the best
/// match would have a distance of 0 with the right order, it is the minimum of
/// the set of all PriceTagMatch.
///
/// It should be noted that we can’t say that two PriceTagMatch are equal if
/// they have different PriceTag. Thus, we only define partial ordering.
impl<'c> PartialOrd for PriceTagMatch<'c> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let o = self.distance.cmp(&other.distance);
        match o {
            Ordering::Less | Ordering::Greater => Some(o),
            Ordering::Equal => match (self.correct_symbol_order, other.correct_symbol_order) {
                (true, true) | (false, false) => if self == other { Some(Ordering::Equal) } else { None },
                (true, false) => Some(Ordering::Less),
                (false, true) => Some(Ordering::Greater),
            }
        }
    }
}

/// PartialEq consistent with the PartialOrd we defined
impl<'c> PartialEq for PriceTagMatch<'c> {
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount && *self.currency == *other.currency &&
            self.distance == other.distance && self.correct_symbol_order == self.correct_symbol_order
    }
}

impl<'c> PriceTagMatch<'c> {
    fn new(
        amount: f64,
        currency: &'c Currency,
        distance: i32,
        correct_symbol_order: bool,
    ) -> PriceTagMatch {
        PriceTagMatch {
            amount,
            correct_symbol_order,
            currency,
            distance,
        }
    }
}

impl<'c> From<PriceTagMatch<'c>> for PriceTag<'c> {
    fn from(cm: PriceTagMatch<'c>) -> Self {
        Self::new(cm.currency, cm.amount)
    }
}

// Find iso symbol with price tag in text. Before and TODO after
// Return a currency match
fn iso_for_currency<'c>(c: &'c Currency, text: &str) -> Vec<PriceTagMatch<'c>> {
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
            currency_matches.push(PriceTagMatch::new(
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
/// For price before and after the iso symbol
pub fn iso<'c>(currencies: &'c [Currency], text: &str) -> Vec<PriceTag<'c>> {
    // TODO Configure the engine to use only iso symbol, only given currencies…
    let engine = Engine::new().unwrap();
    let pricetags = engine.all_price_tags(text);
    dbg!(pricetags);
    // TODO Return pricetags computed above
    vec![]
}

/// Price tag engine, used to extract price tags in plain text
/// It proceeds in 3 steps:
/// 1. Find positions of all number (possibly with various separator)
/// 2. Find positions of currencies looked for, and for each, look for number,
///    forward and backward in a certain distance (name *window*). A
///    probability of “matching” is computed for each.
/// 3. Return N topmost matches or all of them
#[derive(Debug)]
pub struct Engine<'c> {
    options: EngineOptions<'c>,
    /// Regular expression to match prices in plain text format
    price_match: Regex,
    /// Regular expression to match currency symbol or iso in plain text format, by currency main iso
    currency_matches: HashMap<&'c str, Regex>,
}

impl<'c> Engine<'c> {
    fn new() -> Result<Engine<'c>, EngineError> {
        let e = EngineBuilder::new().fire();
        trace!("Engine::new {:?}", e);
        e
    }

    // TODO Return an iterator to lazily cut evaluation
    /// Return all price tag matches found in plain_text
    fn find<'txt>(&self, plain_text: &'txt str) -> Vec<PriceTagMatch> {
        // Record locations of price ends in price tags
        let price_locations = || {
            debug!("computing price_locations…");
            let mut price_loc_start = BTreeMap::new();
            let mut price_loc_end = BTreeMap::new();

            for cap in self.price_match.captures_iter(plain_text) {
                match cap.get(0) {
                    Some(m) => {
                        price_loc_start.insert(m.start(), m.as_str());
                        price_loc_end.insert(m.end(), m.as_str());
                    },
                    None => unreachable!(), // Normally, get(0) gives the whole pattern, which always exist
                }
            }
            debug!("price_loc_start: {:?}", price_loc_start);
            trace!("price_loc_end: {:?}", price_loc_end);
            (price_loc_start, price_loc_end)
        };

        let (price_loc_start, price_loc_end) = price_locations();

        let mut pricetag_matches = Vec::new();
        for (currency_main_iso, currency_match) in &self.currency_matches {
            debug!("Matches for {}", currency_main_iso);
            let currency = currency::existing_from_iso(currency_main_iso).unwrap();
            for cap in currency_match.captures_iter(plain_text) {
                let m = cap.get(0);
                trace!("m, before unwrapping: {:?}", m);
                let m = cap.get(0).unwrap(); // 0 is the whole pattern, always present
                let (start, end, win) = (m.start(), m.end(), self.options.window_size);
                trace!("start, end, win: {}, {}, {}", start, end, win);
                let win_before_start = if win>start { 0 } else { start - win };
                // Look backward, for the end of the price. If we were looking
                // from the start of the price, we would miss some corner
                // cases, like this one:
                //     window_size
                //   /-------------\
                //133  Lorem ipsumm USD
                trace!("before forward look, pricetag_matches: {:?}", pricetag_matches);
                for (&location, &price_str) in price_loc_end.range((Included(&win_before_start), Excluded(&start))) {
                    trace!("&location, &price_str: {:?}, {:?}", &location, &price_str);
                    let ptm = PriceTagMatch::new(
                        price_str.parse().expect("Float impossible to parse"),
                        currency,
                        ((start-location) as i32).try_into().unwrap(),
                        currency.pos() == currency::Pos::Before,
                        );
                    pricetag_matches.push(ptm);
                }
                trace!("after forward look, pricetag_matches: {:?}", pricetag_matches);
                // Idem, but with the start of the number when looking forward
                for (&location, &price_str) in price_loc_start.range((Excluded(&end), Included(&(end+win)))) {
                    // TODO Idem
                    // unimplemented!();
                }
                debug!("after forward and backward look, pricetag_matches: {:?}", pricetag_matches);
            }
        }

        pricetag_matches.sort_by_key(|ptm| (ptm.distance, ptm.correct_symbol_order));
        pricetag_matches
    }

    /// Return all price tag found in plain_text
    pub fn all_price_tags<'txt>(&self, plain_text: &'txt str) -> Vec<PriceTag> {
        self.find(plain_text).into_iter().map(|ptm| ptm.into()).collect()
    }

    /// Return the top `n` price tags 
    pub fn top_price_tags(&self, n: usize, plain_text: &str) -> Vec<PriceTag> {
        self.find(plain_text).into_iter().take(n).map(|ptm| ptm.into()).collect()
    }
}

#[derive(Debug)]
pub struct EngineOptions<'c> {
    window_size: usize,
    currencies: &'c [Currency],
    by_symbol: bool,
    by_iso: bool,
    price_format: Option<Regex>,
}

impl<'c> Default for EngineOptions<'c> {
    fn default() -> EngineOptions<'c> {
        let currencies: &'c [Currency] = &(*currency::ALL_CURRENCIES);
        EngineOptions {
            window_size: 10,
            currencies,
            by_symbol: true,
            by_iso: true,
            // TODO Try to avoid clone call here
            price_format: Some((*currency::PRICE_FORMAT_COMMON).clone()),
        }
    }
}

#[derive(Debug)]
pub struct EngineBuilder<'c>(EngineOptions<'c>);

impl<'c> EngineBuilder<'c> {
    /// Create a builder with default option, to be custumized
    fn new() -> EngineBuilder<'c> {
        EngineBuilder(Default::default())
    }

    /// Consume Builder and fire the Engine, so that it be used to match text
    fn fire(self) -> Result<Engine<'c>, EngineError> {
        // TODO Build a regex from formats in currency used in case
        // price_format is None, instead of panicking as unwrap() does here
        // TODO Return EngineError.PriceMatchRegex
        let price_match = self.0.price_format.clone().unwrap();

        let mut currency_matches = HashMap::new();
        for currency in self.0.currencies {
            let mut alternatives_slices: Vec<&[&str]> = Vec::new();
            let mut currency_match_string = String::new();
            let mut add_regex_altenatives = |new_alternatives: &'static [&'static str]| {
                trace!("add_regex_altenatives: alternatives (before): {:?}", &alternatives_slices);
                trace!("add_regex_altenatives: new_alternatives (after): {:?}", &new_alternatives);
                alternatives_slices.push(new_alternatives);
                trace!("add_regex_altenatives: alternatives (after): {:?}", &alternatives_slices);
            };
            if self.0.by_iso {
                add_regex_altenatives(currency.isos());
            }
            if self.0.by_symbol {
                add_regex_altenatives(currency.symbols());
            }
            // Escape currency symbols and iso by inserting litteral unicode in the regex
            let alternatives_unicode_escaped = alternatives_slices.into_iter().map(|a| a.into_iter().map(|s| format!("{}", s.escape_unicode())));
            for s in alternatives_unicode_escaped.flatten().intersperse("|".to_owned()) {
                currency_match_string.push_str(&s);
            }
            let currency_match_err = Regex::new(currency_match_string.as_str());
            match currency_match_err {
                Ok(currency_match) => currency_matches.insert(currency.get_main_iso(), currency_match),
                Err(err) => return Err(EngineError::CurrencyMatchRegex(err)),
            };
        }

        Ok(Engine {
            options: self.0,
            price_match,
            currency_matches,
        })
    }

    /// Set the size of the window used as the distance between a price and a
    /// currency unit when looking for price tag
    fn window(&mut self, size: usize) -> &mut EngineBuilder<'c> {
        self.0.window_size = size;
        self
    }

    /// Find price tag using the symbol of the currency, like “€” or “$”
    fn by_symbol(&mut self, yes: bool) -> &mut EngineBuilder<'c> {
        self.0.by_symbol = yes;
        self
    }

    /// Find price tag using the iso of the currency, like “EUR” or “USD”
    fn by_iso(&mut self, yes: bool) -> &mut EngineBuilder<'c> {
        self.0.by_iso = yes;
        self
    }

    /// Set the regular expression used to match prices in plain text
    /// If set to None, will be inferred from the currency list
    fn price(&mut self, format: Option<Regex>) -> &mut EngineBuilder<'c> {
        self.0.price_format = format;
        self
    }
}

/// Error that occured while building and firing the engine
#[derive(Clone, PartialEq, Debug)]
pub enum EngineError {
    /// Invalid regex for price_match
    PriceMatchRegex(regex::Error),

    /// Invalid regex for currency_match
    CurrencyMatchRegex(regex::Error),

    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}
