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

#[cfg(test)]
const SHORT_TXT: &str = "short";
const LONG_TXT: &str = "some quite loooooooooooooooooooooooooooong text";

mod iso {
    use super::super::iso;
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
        assert_eq!(&iso(&vec![USD], txt), exp1_usd);
        println!("usd ok");
        assert_eq!(&iso(&vec![EUR], txt), exp2_eur);
        println!("eur ok");
        assert_eq!(&iso(&vec![BTC], txt), exp3_btc);
        println!("btc ok");
        assert_eq!(&iso(&vec![USD, EUR, BTC], txt), &exp.cloned());
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
        let currency_amount = Some(CurrencyAmount::new(&EUR, 15.));
        test_iso_usd_then_with_other("EUR 15", &None, &currency_amount, &None);
    }

    /* TODO , Separator
    #[test]
    fn iso_eur_before_float() {
        let eur = EUR;
        let currency_amount = Some(CurrencyAmount::new(&eur, 15.11));
        test_iso_usd_then_with_other("EUR 15,11", &None, &currency_amount, &None);
    }
    */

    #[test]
    fn iso_before() {
        let usd = USD;
        let currency_amount = Some(CurrencyAmount::new(&usd, 13.));
        test_iso_usd_then_with_other("USD 13", &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_float() {
        let usd = USD;
        let currency_amount = Some(CurrencyAmount::new(&usd, 13.5));
        test_iso_usd_then_with_other("USD 13.5", &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_null_amount() {
        let usd = USD;
        let currency_amount = Some(CurrencyAmount::new(&usd, 0.));
        test_iso_usd_then_with_other(&format!("USD 0"), &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_negative_amount() {
        let usd = USD;
        let currency_amount = Some(CurrencyAmount::new(&usd, -12.));
        test_iso_usd_then_with_other(&format!("USD -12"), &currency_amount, &None, &None);
    }

    /*
    #[test]
    fn iso_after() {
        let usd = USD;
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
