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

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, AppSettings, Parser, Subcommand};
use log::{error, info};

mod api;
mod config;
mod convert;
pub mod currency;
mod db;
mod history;
mod price_format;
pub mod price_in_text;
mod rate;

use crate::config::Config;
use crate::currency::Currency;
use crate::db::Db;

/// Main context to pass what is initiliazed in this module and what is parsed
/// in global tags
pub struct MainContext<'mc> {
    db: Db,
    destination_currencies: Vec<&'mc Currency>,
    cfg: Config,
}

impl<'mc> MainContext<'mc> {
    pub(crate) fn new(cfg: Config, destination_currencies: Vec<&'mc Currency>) -> Result<Self> {
        let db = Db::new(&cfg).unwrap();

        Ok(MainContext {
            cfg,
            db,
            destination_currencies,
        })
    }
}

#[derive(Parser, Debug)]
#[clap(name = "sesters",
  version = crate_version!(),
  author = crate_authors!(),
  about = concat!(crate_description!(),
    "\n",
    "https://cj.rs/sesters/"),
  long_about = None)]
#[clap(setting(AppSettings::DontCollapseArgsInUsage))]
pub(crate) struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Target currency by ISO symbol, uses defaults from the configuration file if not set
    #[clap(short = 't', value_name = "CURRENCY")]
    to: Vec<String>,
    // TODO Add flag for verbosity
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Perform currency conversion to your preferred currency,
    /// from a price tag found in plain text
    #[clap(setting(AppSettings::InferSubcommands))]
    Convert {
        /// Read text containing price tag from stdin
        #[clap(long = "stdin")]
        stdin: bool,

        /// Find at most n price tag in the text, i.e. 3
        #[clap(short = 'n')]
        findn: Option<usize>,

        /// Plain text to extract a price tag from. If not set, plain text will be read from stdin
        plain_text: Vec<String>,
    },

    /// Access and manage the history of price tags extracted
    #[clap(setting(AppSettings::InferSubcommands))]
    History {
        #[clap(subcommand)]
        command: HistoryCommands,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum HistoryCommands {
    /// List entries in the history
    #[clap(setting(AppSettings::InferSubcommands))]
    List {
        /// Don’t perform conversions of the history content
        #[clap(short = 'n', long = "noconvert")]
        no_convert: bool,
        /// Show at most <N> entries
        #[clap(short = 'm', long = "max", default_value = "50")]
        max_entries: usize,
    },

    /// Removes older entries from history
    #[clap(setting(AppSettings::InferSubcommands))]
    Expire {
        // TODO Implement
        /// Removes all entries from history
        #[clap(long = "all")]
        all: bool,

        /// Delete all entries older than the given number of days (default 30)
        #[clap(default_value = crate::history::EXPIRE_DELAY)]
        days: usize,
    },
}

fn from_args() -> Result<()> {
    Config::init()?;
    let cfg = Config::new()?;

    // Argument parsing
    let args = Cli::parse();

    let txt_destination_currencies = if args.to.len() == 0 {
        // Use configuration currency if none are specified
        cfg.currencies()
    } else {
        &args.to
    };
    let destination_currencies: Vec<&Currency> = txt_destination_currencies
        .iter()
        .filter_map(|iso_name| {
            currency::existing_from_iso(&iso_name).or_else(|| {
                error!("Invalid currency iso symbol '{}', ignored", iso_name);
                None
            })
        })
        .collect();

    let ctxt = MainContext::new(cfg, destination_currencies)?;

    match args.command {
        Commands::Convert {
            stdin,
            findn,
            plain_text,
        } => convert::run(ctxt, stdin, findn, plain_text)?,
        Commands::History { command } => history::run(ctxt, command)?,
    }

    info!("Exiting");
    Ok(())
}

fn main() {
    log::set_max_level(log::LevelFilter::Info);
    env_logger::init();
    info!("Starting up");

    from_args().unwrap();
}
