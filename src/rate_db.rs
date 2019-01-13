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

use chrono::offset::Local as LocalTime;
use chrono::prelude::*;
/// Structs related to rate and rate storage
use log::{info, warn};
use rkv::{Manager, Rkv, Store, Value, Writer};

use std::path::Path;
use std::sync::{RwLockReadGuard, RwLock};

use crate::config::Config;
use crate::currency;
use crate::currency::{Currency, USD};

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

    /// A 1:1 rate for a currency and itself
    pub fn parity(c: &'c Currency) -> Self {
        Rate::new(c, c, Local::now(), 1.)
    }
}

// Rate as stored in LMDB
struct RateLmdb {
    key: String,
    value: Vec<u8>,
}

impl<'c> From<Rate<'c>> for RateLmdb {
    fn from(val: Rate) -> RateLmdb {
        RateLmdb {
            key: format!("r:{}:{}", val.src.get_main_iso(), val.dst.get_main_iso()),
            value: bincode::serialize(&(val.date, val.rate)).unwrap(),
        }
    }
}

impl<'u> From<RateLmdb> for Rate<'u> {
    fn from(val: RateLmdb) -> Rate<'u> {
        let mut currency_keys = val.key.split(":").into_iter();
        // Check type of the key
        assert_eq!("r", currency_keys.next().unwrap());
        let src_key = currency_keys.next().unwrap();
        let dst_key = currency_keys.next().unwrap();
        let (date, rate) = bincode::deserialize(val.value.as_slice()).unwrap();
        Rate {
            src: currency::existing_from_iso(src_key).unwrap(),
            dst: currency::existing_from_iso(dst_key).unwrap(),
            date,
            rate,
        }
    }
}

/// Store and retrieve exchange rate in LMDB database
/// Note that rate from a currency to itself are not stored
pub struct RateDb<'a> {
    // To get LMDB environnement
    // arc: std::sync::Arc<RwLockReadGuard<'a, Rkv>>,
    arc: std::sync::Arc<RwLock<Rkv>>,
    db_path: &'a str,
}

const TMP: &str = "TMP unimplemented";

impl<'a> RateDb<'a> {
    /// Initialize the rate database
    pub fn new(c: Config) -> Self {
        info!("Initialize database");
        let arc = Manager::singleton()
            .write()
            .unwrap()
            .get_or_create(Path::new(&c.db_path), Rkv::new)
            .unwrap();
        // let env = arc.read().unwrap();
        // let store = env.open_or_create_default().unwrap();
        // RateDb { arc, store, db_path: TMP }
        RateDb { arc, db_path: TMP }
    }

    // /// Get current environnement
    // fn env<'b>(&'b self) -> RwLockReadGuard<'b, Rkv> {
    //     self.arc.read().unwrap()
    // }
}

    pub struct RateDbOpened<'a> {
        env: RwLockReadGuard<'a, Rkv>,
        store: Store,
    }

impl<'a> RateDbOpened<'a> {
    /// Get named store
    fn store_named(&self, name: &str) -> Store {
        self.env.open_or_create(name).unwrap()
    }

    /// Get default store
    fn default_store(&self) -> Store {
        self.env.open_or_create_default().unwrap()
    }

    /// Get a writer
    fn new_writer(&self) -> Writer<&str> {
        self.env.write().unwrap()
    }

    /// Get a reader
    fn new_reader(&self) -> Writer<&str> {
        self.env.write().unwrap()
    }

    /// Retrieve rate from a currency to another
    fn get_rate<'c>(&self, src: &'c Currency, dst: &Currency) -> Rate<'c> {
        // Hard code this to limit storage overhead
        if src == dst {
            warn!("Same  source and destination currency, don’t store");
            return Rate::parity(src);
        }
        unimplemented!();
    }

    /// Set rate from a currency to another
    fn set_rate(&self, rate: &Rate) {
        if rate.src == rate.dst {
            warn!("Same  source and destination currency, don’t store");
            return;
        }
        let mut writer = self.new_writer();
        let store = self.default_store();
        writer.put(store, "t", &Value::I64(1));
        unimplemented!();
    }
}
