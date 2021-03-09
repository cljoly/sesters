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

//! Module for price formats, like "1,000.00" or "1.000,00"

use lazy_static::lazy_static;
use log::{debug, error, trace};
use regex::Regex;

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    lazy_static! {
        static ref FR_PRICE_FORMATS: Vec<&'static str> =
            vec!["1000", "345,12", "-10000", "-10 000,87", "189.13487"];
        static ref US_PRICE_FORMATS: Vec<&'static str> =
            vec!["1000", "345.12", "10000", "10,000.87", "189.13487"];
        static ref OTHER_PRICE_FORMATS: Vec<&'static str> =
            vec!["10 00", "3.4.5,12", "-10 0 00", "10 000,87", "189.13487"];
    }

    // Test static currencies
    #[test_case(&FR, &[&FR_PRICE_FORMATS] ; "French price format")]
    #[test_case(&US, &[&US_PRICE_FORMATS] ; "American price format")]
    #[test_case(&COMMON, &[&US_PRICE_FORMATS, &OTHER_PRICE_FORMATS] ; "Common price format")]
    fn match_price_format(price_format: &PriceFormat, price_format_samples: &[&Vec<&str>]) {
        for samples in price_format_samples {
            for s in *samples {
                println!("price sample: {}", s);
                &price_format.regex;
                assert!(price_format.regex.is_match(s));
            }
        }
    }

    #[test_case("1000" => 1000. ; "Simple 1000")]
    #[test_case("100" => 100. ; "Simple 100")]
    #[test_case("10" => 10. ; "Simple 10")]
    #[test_case("1" => 1. ; "Simple 1")]
    #[test_case("100.01" => 100.01)]
    #[test_case("100,01" => 100.01 ; "Comma separator 100")]
    #[test_case("-100.01" => -100.01 ; "100.01, negative")]
    #[test_case("- 100.01" => -100.01 ; "100.01, negative spaced")]
    #[test_case("-300,03" => -300.03 ; "Comma separator 300, negative")]
    #[test_case("-20 000.02" => -20_000.02)]
    #[test_case("-40 000,04" => -40_000.04)]
    #[test_case("50 000,05" => 50_000.05)]
    #[test_case("7 00 0 00 0,0 7" => 7_000_000.07)]
    #[test_case("-5 00 0 00 0,0 5" => -5_000_000.05)]
    fn extract_number_common(price_sample: &str) -> f64 {
        COMMON.captures_iter(price_sample).get(0).unwrap().price()
    }

    #[test]
    #[should_panic]
    fn separator_duplicated_thousand_decimal() {
        PriceFormat::new(vec![',', '.'], vec!['.', ' ']);
    }
}

/// Match string representing price and converting them to number
#[derive(Debug, Clone)]
pub struct PriceFormat {
    decimal_separators: Vec<char>,
    thousand_separators: Vec<char>,
    /// Regular expression matching the given PriceFormat, inferred from
    /// previous parameters. With:
    /// 1. Capture group “sign” catching the sign of the number, if any
    /// 2. Capture group “int” catching the integer part of the number, if any
    /// 3. Capture group “dec” catching the decimal part of the number, if any
    regex: Regex,
}

impl PriceFormat {
    /// Create PriceFormat with the given separators. A regular expression
    /// conform to what regex() method guaranties is built.
    fn new(thousand_separators: Vec<char>, decimal_separators: Vec<char>) -> PriceFormat {
        fn unicode_escape(vec: &[char]) -> String {
            vec.iter()
                .map(|c| format!("{}", c.escape_unicode()))
                .collect()
        }

        // TODO Support caracters in both set of separator. Should be doable using the position and the fact that one is present at most once
        for ds in &decimal_separators {
            for ts in &thousand_separators {
                assert!(ds != ts);
            }
        }

        // Part matching number and thousand separator
        let number_and_separator;
        // Decimal separator
        let mut dec_sep = String::new(); // TODO Use with capacity

        let escaped_tsep = unicode_escape(&thousand_separators);
        {
            if escaped_tsep != "" {
                number_and_separator = [r"(\d[", &escaped_tsep, r"]*)+"].join("");
            } else {
                number_and_separator = r"(\d)+".to_string();
            }
        }

        {
            let escaped_dsep = unicode_escape(&decimal_separators);
            if escaped_dsep != "" {
                dec_sep.push_str("[");
                dec_sep.push_str(&escaped_dsep);
                dec_sep.push_str("]*");
            }
        }

        let regex = Regex::new(
            [
                "(?P<sign>(-([",
                escaped_tsep.as_str(), // Allow thousand separators between sign and price
                "])?)?)(?P<int>",
                number_and_separator.as_str(),
                ")(",
                dec_sep.as_str(),
                "(?P<dec>",
                number_and_separator.as_str(),
                "))?",
            ]
            .join("")
            .as_str(),
        )
        .unwrap(); // unwrap() is safe because we are not building invalid regexes
        debug!("PriceFormat.regex (before construction): {:?}", regex);
        PriceFormat {
            decimal_separators,
            thousand_separators,
            regex,
        }
    }

    // TODO Use an iterator here
    pub fn captures_iter(&self, txt: &str) -> Vec<PriceFormatMatch> {
        self.regex
            .captures_iter(txt)
            .filter_map(|cap: regex::Captures| -> Option<PriceFormatMatch> {
                debug!("cap: {:?}", cap);
                cap.get(0)
                    .and_then(|m: regex::Match| -> Option<PriceFormatMatch> {
                        let cap_or_empty =
                            |cap_name| cap.name(cap_name).map(|m| m.as_str()).unwrap_or("");
                        let remove_separator = |cap_name| -> String {
                            cap_or_empty(cap_name)
                                .chars()
                                .filter(|c| !self.thousand_separators.contains(c))
                                .collect()
                        };
                        let sign = remove_separator("sign");
                        let integer = remove_separator("int");
                        let dec = remove_separator("dec");
                        trace!("sign: {}, integer: {}, dec: {}", sign, integer, dec);
                        let price_str = [&sign, &integer, ".", &dec].join("");
                        let price = price_str.parse();
                        trace!("price_str: '{}' => price: '{:?}'", price_str, price);
                        match price {
                            Ok(price) => {
                                return Some(PriceFormatMatch::new(m.start(), m.end(), price))
                            }
                            Err(e) => {
                                error!("Unable to parse '{}': {}", price_str, e);
                                return None;
                            }
                        }
                    })
            })
            .collect()
    }
}

// TODO Complete this, with more than just the most common common format
// TODO Add a preferred set of formats for each currency
// TODO Test that these format are correct regular exprossions
// Price formats
lazy_static! {
    /// Common price format, should match most
    pub static ref COMMON: PriceFormat = PriceFormat::new(vec![' '], vec![',','.']);

    /// French price format
    pub static ref FR: PriceFormat = PriceFormat::new(vec![' '], vec![',','.']);

    /// US price format
    pub static ref US: PriceFormat = PriceFormat::new(vec![',', ' '], vec!['.']);
}

/// When a price format is matched in text, we return this
pub struct PriceFormatMatch {
    start: usize,
    end: usize,
    price: f64,
}

impl PriceFormatMatch {
    fn new(start: usize, end: usize, price: f64) -> PriceFormatMatch {
        PriceFormatMatch { start, end, price }
    }

    /// Start of the price matched
    pub fn start(&self) -> usize {
        self.start
    }

    /// End of the price matched
    pub fn end(&self) -> usize {
        self.end
    }

    /// Price matched, as a number
    pub fn price(&self) -> f64 {
        self.price
    }
}
