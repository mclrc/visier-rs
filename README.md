# vizier-rs
[![Tests](https://github.com/mclrc/vizier-rs/actions/workflows/tests.yml/badge.svg)](https://github.com/mclrc/vizier-rs/actions/workflows/tests.yml) [![Crates.io](https://img.shields.io/crates/v/vizier-adql.svg)](https://crates.io/crates/vizier-adql)

## A basic VizieR client for rust

Allows easy, type-safe access to [VizieR](https://vizier.cds.unistra.fr/) TAP APIs to query a wide variety of astronomical catalogues using [ADQL](https://tapvizier.u-strasbg.fr/adql/help.html).

### Installation
Run `$ cargo add vizier-adql`, or add `vizier-adql = "0.1.1"` to `Cargo.toml` under `[dependencies]`.

### Basic, untyped usage
```rust
use vizier_adql::Client;
use serde_json::Value;

// 1. Create a client
let client = Client::default();

// 2. Execute queries
let objects = client
    .query::<Value>("SELECT TOP 100 * FROM \"I/261/fonac\"")
    .await
    .unwrap();

// ...
```
`Client::default()` will use `http://tapvizier.u-strasbg.fr/TAPVizieR/tap/sync` as the TAP endpoint. If you need to specify a different endpoint, use `Client::new("your_endpoint_url")`.

### Sync/Async
If you don't want to use async, enable the `is_sync` feature on the crate. The API won't change, except that `Client::query` now blocks and returns the value directly.

### Typed usage
To strictly parse the response, create a struct resembling an element from the response and derive `Deserialize` for it.

```rust
#[derive(Deserialize)]
struct CatalogueObject {
    AC2000: i32,
    ACT: Option<i32>,
    #[serde(rename = "B-R")]
    BR: Option<f64>,
    #[serde(rename = "B-V")]
    BV: Option<f64>,
    Bmag: f64,
    DEJ2000: f64,
    #[serde(rename = "Ep-1900")]
    Ep1900: f64,
    Qflag: Option<i32>,
    RAJ2000: f64,
    pmDE: f64,
    pmRA: f64,
    q_Bmag: Option<i32>,
    recno: i32,
}
```
Then, query and parse like so:
```rust
let objects = client
    .query::<CatalogueObject>("SELECT TOP 100 * FROM \"I/261/fonac\"")
    .await
    .unwrap();

// ...
```
