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

//! Structs related to rate and rate storage

use chrono::offset::{Local as LocalTime};
use chrono::prelude::*;
use kv::bincode::Bincode;
use kv::{Bucket, Config as KvConfig, Serde, Store, Txn, ValueBuf};
use log::{debug, error, trace, warn};
use serde_derive::{Deserialize, Serialize};

use crate::currency;
use crate::currency::{Currency};
use crate::rate::Rate;

#[cfg(test)]
mod tests {

    use chrono::Duration;

    use super::*;
    use crate::currency::{BTC, CHF, EUR, USD};

    // Ensure nothing is lost when converting to RateKey
    #[test]
    fn ratekey_and_back() {
        let fut = Local.ymd(2020, 9, 30).and_hms(17, 38, 49);
        let horrible_provider_name =
            String::from("somelongand://strange_name.1230471-02347.TEST.com/@ñé:--:123");
        let rk1 = RateKey::new(&EUR, &EUR, "TEST1", &fut);
        let rk2 = RateKey::new(&USD, &BTC, &horrible_provider_name, &fut);
        assert_eq!(rk1.data(), (&EUR, &EUR, String::from("TEST1"), fut.clone()));
        assert_eq!(
            rk2.data(),
            (&USD, &BTC, horrible_provider_name, fut.clone())
        )
    }

    // Check consistency between
    #[test]
    fn partialratekey_consistency() {
        let src = &EUR;
        let dst = &CHF;
        let provider = "TEST1";
        let fut = Local::now() + Duration::days(11);
        let key = RateKey::new(src, dst, provider, &fut);
        let key_str = key.0.as_str();

        let partial3 = PartialRateKey::src_dst_provider(src, dst, provider);
        let len3 = partial3.0.len();
        assert_eq!(partial3.0, key_str[..len3]);

        let partial2 = PartialRateKey::src_dst(src, dst);
        let len2 = partial2.0.len();
        assert_eq!(partial2.0, key_str[..len2]);

        let partial1 = PartialRateKey::src(src);
        let len1 = partial1.0.len();
        assert_eq!(partial1.0, key_str[..len1]);
    }

    // Check is_compatible_with
    #[test]
    fn partialratekey_compatible_with_method() {
        let src = &EUR;
        let dst = &CHF;
        let provider = "TEST1";
        let fut = Local::now() + Duration::days(11);
        let key = RateKey::new(src, dst, provider, &fut);

        let partial3 = PartialRateKey::src_dst_provider(src, dst, provider);
        assert!(partial3.is_compatible_with(&key));

        let partial2 = PartialRateKey::src_dst(src, dst);
        assert!(partial2.is_compatible_with(&key));

        let partial1 = PartialRateKey::src(src);
        assert!(partial1.is_compatible_with(&key));
    }

    // Ensure nothing is lost when converting to RateInternal
    #[test]
    fn rateinternal_and_back() {
        let now = Local::now();
        let fut = now + Duration::hours(3);
        // Precision does not go beyond second for expriration, ceil fut to seconds
        let fut_ceil = fut - Duration::nanoseconds(fut.timestamp_subsec_nanos() as i64);
        let now_plus_1s_ceil =
            now - Duration::nanoseconds(fut.timestamp_subsec_nanos() as i64) + Duration::seconds(1);
        dbg!((fut, fut_ceil));
        let r1 = Rate::new(
            &EUR,
            &CHF,
            now,
            9.12,
            "RateInternal".to_string(),
            Some(fut_ceil),
        );
        let ri1: RateInternal = r1.clone().into();
        let r1_back: Rate = ri1.into();
        assert_eq!(r1, r1_back);

        let r2 = Rate::new(
            &BTC,
            &USD,
            now,
            9.12,
            "RateInternalNoCache".to_string(),
            Some(now_plus_1s_ceil),
        );
        assert_ne!(r1, r2);
        assert_ne!(r1_back, r2);
        let r3 = Rate::new(
            &BTC,
            &USD,
            now,
            9.12,
            "RateInternal".to_string(),
            Some(fut_ceil),
        );
        assert_ne!(r1, r3);
        assert_ne!(r1_back, r3);
        let ri2: RateInternal = r2.clone().into();
        let r2_back: Rate = ri2.into();
        assert_eq!(r2, r2_back);
    }

    // Ensure we cannot store rate that don’t have cache limit date
    #[test]
    #[should_panic]
    fn rateinternal_cache_none() {
        let now = Local::now();
        let r2 = Rate::new(
            &BTC,
            &USD,
            now,
            9.1,
            "RateInternalNoCache".to_string(),
            None,
        );
        let _ri2: RateInternal = r2.clone().into();
    }


