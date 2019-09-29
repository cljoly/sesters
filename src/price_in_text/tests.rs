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
    use super::super::{Engine, EngineBuilder};
    use crate::currency::*;

    use test_case::test_case;

    fn test_iso_usd_then_with_other(
        txt: &str,
        exp1_usd: &Option<PriceTag>,
        exp2_eur: &Option<PriceTag>,
        exp3_btc: &Option<PriceTag>,
    ) {
        let exp1_usd: &Option<&PriceTag> = &exp1_usd.as_ref();
        let exp2_eur: &Option<&PriceTag> = &exp2_eur.as_ref();
        let exp3_btc: &Option<&PriceTag> = &exp3_btc.as_ref();
        // Infer combination when there is at most one Some
        // let exp2 = exp1_usd.as_ref().or(exp2_eur.as_ref());
        let exp = match (exp1_usd, exp2_eur, exp3_btc) {
            (Some(ref e), None, None) => Some(e),
            (None, Some(ref e), None) => Some(e),
            (None, None, Some(ref e)) => Some(e),
            (None, None, None) => None,
            _ => panic!("More than one value is Some"),
        };
        let mut engine_builder = EngineBuilder::new();
        engine_builder.by_iso(true).by_symbol(false);
        fn iso_engine(engine_builder: &EngineBuilder<'static>, currencies: &'static [Currency]) -> Engine<'static> {
            engine_builder.clone().currencies(currencies).clone().fire().unwrap()
        };
        println!("===============================");
        assert_eq!(&iso_engine(&engine_builder, &[USD]).all_price_tags(txt).first(), exp1_usd);
        println!("usd ok");
        assert_eq!(&iso_engine(&engine_builder, &[EUR]).all_price_tags(txt).first(), exp2_eur);
        println!("eur ok");
        assert_eq!(&iso_engine(&engine_builder, &[BTC]).all_price_tags(txt).first(), exp3_btc);
        println!("btc ok");
        assert_eq!(&iso_engine(&engine_builder, &[USD, EUR, BTC]).all_price_tags(txt).first(), &exp.cloned());
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
        let currency_amount = Some(PriceTag::new(&EUR, 15.));
        test_iso_usd_then_with_other("EUR 15", &None, &currency_amount, &None);
    }

    #[test_case("1234 EUR")]
    #[test_case("EUR 1234")]
    #[test_case("1234%EUR" ; "Price before, percent")]
    #[test_case("EUR%1234" ; "Price after, percent")]
    #[test_case("1234EUR")]
    #[test_case("EUR1234")]
    #[test_case("1234 €" ; "Spaced symbol")]
    #[test_case("1234€" ; "Symbol, no space")]
    fn spaces(txt: &str) {
        let pt = PriceTag::new(&EUR, 1234.);
        let engine = Engine::new().unwrap();
        assert_eq!(*engine.all_price_tags(txt).first().unwrap(), pt);
    }

    #[test_case("1234,5678 EUR")]
    #[test_case("EUR 1234,5678")]
    #[test_case("1234,5678%EUR" ; "Price before, percent")]
    #[test_case("EUR%1234,5678" ; "Price after, percent")]
    #[test_case("1234,5678EUR")]
    #[test_case("EUR1234,5678")]
    #[test_case("1234,5678 €" ; "Spaced symbol")]
    #[test_case("1234,5678€" ; "Symbol, no space")]
    fn spaces_comma(txt: &str) {
        let pt = PriceTag::new(&EUR, 1234.5678);
        let engine = Engine::new().unwrap();
        assert_eq!(*engine.all_price_tags(txt).first().unwrap(), pt);
    }

    /* TODO , Separator
    #[test]
    fn iso_eur_before_float() {
        let eur = EUR;
        let currency_amount = Some(PriceTag::new(&eur, 15.11));
        test_iso_usd_then_with_other("EUR 15,11", &None, &currency_amount, &None);
    }
    */

    #[test]
    fn iso_before() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 13.));
        test_iso_usd_then_with_other("USD 13", &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_float() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 13.5));
        test_iso_usd_then_with_other("USD 13.5", &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_null_amount() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 0.));
        test_iso_usd_then_with_other(&format!("USD 0"), &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_negative_amount() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, -12.));
        test_iso_usd_then_with_other(&format!("USD -12"), &currency_amount, &None, &None);
    }

    /*
    #[test]
    fn iso_after() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 13.));
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

mod price_tag_match {
    use super::super::PriceTagMatch;
    use crate::currency::{EUR, USD, BTC};

    #[test]
    fn right_partial_order() {
        let a1 = PriceTagMatch::new(1.0, &EUR, 0, true);
        let a2 = PriceTagMatch::new(1.0, &USD, 0, true);
        let a3 = PriceTagMatch::new(3.0, &USD, 0, true);
        let a4 = PriceTagMatch::new(3.0, &EUR, 0, true);
        let a5 = PriceTagMatch::new(-1.0, &EUR, 0, true);
        let a6 = PriceTagMatch::new(-1.0, &BTC, 0, true);
        let b1 = PriceTagMatch::new(1.0, &EUR, 0, false);
        let b2 = PriceTagMatch::new(1.0, &USD, 0, false);
        let b3 = PriceTagMatch::new(3.0, &USD, 0, false);
        let b4 = PriceTagMatch::new(3.0, &EUR, 0, false);
        let b5 = PriceTagMatch::new(-1.0, &EUR, 0, false);
        let b6 = PriceTagMatch::new(-1.0, &BTC, 0, false);
        let c1 = PriceTagMatch::new(1.0, &EUR, 1, true);
        let c2 = PriceTagMatch::new(1.0, &USD, 1, true);
        let c3 = PriceTagMatch::new(3.0, &USD, 1, true);
        let c4 = PriceTagMatch::new(3.0, &EUR, 1, true);
        let c5 = PriceTagMatch::new(-1.0, &EUR, 1, true);
        let c6 = PriceTagMatch::new(-1.0, &BTC, 1, true);
        let d1 = PriceTagMatch::new(1.0, &EUR, 1, false);
        let d2 = PriceTagMatch::new(1.0, &USD, 1, false);
        let d3 = PriceTagMatch::new(3.0, &USD, 1, false);
        let d4 = PriceTagMatch::new(3.0, &EUR, 1, false);
        let d5 = PriceTagMatch::new(-1.0, &EUR, 1, false);
        let d6 = PriceTagMatch::new(-1.0, &BTC, 1, false);

        let v = vec![
            a1, a2, a3, a4, a5, a6,
            b1, b2, b3, b4, b5, b6,
            c1, c2, c3, c4, c5, c6,
            d1, d2, d3, d4, d5, d6,
        ];
        // TODO Use assert!(v.is_sorted()); once in stable
        for i in 0..v.len()-1 {
            assert!(v[i]<v[i+1] || (!(v[i] > v[i+1] && v[i] != v[i+1])));
        }
    }
}
