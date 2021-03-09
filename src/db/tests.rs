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

//! Tests for the database

use chrono::offset::Utc;
use chrono::Duration;

use crate::{
    currency::{BTC, CHF, EUR, JPY},
    db::rate::RateInternalConversionError::MissingCacheUntil,
};

use super::*;

#[test]
fn migrations_test() {
    assert!(MIGRATIONS.validate().is_ok());
}

// TODO Test 3 functions on rate, with rate.cache_until = Some() / None

// Rate with cache_until == None
fn rate_cun() -> Rate<'static> {
    Default::default()
}

// Rate with cache_until == Some(date) where date is in the future
fn rate_cus_future() -> Rate<'static> {
    Rate::now(
        &JPY,
        &BTC,
        2777277.,
        String::from("kraken"),
        Some(Duration::weeks(3)),
    )
}

// Rate with cache_until == Some(date) where date is in the past (expired rate)
fn rate_cus_past() -> Rate<'static> {
    Rate::new(
        &CHF,
        &EUR,
        Utc::now() - Duration::weeks(16),
        0.9,
        String::from("xe"),
        Some(Utc::now() - Duration::weeks(15)),
    )
}

// Ensure we can convert back and forth
#[test]
fn rate_convert_back_forth_test() {
    let now = Utc::now();
    let db = Db::new_in_memory().unwrap();

    let mut retrieved_rates_uptodate = Vec::new();
    let rates = vec![rate_cus_future(), rate_cus_past()];
    for rate in &rates {
        assert!(db.set_rate(&rate).is_ok());

        use rusqlite::ToSql;
        &rate.cache_until().to_sql();

        retrieved_rates_uptodate.append(
            &mut db
                .get_uptodate_rates(rate.src(), rate.dst(), rate.provider(), now)
                .unwrap(),
        );
    }

    assert_eq!(retrieved_rates_uptodate.len(), 1);
    assert_eq!(rates[0], retrieved_rates_uptodate[0]);
}

// Reject cache_until == None
#[test]
fn cache_until_none_rejected_test() {
    let ri: Result<RateInternal, _> = (&rate_cun()).try_into();
    assert_eq!(Err(MissingCacheUntil), ri);

    let db = Db::new_in_memory().unwrap();
    assert!(db.set_rate(&rate_cun()).is_err());
}

// Ensure returned rates are actually up to date
#[test]
fn uptodate_test() {
    let now = Utc::now();

    for rate in vec![rate_cus_future(), rate_cus_past()] {
        let db = Db::new_in_memory().unwrap();
        assert!(db.set_rate(&rate).is_ok());

        let rates_uptodate = db
            .get_uptodate_rates(rate.src(), rate.dst(), rate.provider(), now)
            .unwrap();

        &rates_uptodate;

        for ru in rates_uptodate {
            assert!(ru.uptodate(&now))
        }
    }
}
