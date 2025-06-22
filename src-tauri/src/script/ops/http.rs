//! # HTTP (JS API)
//!
//! API for performing HTTP requests from within the JS runtime

use deno_core::op2;
use deno_error::JsErrorBox;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, DurationMilliSeconds, Map};
use std::time::Duration;

/// Request structure from JS to perform an HTTP request
#[serde_as]
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct HttpRequest {
    /// URL for the HTTP request
    url: String,
    /// Method for the HTTP request
    #[serde_as(as = "DisplayFromStr")]
    method: Method,
    /// Body for the HTTP request
    body: Option<HttpRequestBody>,
    /// Headers for the HTTP request
    #[serde_as(as = "Option<Map<_, _>>")]
    headers: Option<Vec<(String, String)>>,
    /// Optional request timeout
    #[serde_as(as = "Option<DurationMilliSeconds>")]
    timeout: Option<Duration>,
    /// Requested format of the response
    response_format: ResponseFormat,
}

#[serde_as]
#[derive(Serialize)]
pub struct HttpResponse {
    /// HTTP response status code
    status: u16,

    /// Response headers
    #[serde_as(as = "serde_with::Map<_, _>")]
    headers: Vec<(String, String)>,

    /// Body of the response
    body: HttpResponseBody,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Json,
    Raw,
    #[default]
    Text,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum HttpRequestBody {
    // Raw binary data from JS
    Raw(Vec<u8>),
    // Text body content
    Text(String),
    // JSON body content
    Json(serde_json::Value),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum HttpResponseBody {
    Json(serde_json::Value),
    Raw(Vec<u8>),
    Text(String),
}

/// Operation for performing a GET request to a specific URL from JS
#[op2(async)]
#[serde]
pub async fn op_http_request(#[serde] req: HttpRequest) -> Result<HttpResponse, JsErrorBox> {
    // Get or create HTTP client
    let client = reqwest::Client::new();

    let mut builder = client.request(req.method, req.url);

    // Append request body
    if let Some(body) = req.body {
        match body {
            HttpRequestBody::Raw(vec) => {
                builder = builder.body(vec);
            }
            HttpRequestBody::Text(text) => builder = builder.body(text),
            HttpRequestBody::Json(value) => builder = builder.json(&value),
        }
    }

    // Append request headers
    if let Some(headers) = req.headers {
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
    }

    // Set request timeout
    if let Some(timeout) = req.timeout {
        builder = builder.timeout(timeout);
    }

    let response = builder.send().await.map_err(|err| {
        log::error!("failed to send http request: {err}");
        JsErrorBox::generic("failed to send http request")
    })?;

    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .into_iter()
        .map(|(key, value)| {
            let key = key.to_string();
            let value = value.to_str()?;
            anyhow::Ok((key, value.to_string()))
        })
        .filter_map(|value| value.ok())
        .collect();

    let body = match req.response_format {
        ResponseFormat::Json => {
            let value: serde_json::Value = response.json().await.map_err(|err| {
                log::error!("failed to parse json response: {err}");
                JsErrorBox::generic("failed to parse json response")
            })?;
            HttpResponseBody::Json(value)
        }
        ResponseFormat::Raw => {
            let value = response.bytes().await.map_err(|err| {
                log::error!("failed to read response bytes: {err}");
                JsErrorBox::generic("failed to read response bytes")
            })?;
            HttpResponseBody::Raw(value.into())
        }
        ResponseFormat::Text => {
            let value = response.text().await.map_err(|err| {
                log::error!("failed to read response text: {err}");
                JsErrorBox::generic("failed to read response text")
            })?;
            HttpResponseBody::Text(value)
        }
    };

    Ok(HttpResponse {
        status,
        body,
        headers,
    })
}
