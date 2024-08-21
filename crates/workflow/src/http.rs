use crate::{
    bindings::{
        self,
        component::workflow::{
            abi::{Content, GuestToHost, HostToGuest},
            http,
        },
        WorkflowError,
    },
    Result,
};
use ::http::{StatusCode, Version};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use uuid::Uuid;

/// Performs a GET request to the specified path.
///
/// # Arguments
///
/// * `path` - The URL path for the request.
/// * `headers` - Optional headers to include in the request.
///
/// # Returns
///
/// A `Result` containing the `Response` if successful, or an `Error` if the request fails.
#[allow(unused)]
pub fn get(path: impl Into<String>, headers: Option<HashMap<&str, &str>>) -> Result<Response> {
    request(http::Method::Get, path, headers, None)
}

/// Performs a POST request to the specified path.
///
/// # Arguments
///
/// * `path` - The URL path for the request.
/// * `headers` - Optional headers to include in the request.
/// * `body` - Optional body content for the request.
///
/// # Returns
///
/// A `Result` containing the `Response` if successful, or an `Error` if the request fails.
#[allow(unused)]
pub fn post(
    path: impl Into<String>,
    headers: Option<HashMap<&str, &str>>,
    body: Option<Vec<u8>>,
) -> Result<Response> {
    request(http::Method::Post, path, headers, body)
}

/// Performs a DELETE request to the specified path.
///
/// # Arguments
///
/// * `path` - The URL path for the request.
/// * `headers` - Optional headers to include in the request.
/// * `body` - Optional body content for the request.
///
/// # Returns
///
/// A `Result` containing the `Response` if successful, or an `Error` if the request fails.
#[allow(unused)]
pub fn delete(
    path: impl Into<String>,
    headers: Option<HashMap<&str, &str>>,
    body: Option<Vec<u8>>,
) -> Result<Response> {
    request(http::Method::Delete, path, headers, body)
}

/// Internal function to perform an HTTP request.
///
/// # Arguments
///
/// * `method` - The HTTP method for the request.
/// * `path` - The URL path for the request.
/// * `headers` - Optional headers to include in the request.
/// * `body` - Optional body content for the request.
///
/// # Returns
///
/// A `Result` containing the `Response` if successful, or an `Error` if the request fails.
fn request(
    method: http::Method,
    path: impl Into<String>,
    headers: Option<HashMap<&str, &str>>,
    body: Option<Vec<u8>>,
) -> Result<Response> {
    let request = GuestToHost::HttpRequest(http::Request {
        method,
        path: path.into(),
        body,
        headers: headers
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| http::Header {
                key: k.into(),
                value: v.into(),
            })
            .collect::<Vec<_>>(),
    });

    match bindings::call(&request) {
        HostToGuest {
            id,
            content: Content::HttpResponse(response),
        } => Ok(response
            .map(|response| Response::from_response(id.clone(), response))
            .map_err(|error| Error::from_error(id.clone(), error))?),
        m => {
            log::error!("expected Content::HttpResponse got {:?}", m);
            unreachable!()
        }
    }
}

/// Represents an HTTP response.
pub struct Response {
    id: Uuid,
    url: String,
    version: Version,
    status: ::http::StatusCode,
    headers: HashMap<String, String>,
    content_length: Option<u64>,
    body: Vec<u8>,
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Response {
            url,
            version,
            status,
            headers,
            content_length,
            ..
        } = self;

        f.debug_struct("Response")
            .field("url", &url)
            .field("version", &version)
            .field("status", &status)
            .field("headers", &headers)
            .field("content_length", &content_length)
            .finish()
    }
}

#[allow(dead_code)]
impl Response {
    /// Creates a new `Response` from an `http::Response`.
    fn from_response(id: String, response: http::Response) -> Self {
        Response {
            id: Uuid::parse_str(&id).unwrap(),
            url: response.url,
            version: match response.http_version {
                http::Version::HttpZeroNine => Version::HTTP_09,
                http::Version::HttpOneZero => Version::HTTP_10,
                http::Version::HttpOneOne => Version::HTTP_11,
                http::Version::HttpTwoZero => Version::HTTP_2,
                http::Version::HttpThreeZero => Version::HTTP_3,
            },
            status: StatusCode::from_u16(response.status).unwrap(),
            headers: response
                .headers
                .into_iter()
                .map(|header| (header.key, header.value))
                .collect(),
            content_length: response.content_length,
            body: response.body,
        }
    }

    /// Get the `id` of this `Response`.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the `StatusCode` of this `Response`.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the HTTP `Version` of this `Response`.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the `Headers` of this `Response`.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a mutable reference to the `Headers` of this `Response`.
    pub fn headers_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.headers
    }

    /// Get the content-length of this response, if known.
    ///
    /// Reasons it may not be known:
    ///
    /// - The server didn't send a `content-length` header.
    /// - The response is compressed and automatically decoded (thus changing
    ///   the actual decoded length).
    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    /// Get the final `Url` of this `Response`.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Turn a response into an error if the server returned an error.
    pub fn error_for_status(self) -> crate::Result<Self> {
        let status = self.status();
        if status.is_client_error() || status.is_server_error() {
            Err(WorkflowError::new(
                Some(self.id.to_string()),
                status
                    .canonical_reason()
                    .map(|reason| reason.to_string())
                    .unwrap_or(status.to_string()),
            ))
        } else {
            Ok(self)
        }
    }

    /// Get the full response text.
    pub fn text(self) -> Result<String> {
        let text = String::from_utf8_lossy(&self.body);
        Ok(text.into_owned())
    }

    /// Try to deserialize the response body as JSON.
    pub fn json<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_slice(&self.body)
            .map_err(|err| WorkflowError::new(Some(self.id.to_string()), err.to_string()))
    }

    /// Get the full response body as `Bytes`.
    pub fn bytes(self) -> Result<bytes::Bytes> {
        Ok(bytes::Bytes::from(self.body))
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Error {
    pub id: Uuid,
    pub url: Option<String>,
    pub kind: Option<Kind>,
}

impl Error {
    pub fn from_error(id: String, error: http::Error) -> Self {
        Self {
            id: Uuid::parse_str(&id).unwrap(),
            url: error.url,
            kind: error.kind.map(|kind| match kind {
                http::Kind::Builder => Kind::Builder,
                http::Kind::Request => Kind::Request,
                http::Kind::Redirect => Kind::Redirect,
                http::Kind::Status(code) => Kind::Status(StatusCode::from_u16(code).unwrap()),
                http::Kind::Body => Kind::Body,
                http::Kind::Decode => Kind::Decode,
                http::Kind::Upgrade => Kind::Upgrade,
            }),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Kind {
    Builder,
    Request,
    Redirect,
    Status(StatusCode),
    Body,
    Decode,
    Upgrade,
}
