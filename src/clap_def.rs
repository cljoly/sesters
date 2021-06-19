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

pub fn get_app() -> App<'static, 'static> {
    clap_app!(sesters =>
        // (@setting DontCollapseArgsInUsage)
        (version: crate_version!())
        (author: crate_authors!())
        (about: concat!(crate_description!(), "\n", "https://seste.rs"))
        // TODO Implement -c
        // (@arg CONFIG: -c --config +global +takes_value "Sets a custom config file")
        // TODO Add flag for verbosity
        // TODO Implement tag for preferred currency
        (@arg TO: -t --to <CURRENCY> !required +takes_value +global +multiple "Target currency by ISO symbol, uses defaults from the configuration file if not set")

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
            (@arg NO_CONVERT: --no-convert "Don’t perform conversions of the history content")
            (@arg MAX_ENTRIES: --max-entries <N> +takes_value "Show at most <N> entries")
            (@subcommand clear =>
                (about: "Removes entries from history")
                (@arg ALL: --all "Removes all entries from history")
                (@arg MAX_ENTRIES: +multiple <ID> "Removes entry with the given <ID>s")
            )
        )
    )
}
