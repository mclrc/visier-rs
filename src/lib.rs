use serde_json::{json, Map, Value};
use std::error::Error;

const VIZIER_TAP_URL: &str = "http://tapvizier.u-strasbg.fr/TAPVizieR/tap/sync";

type QueryResult = Vec<Value>;

fn parse_query_result(data: Value) -> Result<QueryResult, Box<dyn Error>> {
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
        result.push(Value::Object(row_data));
    }

    Ok(result)
}

pub async fn query_vizier(adql_query: &str) -> Result<QueryResult, Box<dyn Error>> {
    let request_body = json!({
        "request": "doQuery",
        "lang": "ADQL",
        "format": "json",
        "query": adql_query
    });

    let response = {
        let client = reqwest::Client::new();

        client
            .get(VIZIER_TAP_URL)
            .query(&request_body)
            .send()
            .await?
    };

    if response.status().is_success() {
        let data = response.json::<Value>().await?;
        let parsed_data = parse_query_result(data)?;
        Ok(parsed_data)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to fetch data",
        )))
    }
}

#[cfg(test)]
#[tokio::test]
async fn query_vizier_test() {
    let adql_query = "SELECT TOP 100 * FROM \"I/261/fonac\"";
    let result = query_vizier(adql_query).await.unwrap();
    assert!(result.len() == 100);
}

pub trait SelectClause {
    fn select(&self) -> &str;
}

impl SelectClause for &str {
    fn select(&self) -> &str {
        self
    }
}

pub trait FromClause {
    fn from(&self) -> &str;
}

impl FromClause for &str {
    fn from(&self) -> &str {
        self
    }
}

pub trait WhereClause {
    fn where_(&self) -> &str;
}

impl WhereClause for &str {
    fn where_(&self) -> &str {
        self
    }
}

impl WhereClause for () {
    fn where_(&self) -> &str {
        ""
    }
}

pub struct Query<S, F, W> {
    select: S,
    from: F,
    where_: W,
}

impl Default for Query<(), (), ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Query<(), (), ()> {
    pub fn new() -> Self {
        Query {
            select: (),
            from: (),
            where_: (),
        }
    }
}

impl<S, F, W> Query<S, F, W> {
    pub fn select<S2: SelectClause>(self, select: S2) -> Query<S2, F, W> {
        Query {
            select,
            from: self.from,
            where_: self.where_,
        }
    }
}

impl<S, F, W> Query<S, F, W> {
    pub fn from<F2: FromClause>(self, from: F2) -> Query<S, F2, W> {
        Query {
            select: self.select,
            from,
            where_: self.where_,
        }
    }
}

impl<S, F, W> Query<S, F, W> {
    pub fn filter<W2: WhereClause>(self, filter: W2) -> Query<S, F, W2> {
        Query {
            select: self.select,
            from: self.from,
            where_: filter,
        }
    }
}

impl<S, F, W> Query<S, F, W>
where
    S: SelectClause,
    F: FromClause,
    W: WhereClause,
{
    pub async fn send(self) -> Result<QueryResult, Box<dyn Error>> {
        let adql_query = format!(
            "{} {} {}",
            self.select.select(),
            self.from.from(),
            self.where_.where_()
        );

        query_vizier(&adql_query).await
    }
}

#[cfg(test)]
#[tokio::test]
async fn query_builder_test() {
    let response = Query::new()
        .select("SELECT TOP 100 *")
        .from("FROM \"I/261/fonac\"")
        .send()
        .await
        .unwrap();

    println!("{:?}", response);

    assert!(response.len() == 100)
}
