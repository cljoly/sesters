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

#[cfg(test)]
mod tests {
    const short_txt: &'static str = "short";
    const long_txt: &'static str = "some quite loooooooooooooooooooooooooooong text";

    mod iso {
        use super::super::iso;
        use super::super::CurrencyAmount;
        use crate::currency::*;

        fn test_iso_usd_then_with_other(
            txt: &str,
            exp1_usd: &Option<CurrencyAmount>,
            exp2_eur: &Option<CurrencyAmount>,
            exp3_btc: &Option<CurrencyAmount>,
        ) {
            // Infer combination when there is at most one Some
            // let exp2 = exp1_usd.as_ref().or(exp2_eur.as_ref());
            let exp = match (exp1_usd, exp2_eur, exp3_btc) {
                (Some(ref e), None, None) => Some(e),
                (None, Some(ref e), None) => Some(e),
                (None, None, Some(ref e)) => Some(e),
                (None, None, None) => None,
                _ => panic!("More than one value is Some"),
            };
            println!("===============================");
            assert_eq!(&iso(&vec![usd()], txt), exp1_usd);
            println!("usd ok");
            assert_eq!(&iso(&vec![eur()], txt), exp2_eur);
            println!("eur ok");
            assert_eq!(&iso(&vec![btc()], txt), exp3_btc);
            println!("btc ok");
            assert_eq!(&iso(&vec![usd(), eur(), btc()], txt), &exp.cloned());
            println!("usd, eur, btc ok");
            println!("===============================");
        }

        #[test]
        fn iso_empty_string() {
            test_iso_usd_then_with_other(&format!(""), &None, &None, &None);
        }

        #[test]
        fn iso_none() {
            test_iso_usd_then_with_other(&format!("13"), &None, &None, &None);
        }

        #[test]
        fn iso_none_before() {
            test_iso_usd_then_with_other(&format!("OOO 13"), &None, &None, &None);
        }

        #[test]
        fn iso_none_after() {
            test_iso_usd_then_with_other(&format!("13 OOO"), &None, &None, &None);
        }

        #[test]
        fn iso_eur_before() {
            let eur = eur();
            let currency_amount = Some(CurrencyAmount::new(&eur, 15.));
            test_iso_usd_then_with_other("EUR 15", &None, &currency_amount, &None);
        }

        /* TODO , Separator
        #[test]
        fn iso_eur_before_float() {
            let eur = eur();
            let currency_amount = Some(CurrencyAmount::new(&eur, 15.11));
            test_iso_usd_then_with_other("EUR 15,11", &None, &currency_amount, &None);
        }
        */

        #[test]
        fn iso_before() {
            let usd = usd();
            let currency_amount = Some(CurrencyAmount::new(&usd, 13.));
            test_iso_usd_then_with_other("USD 13", &currency_amount, &None, &None);
        }

        #[test]
        fn iso_before_float() {
            let usd = usd();
            let currency_amount = Some(CurrencyAmount::new(&usd, 13.5));
            test_iso_usd_then_with_other("USD 13.5", &currency_amount, &None, &None);
        }

        #[test]
        fn iso_before_null_amount() {
            let usd = usd();
            let currency_amount = Some(CurrencyAmount::new(&usd, 0.));
            test_iso_usd_then_with_other(&format!("USD 0"), &currency_amount, &None, &None);
        }

        #[test]
        fn iso_before_negative_amount() {
            let usd = usd();
            let currency_amount = Some(CurrencyAmount::new(&usd, -12.));
            test_iso_usd_then_with_other(&format!("USD -12"), &currency_amount, &None, &None);
        }

        /*
        #[test]
        fn iso_after() {
            let usd = usd();
            let currency_amount = Some(CurrencyAmount::new(&usd, 13.));
            test_iso_usd_then_with_other(
                &format!("13 USD"),
                &currency_amount,
                &None,
                &currency_amount,
            );
        }
        */

        /* TODO Other cases
        #[test]
        fn iso_before_long() {
            test_iso_usd_then_with_other(&format!("USD some quite looooooong text 13"), None, None);
        }

        #[test]
        fn iso_after_long() {
            test_iso_usd_then_with_other(&format!("13 some quite loooong USD"), None, None);
        }

        #[test]
        fn iso_before_words() {
            test_iso_usd_then_with_other(&format!("USD 13"), None, None);
        }

        #[test]
        fn iso_after_words() {
            test_iso_usd_then_with_other(&format!("USD 13"), None, None);
        }
        */
    }
}

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
