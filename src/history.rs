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

use crate::convert::conversions_string;
use crate::MainContext;

pub(crate) fn run(ctxt: MainContext, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("clear", m) => clear(ctxt, m)?,
        ("list", m) | (_, m) => list(ctxt, m)?,
    }

    Ok(())
}

fn list(ctxt: MainContext, matches: Option<&ArgMatches>) -> Result<()> {
    // TODO
    // - expire command + auto expire
    // - delete an entry
    // - display conversions
    let limit = matches
        .and_then(|m| m.value_of("MAX_ENTRIES"))
        .map(|l| i32::from_str(l).expect("was validated by clap"))
        .unwrap_or(-1);
    let rows = ctxt.db.read_from_history_max(limit)?;
    let mut table = Table::new();

    let no_convert = matches.map(|m| m.is_present("NO_CONVERT")).unwrap_or(false);

    if rows.len() == 0 {
        println!("History is empty for now");
        return Ok(())
    }

    for row in rows {
        let mut v = Vec::with_capacity(4);
        v.push(format!("{}", row.rowid));
        v.push(format!("{}", row.datetime));
        v.push(format!("{}", row.content));

        if !no_convert {
            v.push(conversions_string(&ctxt, &row.content, Some(3))?);
        }

        table.add_row(Row::new(v))
    }

    println!("{}", table.render());

    Ok(())
}

fn clear(ctxt: MainContext, _matches: Option<&ArgMatches>) -> Result<()> {
    let now = Utc::now();
    let remove_before = now.checked_add_signed(Duration::days(-30)).expect("overflow");
    let n = ctxt.db.remove_from_history(&remove_before)?;
    Ok(println!("Deleted {} entries", n))
}
