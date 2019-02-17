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

use chrono::prelude::*;
use chrono::offset::Local as LocalTime;
use kv::{Bucket, Config as KvConfig, Serde, Store, Txn, ValueBuf};
use kv::bincode::Bincode;
use lazy_static::lazy_static;
use log::{info, warn, debug, trace};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use crate::currency;
use crate::currency::{Currency, USD};
use crate::rate::Rate;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::currency::{BTC, EUR, USD, CHF};

    // Ensure nothing is lost when converting to RateKey
    #[test]
    fn ratekey_and_back() {
        let fut = Local.ymd(2020,9,30).and_hms(17,38,49);
        let horrible_provider_name =  String::from("somelongand://strange_name.1230471-02347.TEST.com/@ñé:--:123");
        let rk1 = RateKey::new(&EUR, &EUR, "TEST1", &fut);
        let rk2 = RateKey::new(&USD, &BTC, &horrible_provider_name, &fut);
        assert_eq!(rk1.data(), (&EUR, &EUR, String::from("TEST1"), fut.clone()));
        assert_eq!(rk2.data(), (&USD, &BTC, horrible_provider_name, fut.clone()))
    }

    // Ensure nothing is lost when converting to RateInternal
    #[test]
    fn rateinternal_and_back() {
        let now = Local::now();
        let fut = Local.ymd(2020,9,30).and_hms(17,38,49);
        let r1 = Rate::new(&EUR, &CHF, now, 9.12, "RateInternal".to_string(), Some(fut.clone()));
        let ri1: RateInternal = r1.clone().into();
        let r1_back: Rate = ri1.into();
        assert_eq!(r1, r1_back);

        let r2 = Rate::new(&BTC, &USD, now, 9.12, "RateInternalNoCache".to_string(), Some(fut.clone()));
        assert_ne!(r1, r2);
        assert_ne!(r1_back, r2);
        let r3 = Rate::new(&BTC, &USD, now, 9.12, "RateInternal".to_string(), Some(fut.clone()));
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
        let r2 = Rate::new(&BTC, &USD, now, 9.1, "RateInternalNoCache".to_string(), None);
        let ri2: RateInternal = r2.clone().into();
    }
}

// The key to find a rate in the database
#[derive(Clone, Debug, PartialOrd, PartialEq)]
struct RateKey(String);

impl RateKey {
    // New rate key
    fn new(src: &Currency, dst: &Currency, provider: &str, cache_until: &DateTime<LocalTime>) -> RateKey {
        RateKey(format!("{}:{}:{}:{}", src.get_main_iso(), dst.get_main_iso(), provider, cache_until.timestamp()))
    }

    // Get data stored in the rate key. Panics if a rate is malformed.
    fn data(&self) -> (&'static Currency, &'static Currency, String, DateTime<LocalTime>) {
        lazy_static! {
            static ref KEY: Regex = Regex::new(r"^(?P<src>[A-Z]{3}):(?P<dst>[A-Z]{3}):(?P<prov>.+):(?P<until>\d+)$").unwrap();
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
        RateInternal::new(RateKey::new(val.src(), val.dst(), val.provider(), val.cache_until().as_ref().unwrap()), RateVal(*val.date(), val.rate()))
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
        RateInternal {
            key,
            value: val,
        }
    }
}

/// Store and retrieve exchange rate in LMDB database
/// Note that rate from a currency to itself are not stored
pub struct RateDb<'a> {
    bucket: Bucket<'a, RateKey, ValueBuf<Bincode<RateVal>>>,
}

impl std::convert::AsRef<[u8]> for RateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref()
    }
}

pub const BUCKET_NAME: &str = "rate";

impl super::Db {
    /// Retrieve rate from a currency to another
    pub fn get_rate<'c>(
        &self,
        txn: &Txn,
        store: &Store,
        src: &'c Currency,
        dst: &Currency,
        provider: &str,
    ) -> Option<Rate<'c>> {
        // Hard code this to limit storage overhead
        if src == dst {
            warn!("Same source and destination currency, don’t store");
            return Some(Rate::parity(src));
        }
        // TODO Return None only when a key is not found, not for any error
        let rk = RateKey::new(src, dst, provider, unimplemented!());
        let bucket = self.bucket_rate(store);
        let rvg = txn.get(&bucket.as_bucket(), rk.clone());
        let rv: Option<RateVal> = rvg.map(|buf| buf.inner().unwrap().to_serde()).ok();
        rv.map(|rv| RateInternal::new(rk, rv).into())
    }

    /// Set rate from a currency to another
    // TODO Return error type
    pub fn set_rate<'t, 'd>(
        &'d self,
        txn: &mut Txn<'t>,
        store: &Store,
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
