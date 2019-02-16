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

use chrono::offset::Local as LocalTime;
use chrono::prelude::*;
use kv::bincode::Bincode;
use kv::{Bucket, Config as KvConfig, Serde, Store, Txn, ValueBuf};
use lazy_static::lazy_static;
use log::{info, warn};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use crate::currency;
use crate::currency::{Currency, USD};

#[cfg(test)]
mod tests {

    use super::*;
    use crate::currency::{BTC, EUR, USD};

    // Ensure nothing is lost when converting to RateKey
    #[test]
    fn ratekey_and_back() {
        let rk1 = RateKey::new(&EUR, &EUR);
        let rk2 = RateKey::new(&USD, &BTC);
        assert_eq!(rk1.currencies(), (&EUR, &EUR));
        assert_eq!(rk2.currencies(), (&USD, &BTC))
    }
}

/// Rate form src currency to dst currency
#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub struct Rate<'c> {
    src: &'c Currency,
    dst: &'c Currency,
    date: DateTime<LocalTime>,
    rate: f64,
}

impl<'c> Default for Rate<'c> {
    fn default() -> Self {
        Rate {
            date: Local::now(),
            rate: 0.,
            src: &USD,
            dst: &USD,
        }
    }
}

impl<'c> Rate<'c> {
    /// Instanciate
    pub fn new(src: &'c Currency, dst: &'c Currency, date: DateTime<LocalTime>, rate: f64) -> Self {
        Rate {
            src,
            dst,
            date,
            rate,
        }
    }

    /// New rate with date set to now (local time)
    pub fn now(src: &'c Currency, dst: &'c Currency, rate: f64) -> Self {
        Self::new(src, dst, Local::now(), rate)
    }

    /// A 1:1 rate for a currency and itself
    pub fn parity(c: &'c Currency) -> Self {
        Rate::new(c, c, Local::now(), 1.)
    }

    /// Source currency
    pub fn src(&self) -> &Currency {
        &self.src
    }

    /// Destination currency
    pub fn dst(&self) -> &Currency {
        &self.dst
    }

    /// Date of the rate
    pub fn date(&self) -> &DateTime<LocalTime> {
        &self.date
    }

    /// Rate
    pub fn rate(&self) -> f64 {
        self.rate
    }
}

// The key to find a rate in the database
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct RateKey(String);

impl RateKey {
    // New rate key
    fn new(src: &Currency, dst: &Currency) -> RateKey {
        RateKey(format!("{}:{}", src.get_main_iso(), dst.get_main_iso()))
    }

    // Get currencies used in the rate key. Panics if a rate is malformed.
    fn currencies(&self) -> (&'static Currency, &'static Currency) {
        lazy_static! {
            static ref KEY: Regex = Regex::new("^(?P<src>[A-Z]{3}):(?P<dst>[A-Z]{3})$").unwrap();
        }
        let cap = KEY.captures(&self.0).unwrap();
        let src_iso = cap.name("src").unwrap().as_str();
        let dst_iso = cap.name("dst").unwrap().as_str();
        let src = currency::existing_from_iso(src_iso).unwrap();
        let dst = currency::existing_from_iso(dst_iso).unwrap();
        (src, dst)
    }
}

// Data of a rate (value of the key in the database)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateVal(DateTime<LocalTime>, f64);

// Internal representation for storage of a rate: in the database, a rate is
// stored partly on the key and partly on the value
struct RateInternal {
    key: RateKey,
    value: RateVal,
}

impl<'c> From<Rate<'c>> for RateInternal {
    fn from(val: Rate) -> RateInternal {
        RateInternal::new(RateKey::new(val.src, val.dst), RateVal(val.date, val.rate))
    }
}

impl From<RateInternal> for Rate<'static> {
    fn from(ri: RateInternal) -> Rate<'static> {
        let (src, dst) = ri.key.currencies();
        let date = ri.value.0;
        let rate = ri.value.1;
        Rate::new(src, dst, date, rate)
    }
}

impl RateInternal {
    fn new(key: RateKey, val: RateVal) -> RateInternal {
        RateInternal {
            // key: RateKey::new(val.src, val.dst),
            // val: RateVal(val.date, val.rate),
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
    ) -> Option<Rate<'c>> {
        // Hard code this to limit storage overhead
        if src == dst {
            warn!("Same source and destination currency, don’t store");
            return Some(Rate::parity(src));
        }
        // TODO Return None only when a key is not found, not for any error
        let rk = RateKey::new(src, dst);
        let bucket = self.bucket_rate(store);
        let rvg = txn.get(&bucket.as_bucket(), rk.clone());
        let rv: Option<RateVal> = rvg.map(|buf| buf.inner().unwrap().to_serde()).ok();
        rv.map(|rv| RateInternal::new(rk, rv).into())
    }

    /// Set rate from a currency to another
    // TODO Return error type
    pub fn set_rate<'t, 'b, 'd>(
        &'d self,
        txn: &mut Txn<'t>,
        store: &Store,
        bucket: &Bucket<'t, RateKey, ValueBuf<Bincode<RateVal>>>,
        rate: Rate,
    ) where
        'd: 'b,
        'b: 't,
    {
        if rate.src == rate.dst {
            warn!("Same  source and destination currency, don’t store");
            return;
        }
        let ri: RateInternal = rate.into();
        txn.set(
            bucket,
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
        info!("Bucket '{}' registered", BUCKET_NAME);
        kcfg.bucket(BUCKET_NAME, None);
        RateBucketRegistered {}
    }
}

/// The bucket of rate type
pub struct RateBucket<'r>(Bucket<'r, RateKey, ValueBuf<Bincode<RateVal>>>);

impl<'r> RateBucket<'r> {
    /// Create a new RateBucket. Should have been registered with the register method before
    pub fn new(_: &RateBucketRegistered, store: &Store) -> Self {
        info!("New RateBucket");
        let rbucket = store.bucket(Some(BUCKET_NAME));
        dbg!("Done");
        RateBucket(rbucket.unwrap())
    }

    pub fn as_bucket(&self) -> &Bucket<'r, RateKey, ValueBuf<Bincode<RateVal>>> {
        &self.0
    }
}