    // Some benchmark to compare fonction extraction from key in db
    #[cfg(all(feature = "unstable", test))]
    mod bench_key_extract_db {
        use chrono::offset::{Local as LocalTime};
        use chrono::prelude::*;
        use lazy_static::lazy_static;
        use regex::Regex;

        use super::super::RateKey;
        use crate::currency;
        use crate::currency::Currency;

        extern crate test;
        use test::Bencher;

        macro_rules! SEPARATOR {
            () => { '\0' };
            ("str") => { "\0" };
        }

        impl RateKey {
            fn data_with_regex(
                &self,
                ) -> (
                    &'static Currency,
                    &'static Currency,
                    String,
                    DateTime<LocalTime>,
                    ) {
                    lazy_static! {
                        static ref KEY: Regex =
                            dbg!(Regex::new(r"^(?P<src>[A-Z]{3}):(?P<dst>[A-Z]{3}):(?P<prov>.+):(?P<until>\d+)$"))
                            .unwrap();
                    }
                    let cap = KEY.captures(&self.0).unwrap();
                    let src_iso = cap.name("src").unwrap().as_str();
                    let dst_iso = cap.name("dst").unwrap().as_str();
                    let provider_str = cap.name("prov").unwrap().as_str();
                    let timestamp_str = cap.name("until").unwrap().as_str();
                    let src = currency::existing_from_iso(src_iso).unwrap();
                    let dst = currency::existing_from_iso(dst_iso).unwrap();
                    let provider = String::from(provider_str);
                    let cache_until = Local.timestamp(timestamp_str.parse().unwrap(), 0);
                    (src, dst, provider, cache_until)
                }

                fn new_with_regex(
                    src: &Currency,
                    dst: &Currency,
                    provider: &str,
                    cache_until: &DateTime<LocalTime>,
                ) -> RateKey {
                    RateKey(format!(
                        "{}:{}:{}:{}",
                        src.get_main_iso(),
                        dst.get_main_iso(),
                        provider,
                        cache_until.timestamp()
                    ))
                }

                fn new_with_null_split(
                    src: &Currency,
                    dst: &Currency,
                    provider: &str,
                    cache_until: &DateTime<LocalTime>,
                ) -> RateKey {
                    RateKey(format!(
                        concat!("{}", SEPARATOR!(), "{}", SEPARATOR!(), "{}", SEPARATOR!(), "{}"),
                        src.get_main_iso(),
                        dst.get_main_iso(),
                        provider,
                        cache_until.timestamp(),
                    ))
                }

                fn new_with_null_split_array_join(
                    src: &Currency,
                    dst: &Currency,
                    provider: &str,
                    cache_until: &DateTime<LocalTime>,
                ) -> RateKey {
                    RateKey(
                        [src.get_main_iso(), dst.get_main_iso(), provider, cache_until.timestamp().to_string().as_str()].join(SEPARATOR!("str")),
                    )
                }

                fn data_with_null_split(
                    &self,
                    ) -> (
                        &'static Currency,
                        &'static Currency,
                        String,
                        DateTime<LocalTime>,
                        ) {
                        let mut split = self.0.split(SEPARATOR!());
                        let src_iso = split.next().unwrap();
                        let dst_iso = split.next().unwrap();
                        let provider_str = split.next().unwrap();
                        let timestamp_str = split.next().unwrap();
                        let src = currency::existing_from_iso(src_iso).unwrap();
                        let dst = currency::existing_from_iso(dst_iso).unwrap();
                        let provider = String::from(provider_str);
                        let cache_until = Local.timestamp(timestamp_str.parse().unwrap(), 0);
                        (src, dst, provider, cache_until)
                    }

        }

        fn key_data() -> (&'static Currency, &'static Currency, &'static str, DateTime<Local>) {
            (&currency::EUR, &currency::USD, "dummy_provider", Local::now())
        }

        #[bench]
        fn bench_extract_data_regex(b: &mut Bencher) {
            let kd = key_data();
            let rk = RateKey::new_with_regex(kd.0, kd.1, kd.2, &kd.3);
            b.iter(|| rk.data_with_regex());
        }

        #[bench]
        fn bench_extract_data_null_split(b: &mut Bencher) {
            let kd = key_data();
            let rk = RateKey::new_with_null_split(kd.0, kd.1, kd.2, &kd.3);
            b.iter(|| rk.data_with_null_split());
        }

        #[bench]
        fn bench_extract_data_current_impl(b: &mut Bencher) {
            let kd = key_data();
            let rk = RateKey::new(kd.0, kd.1, kd.2, &kd.3);
            b.iter(|| rk.data());
        }


        #[bench]
        fn bench_create_data_regex(b: &mut Bencher) {
            let kd = key_data();
            b.iter(|| RateKey::new_with_regex(kd.0, kd.1, kd.2, &kd.3))
        }

