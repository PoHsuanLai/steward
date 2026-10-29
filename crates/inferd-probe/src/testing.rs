//! A fake of the `Http` seam for the probe's tests.

use porter_http::http::{Method, Request, Response};
use porter_http::{Http, HttpError};
use std::collections::HashMap;
use std::sync::Mutex;

pub(crate) fn respond(status: u16, body: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .body(body.as_bytes().to_vec())
        .expect("response")
}

/// An `Http` answering from a table keyed by `METHOD url`; a request not in it is unreachable.
/// Every request is recorded.
#[derive(Default)]
pub(crate) struct Table {
    answers: HashMap<String, Response<Vec<u8>>>,
    pub(crate) seen: Mutex<Vec<String>>,
}

impl Table {
    pub(crate) fn with(mut self, method: Method, url: &str, response: Response<Vec<u8>>) -> Self {
        self.answers.insert(format!("{method} {url}"), response);
        self
    }

    pub(crate) fn get(self, url: &str, response: Response<Vec<u8>>) -> Self {
        self.with(Method::GET, url, response)
    }

    pub(crate) fn seen(&self) -> Vec<String> {
        self.seen.lock().expect("seen").clone()
    }
}

impl Http for Table {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, HttpError> {
        let key = format!("{} {}", request.method(), request.uri());
        self.seen.lock().expect("seen").push(key.clone());
        self.answers
            .get(&key)
            .cloned()
            .ok_or(HttpError::Unreachable)
    }
}
