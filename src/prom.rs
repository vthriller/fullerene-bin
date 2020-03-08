use std::collections::HashMap;

use reqwest::Client;
use chrono::prelude::*;
use serde::Deserialize;

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[derive(Deserialize)]
#[serde(tag = "status")]
enum PromQueryRangeStatus {
	success { data: PromQueryRangeResult },
	error {
		errorType: String,
		error: String,
	},
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
#[serde(tag = "resultType")]
enum PromQueryRangeResult {
	matrix { result: Vec<PromMetric> },
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
#[serde(tag = "resultType")]
struct PromMetric {
	metric: HashMap<String, String>,
	values: Vec<(f64, String)>,
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("failed to fetch data: {0}")]
	Fetch(#[from] reqwest::Error),
	#[error("failed to execute query: {0}")]
	Prom(String),
}

pub struct Metric {
	pub labels: HashMap<String, String>,
	pub data: Vec<(DateTime<Utc>, f64)>,
}

pub async fn fetch(query: &str, start: DateTime<Utc>, end: DateTime<Utc>, pitch: u32) -> Result<Vec<Metric>, Error> {
	let client = Client::new();
	let resp = client.get("http://127.0.0.1:9090/api/v1/query_range")
		.query(&[
			("query", query),
			("start", &format!("{}", start.timestamp())),
			("end", &format!("{}", end.timestamp())),
			("step", &format!("{}", pitch)),
		])
		.send()
		.await?
        .json::<PromQueryRangeStatus>()
		.await?;

	let data = match resp {
		PromQueryRangeStatus::success { data } => match data {
			PromQueryRangeResult::matrix { result } => result,
		},
		PromQueryRangeStatus::error { error, .. } => {
			return Err(Error::Prom(error));
		},
	};

	Ok(data.into_iter()
		.map(|metric| {
			Metric {
				labels: metric.metric,
				data: metric.values.into_iter()
					.map(|(k, v)| (
						// don't care about sub-second precision, sorry
						Utc.timestamp(k as i64, 0),
						// XXX unwrap(): we expect valid floats in strings (including "NaN"s)
						v.parse().unwrap()
					))
					.collect()
			}
		})
		.collect())
}
