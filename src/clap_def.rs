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

//! Define clap subcommand

use clap::{clap_app, crate_authors, crate_description, crate_version, App};
use std::str::FromStr;

pub fn get_app() -> App<'static, 'static> {
    clap_app!(sesters =>
        // (@setting DontCollapseArgsInUsage)
        (version: crate_version!())
        (author: crate_authors!())
        (about: concat!(crate_description!(), "\n", "https://seste.rs"))
        // TODO Add flag for verbosity
        // TODO Implement tag for preferred currency
        (@arg TO: -t --to <CURRENCY> !required +takes_value +multiple "Target currency by ISO symbol, uses defaults from the configuration file if not set")

        (@subcommand convert =>
            (@setting TrailingVarArg)
            (about: "Perform currency conversion to your preferred currency, from a price tag found in plain text")
            (visible_alias: "c")
            (@arg STDIN: --stdin "Read text containing price tag from stdin")
            (@arg FINDN: -n --findn +takes_value "Find at most n price tag in the text, i.e. 3")
            (@arg PLAIN_TXT: +multiple !use_delimiter "Plain text to extract a price tag from. If not set, plain text will be read from stdin")
        )

        (@subcommand history =>
            (about: "Access and manage the history of price tags extracted")
            (@subcommand list =>
                (about: "List entries in the history")
                (@arg NO_CONVERT: -n --noconvert "Don’t perform conversions of the history content")
                (@arg MAX_ENTRIES: -m --max +takes_value validator(integer) "Show at most <N> entries")
            )
            (@subcommand expire =>
                (about: "Removes older entries from history")
                // TODO Implement
                // (@arg ALL: --all "Removes all entries from history")
                (@arg DAYS: -d --days +takes_value validator(integer) "Delete all entries older than the given number of days (default 30)")
            )
        )
    )
}

fn integer(v: String) -> Result<(), String> {
    return i32::from_str(&v).map_err(|e| format!("{}", e)).map(|_| ());
}
