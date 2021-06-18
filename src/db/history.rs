/*
Sesters: easily convert one currency to another
Copyright (C) 2018-2021  Clément Joly <oss+sesters@131719.xyz>

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

//! History table of the database

use serde_derive::Serialize;
use serde_derive::Deserialize;

use chrono::{DateTime, Utc};

/// History maps to the db schema
#[derive(Clone, PartialOrd, PartialEq, Debug, Serialize, Deserialize)]
pub struct History {
    pub rowid: u32,
    pub datetime: DateTime<Utc>,
    pub content: String,
}
