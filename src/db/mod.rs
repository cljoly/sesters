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

use std::convert::TryInto;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use log::{debug, trace, warn};
use rusqlite::named_params;
use rusqlite::Connection;
use serde_rusqlite::columns_from_statement;
use serde_rusqlite::from_row_with_columns;
use serde_rusqlite::to_params_named;

mod migrations;
mod rate;

use migrations::MIGRATIONS;
use rate::RateInternal;

use crate::config::Config;
use crate::currency::Currency;
use crate::rate::Rate;

#[cfg(test)]
mod tests;

/// Store and bucket, represent the whole database
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Initialize the rate database
    pub fn new(cfg: &Config) -> Result<Self> {
        trace!("Initialize database");
        let mut conn = Connection::open(cfg.db_path())?;

        conn.pragma_update(None, "journal_mode", &"WAL").unwrap();

        if log::log_enabled!(log::Level::Trace) {
            conn.profile(Some(|stmt, duration| {
                trace!("sqlite statement ({:?}): {}", duration, stmt)
            }));
        }

        Db::init(conn)
    }

    /// Initialize database, in particular, apply migrations for the schema
    fn init(mut conn: Connection) -> Result<Self> {
        MIGRATIONS.to_latest(&mut conn)?;
        conn.is_autocommit();
        Ok(Db { conn })
    }

    /// In memory database, mainly for testing
    #[cfg(test)]
    fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Db::init(conn)
    }
}

impl Db {
    /// Retrieve rates from a currency to another. Returns a vector of up-to-date rates
    pub fn get_uptodate_rates<'c>(
        &self,
        src: &'c Currency,
        dst: &'c Currency,
        provider: &str,
        now: DateTime<Utc>,
    ) -> Result<Vec<Rate<'c>>> {
        trace!("get_rates_uptodate_rates({}, {}, {:?})", src, dst, provider);
        // Hard code this to limit storage overhead
        if src == dst {
            warn!("Same source and destination currency, don’t store");
            return Ok(vec![Rate::parity(src)]);
        }

        let mut stmt = self.conn.prepare_cached(
            "SELECT * FROM rates \
             WHERE src = :src AND dst = :dst
             AND cache_until > :now
             AND provider = :provider",
        )?;
        let columns = columns_from_statement(&stmt);
        let mut rows = stmt.query_named(named_params! {
            ":src": src.get_main_iso(),
            ":dst": dst.get_main_iso(),
            ":now": now,
            ":provider": provider,
        })?;

        let mut uptodate_rates: Vec<Rate> = Vec::new();
        while let Some(row) = rows.next()? {
            let rate_internal = from_row_with_columns::<RateInternal>(row, &columns)?;
            let rate: Rate = rate_internal.try_into()?;
            uptodate_rates.push(rate);
        }

        trace!("uptodate_rates: {:?}", uptodate_rates);
        Ok(uptodate_rates)
    }

    /// Removes outdated rates. Returns the number of rates deleted
    pub fn remove_outdated_rates<'c>(
        &self,
        src: &'c Currency,
        dst: &'c Currency,
        provider: &str,
        now: DateTime<Utc>,
    ) -> Result<usize> {
        let mut stmt = self.conn.prepare_cached(
            "DELETE FROM rates \
             WHERE src = :src AND dst = :dst
             AND cache_until <= :now
             AND provider = :provider",
        )?;

        let deleted = stmt.execute_named(named_params! {
            ":src": src.get_main_iso(),
            ":dst": dst.get_main_iso(),
            ":now": now,
            ":provider": provider,
        })?;

        trace!("deleted rates: {:?}", deleted);
        Ok(deleted)
    }

    /// Set rate from a currency to another
    pub fn set_rate(&self, rate: &Rate) -> Result<()> {
        if rate.src() == rate.dst() {
            warn!("Same source and destination currency, don’t store");
            return Ok(());
        }

        let ri: RateInternal = rate.try_into()?;

        let n = self.conn.execute_named(
            "INSERT OR REPLACE INTO rates (src, dst, date, rate, provider, cache_until) VALUES (:src, :dst, :date, :rate, :provider, :cache_until)",
            &to_params_named(ri)?.to_slice(),
        )?;

        debug!("Upserted {} rows.", n);
        Ok(())
    }
}
