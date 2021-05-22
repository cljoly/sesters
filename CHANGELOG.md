# Changelog

## Version 0.3.0

- Add API key parameters in exchange rates API calls
- Update dependencies
- Use the lighter [ureq](https://lib.rs/crates/ureq) instead of [reqwest](https://lib.rs/crates/reqwest)
- Migrate to SQLite instead of LMDB. This will make it easier to add functionalities in the future.
