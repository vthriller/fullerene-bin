use std::collections::HashMap;

use reqwest::blocking::Client;
use chrono::prelude::*;
use chrono::Duration;
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

pub fn fetch() -> Result<Vec<Vec<(f64, f64)>>, Error> {
	let now = Utc::now();

	let client = Client::new();
	let resp = client.get("http://127.0.0.1:9090/api/v1/query_range")
		.query(&[
			("query", "sum(rate(node_cpu{instance=\"localhost:9100\"} [5m])) by (mode)"),
			("start", &format!("{}", (now - Duration::hours(1)).timestamp())),
			("end", &format!("{}", now.timestamp())),
			("step", "5"),
		])
		.send()?
        .json::<PromQueryRangeStatus>()?;

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
				// XXX unwrap(): we expect valid floats in strings (including "NaN"s)
				.map(|(k, v)| (k, v.parse().unwrap()))
				.collect()
		})
		.collect())
}
