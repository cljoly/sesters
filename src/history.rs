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
use term_table::{row::Row, Table};

use crate::convert::convert_string;
use crate::db::history::History;
use crate::tools::yes_or_no;
use crate::{HistoryCommands, MainContext};

// Default time in days before history entries are expireed
pub(crate) static EXPIRE_DELAY: &'static str = "30";

pub(crate) fn run(ctxt: MainContext, subcommand: HistoryCommands) -> Result<()> {
    match subcommand {
        HistoryCommands::Expire { all, days, yes } => {
            let days = if all { 0 } else { days };
            expire(&ctxt, days, yes)?
        }
        HistoryCommands::List {
            no_convert,
            max_entries,
        } => {
            list(&ctxt, max_entries as i32, no_convert)?;
            auto_expire(&ctxt)?
        }
    }

    Ok(())
}

fn list(ctxt: &MainContext, limit: i32, no_convert: bool) -> Result<()> {
    // TODO
    // - delete an entry
    let rows = ctxt.db.read_from_history_max(limit)?;

    return print(ctxt, &rows, no_convert);
}

fn print(ctxt: &MainContext, histories: &[History], no_convert: bool) -> Result<()> {
    if histories.len() == 0 {
        println!("History is empty for now");
        return Ok(());
    }

    let mut table = Table::new();
    for history_entry in histories {
        let mut v = Vec::with_capacity(4);
        v.push(format!("{}", history_entry.rowid));
        v.push(format!("{}", history_entry.datetime.to_rfc2822()));
        v.push(format!("{}", history_entry.content));

        if !no_convert {
            v.push(convert_string(&ctxt, &history_entry.content, Some(3))?);
        }

        table.add_row(Row::new(v))
    }

    println!("{}", table.render());

    Ok(())
}

fn expire(ctxt: &MainContext, expire_delay_days: usize, silent: bool) -> Result<()> {
    let now = Utc::now();
    let remove_before = now
        .checked_add_signed(Duration::days(-1 * expire_delay_days as i64))
        .expect("overflow");

    if !silent {
        let history = ctxt.db.read_from_history_before(&remove_before)?;
        print(ctxt, &history, true)?;

        if history.len() == 0 {
            println!("Nothing to delete");
            return Ok(());
        }

        if !yes_or_no("Do you want to remove the above entries from history?") {
            println!("No entry deleted");
            return Ok(());
        }
    }

    let n = ctxt.db.remove_from_history(&remove_before)?;
    if !silent {
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
    expire(ctxt, EXPIRE_DELAY.parse().unwrap(), true)
}
