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
use clap::ArgMatches;
use term_table::{Table, row::Row, table_cell::TableCell};

use crate::MainContext;

pub(crate) fn run(ctxt: MainContext, _matches: &ArgMatches) -> Result<()> {
    // TODO
    // - expire command + auto expire
    // - delete an entry
    // - display conversions
    let rows = ctxt.db.read_from_history()?;
    let mut table = Table::new();

    for row in rows {
        table.add_row(Row::new(vec![
            TableCell::new(row.rowid),
            TableCell::new(row.datetime),
            TableCell::new(row.content),
        ]))
    }

    println!("{}", table.render());

    Ok(())
}