        #[bench]
        fn bench_create_data_null_split(b: &mut Bencher) {
            let kd = key_data();
            b.iter(|| RateKey::new_with_null_split(kd.0, kd.1, kd.2, &kd.3))
        }

        #[bench]
        fn bench_create_data_null_split_array_join(b: &mut Bencher) {
            let kd = key_data();
            b.iter(|| RateKey::new_with_null_split_array_join(kd.0, kd.1, kd.2, &kd.3))
        }

        #[bench]
        fn bench_create_data_current_impl(b: &mut Bencher) {
            let kd = key_data();
            b.iter(|| RateKey::new(kd.0, kd.1, kd.2, &kd.3))
        }

    }
}

// The key to find a rate in the database
#[derive(Clone, Debug, PartialOrd, PartialEq)]
struct RateKey(String);

/// Separator in RateKey
macro_rules! SEPARATOR {
    () => { '\0' };
    ("str") => { "\0" };
}

impl RateKey {
    // New rate key
    fn new(
        src: &Currency,
        dst: &Currency,
        provider: &str,
        cache_until: &DateTime<LocalTime>,
    ) -> RateKey {
        RateKey(
            [src.get_main_iso(), dst.get_main_iso(), provider, cache_until.timestamp().to_string().as_str()].join(SEPARATOR!("str")),
        )
    }

    // Get data stored in the rate key. Panics if a rate is malformed.
    fn data(
        &self,
    ) -> (
        &'static Currency,
        &'static Currency,
        String,
        DateTime<LocalTime>,
    ) {
        let mut split = self.0.split(SEPARATOR!());
        let src_iso = split.next().unwrap();
        let dst_iso = split.next().unwrap();
        let provider_str = split.next().unwrap();
        let timestamp_str = split.next().unwrap();
        let src = currency::existing_from_iso(src_iso).unwrap();
        let dst = currency::existing_from_iso(dst_iso).unwrap();
        let provider = String::from(provider_str);
        let cache_until = Local.timestamp(timestamp_str.parse().unwrap(), 0);
        (src, dst, provider, cache_until)
    }
}

impl From<&[u8]> for RateKey {
    fn from(val: &[u8]) -> RateKey {
        // XXX Was found by reading the code of crate kv, need to find something
        // more robust
        trace!("Converting &[u8] into RateKey: {:?}", val);
        use encoding::all::ISO_8859_1;
        use encoding::{DecoderTrap, Encoding};
        let raw_key = ISO_8859_1.decode(val, DecoderTrap::Ignore);
        if let Ok(rk) = &raw_key {
            trace!("After conversion, got: {}", &rk);
        } else {
            error!("Error converting from &[u8] {:?}", val);
        }
        RateKey(raw_key.unwrap())
    }
}

// Data of a rate (value of the key in the database)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RateVal(DateTime<LocalTime>, f64);

// Internal representation for storage of a rate: in the database, a rate is
// stored partly on the key and partly on the value
struct RateInternal {
    key: RateKey,
    value: RateVal,
}

impl<'c> From<Rate<'c>> for RateInternal {
    fn from(val: Rate) -> RateInternal {
        // TODO Remove unwrap and use TryFrom
        RateInternal::new(
            RateKey::new(
                val.src(),
                val.dst(),
                val.provider(),
                val.cache_until().as_ref().unwrap(),
            ),
            RateVal(*val.date(), val.rate()),
        )
    }
}

impl From<RateInternal> for Rate<'static> {
    fn from(ri: RateInternal) -> Rate<'static> {
        let (src, dst, provider, cache_until) = ri.key.data();
        let date = ri.value.0;
        let rate = ri.value.1;
        Rate::new(src, dst, date, rate, provider, Some(cache_until))
    }
}

impl RateInternal {
    fn new(key: RateKey, val: RateVal) -> RateInternal {
        RateInternal { key, value: val }
    }
}

impl std::convert::AsRef<[u8]> for RateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref()
    }
}

pub const BUCKET_NAME: &str = "rate";

/// To query over partial key
#[derive(Debug)]
struct PartialRateKey(String);

impl PartialRateKey {
    /// Create a partial key before all key with the given src currency, dst
    /// currency and provider
    fn src_dst_provider(src: &Currency, dst: &Currency, provider: &str) -> Self {
        PartialRateKey([src.get_main_iso(), dst.get_main_iso(), provider].join(SEPARATOR!("str")))
    }

    /// Create a partial key before all key with the given src currency and dst
    /// currency
    fn src_dst(src: &Currency, dst: &Currency) -> Self {
        PartialRateKey([src.get_main_iso(), dst.get_main_iso()].join(SEPARATOR!("str")))
    }

    /// Create a partial key before all key with the given src currency and dst
    /// currency
    fn src(src: &Currency) -> Self {
        PartialRateKey(format!("{}{}", src, SEPARATOR!()))
    }

