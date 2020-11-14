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

//! Bucket management

use chrono::offset::Local as LocalTime;
use chrono::prelude::*;
use kv::bincode::Bincode;
use kv::{Bucket, Config as KvConfig, Serde, Store, Txn, ValueBuf};
use log::{debug, error, trace, warn};
use serde_derive::{Deserialize, Serialize};

/// Buckets available in db
pub enum AvailableBucket {
    RateBucket,
    TextBucket,
}

impl AvailableBucket {
    /// Name in the db for the bucket
    fn name(&self) -> String {
        match self {
            RateBucket => String::from("rate"),
            TextBucket => String::from("text"),
        }
    }
}

/// Type mainly forcing to register in the db
pub struct BucketRegistered(AvailableBucket);

impl BucketRegistered {
    /// Register a bucket by its name in the configuration of the database
    pub fn new(kcfg: &mut KvConfig, abucket: AvailableBucket) -> Self {
        debug!("Bucket '{}' registered", self.0.name());
        kcfg.bucket(BUCKET_NAME, None);
        TextBucketRegistered {}
    }
}

/// The bucket of text type
pub struct TextBucket<'r>(Bucket<'r, TextKey, ValueBuf<Bincode<TextVal>>>);

impl<'r> TextBucket<'r> {
    /// Create a new TextBucket. Should have been registered with the register method before
    pub fn new(_: &TextBucketRegistered, store: &Store) -> Self {
        trace!("New TextBucket…");
        let rbucket = store.bucket(Some(BUCKET_NAME));
        trace!("Done");
        TextBucket(rbucket.unwrap())
    }

    fn as_bucket(&self) -> &Bucket<'r, TextKey, ValueBuf<Bincode<TextVal>>> {
        &self.0
    }
}
