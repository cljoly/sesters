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

//! Store common representation for rates

use chrono::prelude::*;
use chrono::offset::Local as LocalTime;

use crate::currency::{Currency, USD};

#[cfg(test)]
mod tests {
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