    /// Treat as key
    fn to_key(&self) -> RateKey {
        RateKey(self.0.clone())
    }

    /// Check whether this partial key is the beginning of another
    fn is_compatible_with(&self, key: &RateKey) -> bool {
        let r = key.0.starts_with(&self.0);
        trace!("Is {:?} compatible with {:?}?: {}", self, key, r);
        r
    }
}

impl super::Db {
    /// Retrieve rates from a currency to another. First member of the tuple
    /// contains up-to-date rates and the second member outdated ones
    pub fn get_rates<'c>(
        &self,
        txn: &Txn,
        store: &Store,
        src: &'c Currency,
        dst: &Currency,
        provider: &str,
    ) -> (Vec<Rate<'c>>, Vec<Rate<'c>>) {
        trace!("get_rates({}, {}, {:?})", src, dst, provider);
        // Hard code this to limit storage overhead
        if src == dst {
            warn!("Same source and destination currency, don’t store");
            return (vec![Rate::parity(src)], vec![]);
        }
        // TODO Have this by argument
        let bucket = self.bucket_rate(store);
        // TODO Return None only when a key is not found, not for any error
        let partial = PartialRateKey::src_dst_provider(src, dst, provider);
        let partial_key = partial.to_key();
        trace!("partial_key: {:?}", partial_key);
        let cursor = txn.read_cursor(&bucket.as_bucket());
        let mut cursor = cursor.unwrap();

        let now = Local::now();

        // TODO Place this in a function
        trace!("Iterating over key compatible with {:?}", partial_key);
        let (uptodate_rates, outdated_rates): (Vec<Rate>, Vec<Rate>) =
            cursor.iter_from(&partial_key)
            .take_while(|(k, _)| {
                // Take key that are starting with right thing, if any
                trace!("Take {:?}?", k);
                partial.is_compatible_with(k)
            })
        .map(|(rk, rv_buf)| {
            trace!("key: {:?}, value_buf: {:?}", rk, rv_buf);
            let rv = rv_buf.inner().unwrap().to_serde();
            trace!("value: {:?}", rv);
            RateInternal::new(rk, rv).into()
        })
        .partition(|rate: &Rate| {
            if let Some(dt) = rate.cache_until() { return dt > &now }
            // Consider outdated the one without cache_limit (shouldn’t be in the database in the fist place)
            else { return false }
            })
        ;
        trace!("uptodate_rates: {:?}", uptodate_rates);
        trace!("outdated_rates: {:?}", outdated_rates);
        (uptodate_rates, outdated_rates)
    }

    /// Set rate from a currency to another
    // TODO Return error type
    pub fn set_rate<'t, 'd>(
        &'d self,
        txn: &mut Txn<'t>,
        bucket: &RateBucket<'t>,
        rate: Rate,
    ) where
        'd: 't,
    {
        if rate.src() == rate.dst() {
            warn!("Same  source and destination currency, don’t store");
            return;
        }
        let ri: RateInternal = rate.into();
        txn.set(
            bucket.as_bucket(),
            ri.key,
            Bincode::to_value_buf(ri.value).unwrap(),
        )
        .unwrap();
    }

    /// Remove rate from a currency to another
    // TODO Return error type
    pub fn del_rate<'t, 'd>(
        &'d self,
        txn: &mut Txn<'t>,
        bucket: &RateBucket<'t>,
        rate: Rate,
    ) where
        'd: 't,
    {
        trace!("Remove rate {:?} from databse.", rate);
        let ri: RateInternal = rate.into();
        txn.del(
            bucket.as_bucket(),
            ri.key,
        )
        .unwrap();
    }
}

/// Type mainly forcing to register in the db
pub struct RateBucketRegistered {}

impl RateBucketRegistered {
    /// Register a bucket by its name in the configuration of the database
    pub fn new(kcfg: &mut KvConfig) -> Self {
        debug!("Bucket '{}' registered", BUCKET_NAME);
        kcfg.bucket(BUCKET_NAME, None);
        RateBucketRegistered {}
    }
}

/// The bucket of rate type
pub struct RateBucket<'r>(Bucket<'r, RateKey, ValueBuf<Bincode<RateVal>>>);

impl<'r> RateBucket<'r> {
    /// Create a new RateBucket. Should have been registered with the register method before
    pub fn new(_: &RateBucketRegistered, store: &Store) -> Self {
        trace!("New RateBucket…");
        let rbucket = store.bucket(Some(BUCKET_NAME));
        trace!("Done");
        RateBucket(rbucket.unwrap())
    }

    fn as_bucket(&self) -> &Bucket<'r, RateKey, ValueBuf<Bincode<RateVal>>> {
        &self.0
    }
}
