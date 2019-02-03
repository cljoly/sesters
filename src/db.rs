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

//! Module grouping all db related concern

use kv::{Config as KvConfig, Manager, Store};
use log::info;

mod rate;

use rate::{RateBucket, RateBucketRegistered};

#[cfg(test)]
mod tests {}

/// Store and bucket, represent the whole database
pub struct Db {
    store_handle: std::sync::Arc<std::sync::RwLock<kv::Store>>,
    rbr: RateBucketRegistered
}

/// All supported bucket
enum BucketList{Rate}

impl Db {
    /// Initialize the rate database
    pub fn new(mut kcfg: KvConfig, mgr: &mut Manager) -> Self {
        info!("Initialize database");

        let rbr = RateBucketRegistered::new(&mut kcfg);

        let store_handle = mgr.open(kcfg).unwrap();

        Db {
            store_handle,
            rbr
        }
    }

    /// Access the store
    fn store(&self) -> std::sync::RwLockWriteGuard<Store> {
        self.store_handle.write().unwrap()
    }

    /// Access a bucket
    // TODO Generic function for any bucket
    fn bucket_rate(&self) -> RateBucket {
        let store = self.store();
        RateBucket::new(&self.rbr, &store)
    }
}
