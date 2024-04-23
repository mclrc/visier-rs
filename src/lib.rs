use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::error::Error;

const DEFAULT_VIZIER_TAP_URL: &str = "http://tapvizier.u-strasbg.fr/TAPVizieR/tap/sync";

pub struct Client {
    tap_url: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(tap_url: &str) -> Self {
        Self {
            tap_url: tap_url.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn query<T: DeserializeOwned>(
        &self,
        adql_query: &str,
    ) -> Result<Vec<T>, Box<dyn Error>> {
        let request_query = json!({
            "request": "doQuery",
            "lang": "ADQL",
            "format": "json",
            "query": adql_query
        });

        let response = self
            .http_client
            .get(&self.tap_url)
            .query(&request_query)
            .send()
            .await?;

        if response.status().is_success() {
            let data = response.json::<Value>().await?;
            let parsed_data = Client::parse_query_result::<T>(data)?;
            Ok(parsed_data)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch data",
            )))
        }
    }

    fn parse_query_result<T: DeserializeOwned>(data: Value) -> Result<Vec<T>, Box<dyn Error>> {
        let columns = data
            .get("metadata")
            .ok_or("Metadata not found")?
            .as_array()
            .ok_or("Metadata is not an array")?;
        let data = data.get("data").ok_or("Data not found")?;

        let mut column_names = Vec::new();
        for column in columns {
            let name = column.get("name").ok_or("Column name not found")?;
            column_names.push(name.as_str().ok_or("Column name is not a string")?);
        }

        let mut result = Vec::new();
        for row in data.as_array().ok_or("Data is not an array")? {
            let mut row_data = Map::new();

            for (i, value) in row
                .as_array()
                .ok_or("Row is not an array")?
                .iter()
                .enumerate()
            {
                row_data.insert(column_names[i].to_string(), value.clone());
            }
            result.push(serde_json::from_value(Value::Object(row_data))?);
        }

        Ok(result)
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
    struct QueryResult {
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

    #[tokio::test]
    async fn query_test_typed() {
        let client = Client::default();

        client
            .query::<QueryResult>("SELECT TOP 100 * FROM \"I/261/fonac\"")
            .await
            .unwrap();
    }
}
