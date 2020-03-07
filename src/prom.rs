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

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		Fetch(e: reqwest::Error) {
			display("failed to fetch data: {}", e)
			from()
		}
		Prom(e: String) {
			display("failed to execute query: {}", e)
		}
	}
}

pub async fn fetch(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Vec<(DateTime<Utc>, f64)>>, Error> {
	let client = Client::new();
	let resp = client.get("http://127.0.0.1:9090/api/v1/query_range")
		.query(&[
			("query", "sum(rate(node_cpu{instance=\"localhost:9100\"} [5m])) by (mode)"),
			("start", &format!("{}", start.timestamp())),
			("end", &format!("{}", end.timestamp())),
			("step", "5"),
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

	// TODO labels
	Ok(data.into_iter()
		.map(|metric| {
			metric.values.into_iter()
				.map(|(k, v)| (
					// don't care about sub-second precision, sorry
					Utc.timestamp(k as i64, 0),
					// XXX unwrap(): we expect valid floats in strings (including "NaN"s)
					v.parse().unwrap()
				))
				.collect()
		})
		.collect())
}
