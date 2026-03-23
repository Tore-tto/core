pub mod beldexd;
pub mod wallet;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct Request<T> {
    /// JSON RPC version, we hard cod this to 2.0.
    jsonrpc: String,
    /// Client controlled identifier, we hard code this to 1.
    id: String,
    /// The method to call.
    method: String,
    /// The method parameters.
    params: T,
}

/// JSON RPC request.
impl<T> Request<T> {
    pub fn new(method: &str, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id: "1".to_owned(),
            method: method.to_owned(),
            params,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
}

/// JSON RPC response.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Response<T> {
    pub id: serde_json::Value,
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<ResponseError>,
}

impl<T> Response<T> {
    pub fn into_result(self) -> Result<T> {
        if let Some(error) = self.error {
            anyhow::bail!("RPC error: {} (code {})", error.message, error.code);
        }
        self.result.context("Missing result field in RPC response")
    }
}
