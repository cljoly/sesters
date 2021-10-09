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

//! History subcommand

use anyhow::Result;
use chrono::{Duration, Utc};
use clap::ArgMatches;
use std::str::FromStr;
use term_table::{row::Row, Table};

use crate::convert::convert_string;
use crate::MainContext;

// Default time in days before history entries are expireed
static EXPIRE_DELAY: usize = 30;

pub(crate) fn run(ctxt: MainContext, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("expire", m) => {
            let days: usize = m
                .and_then(|m| {
                    m.value_of("DAYS")
                        .map(|l| usize::from_str(l).expect("was validated by clap"))
                })
                .unwrap_or(EXPIRE_DELAY);

            expire(&ctxt, days, true)?
        }
        ("list", m) | (_, m) => {
            let no_convert = m.map(|m| m.is_present("NO_CONVERT")).unwrap_or(false);

            let limit = m
                .and_then(|m| m.value_of("MAX_ENTRIES"))
                .map(|l| i32::from_str(l).expect("was validated by clap"))
                .unwrap_or(-1);

            list(&ctxt, limit, no_convert)?;
            auto_expire(&ctxt)?
        }
    }

    Ok(())
}

fn list(ctxt: &MainContext, limit: i32, no_convert: bool) -> Result<()> {
    // TODO
    // - delete an entry
    let rows = ctxt.db.read_from_history_max(limit)?;
    let mut table = Table::new();

    if rows.len() == 0 {
        println!("History is empty for now");
        return Ok(());
    }

    for row in rows {
        let mut v = Vec::with_capacity(4);
        v.push(format!("{}", row.rowid));
        v.push(format!("{}", row.datetime));
        v.push(format!("{}", row.content));

        if !no_convert {
            v.push(convert_string(&ctxt, &row.content, Some(3))?);
        }

        table.add_row(Row::new(v))
    }

    println!("{}", table.render());

    Ok(())
}

fn expire(ctxt: &MainContext, expire_delay_days: usize, print_deleted: bool) -> Result<()> {
    let now = Utc::now();
    let remove_before = now
        .checked_add_signed(Duration::days(-1 * expire_delay_days as i64))
        .expect("overflow");
    let n = ctxt.db.remove_from_history(&remove_before)?;

    if print_deleted {
        match n {
            0 => println!("No entry deleted"),
            1 => println!("Deleted one entry"),
            _ => println!("Deleted {} entries", n),
        }
    }

    Ok(())
}

// Automatic cleaning of entries older than a default number of hours
fn auto_expire(ctxt: &MainContext) -> Result<()> {
    expire(ctxt, EXPIRE_DELAY, false)
}
