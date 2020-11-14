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

//! Structs related to text storage and query

pub const BUCKET_NAME: &str = "text";

/// Type mainly forcing to register in the db
pub struct TextBucketRegistered {}

impl TextBucketRegistered {
    /// Register a bucket by its name in the configuration of the database
    pub fn new(kcfg: &mut KvConfig) -> Self {
        debug!("Bucket '{}' registered", BUCKET_NAME);
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
