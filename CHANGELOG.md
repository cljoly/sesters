# Changelog

## Version 0.3.1

### Bug

- Fix an example in the readme

## Version 0.3.0

### Features

- Add API key parameters in exchange rates API calls

- Allow to read text containing a price tag from stdin
- Allow to limit the number of price tag found to `n`

#### Example

```
echo -e "€12\n£13\nauie\n" | sesters convert --stdin -n 2
GBP 13.00 ➜ EUR 15.10
GBP 13.00 ➜ USD 18.40
EUR 12.00 ➜ USD 14.62
EUR 12.00 ➜ GBP 10.33
```

### Maintenace

- Improve test coverage
- Update dependencies
- Use the lighter [ureq](https://lib.rs/crates/ureq) instead of [reqwest](https://lib.rs/crates/reqwest)
- Migrate to SQLite instead of LMDB. This will make it easier to add functionalities in the future.
