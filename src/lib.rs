use maybe_async::maybe_async;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Map, Value};
use thiserror::Error;

#[cfg(not(feature = "is_sync"))]
use reqwest::Client as HttpClient;

#[cfg(feature = "is_sync")]
use reqwest::blocking::Client as HttpClient;

const DEFAULT_VIZIER_TAP_URL: &str = "http://tapvizier.u-strasbg.fr/TAPVizieR/tap/sync";

#[cfg(not(feature = "is_sync"))]
macro_rules! maybe_await {
    ($future:expr) => {
        $future.await
    };
}

#[cfg(feature = "is_sync")]
macro_rules! maybe_await {
    ($value:expr) => {
        $value
    };
}

#[derive(Error, Debug)]
pub enum VizierError {
    #[error("Request failed: {0}")]
    RequestFailed(reqwest::Error),
    #[error("Non-success status code: {0}")]
    NonSuccessStatus(reqwest::StatusCode),
    #[error("Unexpected response schema: {0}")]
    UnexpectedSchema(String),
    #[error("Failed to deserialize response: {0}")]
    DeserializationFailed(serde_json::Error),
    #[error("{0}")]
    Other(String),
}

#[derive(Deserialize, Debug)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: String,
    pub arraysize: Option<String>,
    pub unit: Option<String>,
    pub ucd: String,
}

#[derive(Deserialize)]
struct ResponseSchema {
    #[serde(rename = "metadata")]
    meta: Vec<ColumnMetadata>,
    data: Vec<Vec<Value>>,
}

pub struct QueryResult<T> {
    meta: Vec<ColumnMetadata>,
    data: Vec<T>,
}

impl<T> QueryResult<T> {
    pub fn meta(&self) -> &[ColumnMetadata] {
        &self.meta
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub struct Client {
    tap_url: String,
    http_client: HttpClient,
}

impl Client {
    pub fn new(tap_url: &str) -> Self {
        Self {
            tap_url: tap_url.to_string(),
            http_client: HttpClient::new(),
        }
    }

    #[maybe_async]
    pub async fn query<T: DeserializeOwned>(
        &self,
        adql_query: &str,
    ) -> Result<QueryResult<T>, VizierError> {
        let request_query = json!({
            "request": "doQuery",
            "lang": "ADQL",
            "format": "json",
            "query": adql_query
        });

        let response = maybe_await!(self
            .http_client
            .get(&self.tap_url)
            .query(&request_query)
            .send())
        .map_err(VizierError::RequestFailed)?;

        if response.status().is_success() {
            let data =
                maybe_await!(response.json::<Value>()).map_err(VizierError::RequestFailed)?;
            let parsed_data = Client::parse_query_result::<T>(data)
                .map_err(VizierError::DeserializationFailed)?;

            Ok(parsed_data)
        } else {
            Err(VizierError::NonSuccessStatus(response.status()))
        }
    }

    fn parse_query_result<T: DeserializeOwned>(
        data: Value,
    ) -> Result<QueryResult<T>, serde_json::Error> {
        let response = serde_json::from_value::<ResponseSchema>(data)?;

        let mut result = Vec::new();
        for row in response.data {
            let mut row_data = Map::new();

            for (i, value) in row.iter().enumerate() {
                row_data.insert(response.meta[i].name.clone(), value.clone());
            }
            result.push(serde_json::from_value(Value::Object(row_data))?);
        }

        Ok(QueryResult {
            meta: response.meta,
            data: result,
        })
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new(DEFAULT_VIZIER_TAP_URL)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[cfg(not(feature = "is_sync"))]
    #[tokio::test]
    async fn query_test() {
        let client = Client::default();

        let result = client
            .query::<Value>("SELECT TOP 100 * FROM \"I/261/fonac\"")
            .await
            .unwrap();

        assert!(result.len() == 100);
    }

    #[derive(Deserialize, Debug)]
    #[allow(non_snake_case, dead_code)]
    struct Object {
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

    #[cfg(not(feature = "is_sync"))]
    #[tokio::test]
    async fn query_test_typed() {
        let client = Client::default();

        client
            .query::<Object>("SELECT TOP 100 * FROM \"I/261/fonac\"")
            .await
            .unwrap();
    }

    #[cfg(feature = "is_sync")]
    #[test]
    fn query_test_sync() {
        let client = Client::default();
        let result = client
            .query::<Value>("SELECT TOP 100 * FROM \"I/261/fonac\"")
            .unwrap();

        assert!(result.len() == 100);
    }
}
