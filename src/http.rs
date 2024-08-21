use std::str::FromStr;

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName};
use uuid::Uuid;

use crate::{
    guest::{
        component::workflow::{
            abi::Content,
            http::{Error, Header, Kind, Method, Response, Version},
        },
        GuestToHost, HostToGuest,
    },
    workflow::State,
};

pub async fn call(state: &mut State, request: GuestToHost) -> Result<HostToGuest> {
    match request {
        GuestToHost::HttpRequest(request) => {
            state
                .retrieve_or_else(|| {
                    async {
                        let client = reqwest::Client::new();

                        let mut headers = HeaderMap::with_capacity(request.headers.len());
                        for header in &request.headers {
                            headers
                                .insert(HeaderName::from_str(&header.key)?, header.value.parse()?);
                        }

                        // Build the request.
                        let mut req = match request.method {
                            Method::Get => client.get(&request.path).headers(headers),
                            Method::Post => client.post(&request.path).headers(headers),
                            Method::Delete => client.delete(&request.path).headers(headers),
                        };

                        if let Some(body) = request.body.clone() {
                            req = req.body(body);
                        }

                        // Execute the request.
                        Ok(HostToGuest {
                            id: Uuid::new_v4().into(),
                            content: Content::HttpResponse(match req.send().await {
                                Ok(resp) => Ok(Response {
                                    status: resp.status().as_u16(),
                                    http_version: match resp.version() {
                                        reqwest::Version::HTTP_09 => Version::HttpZeroNine,
                                        reqwest::Version::HTTP_10 => Version::HttpOneZero,
                                        reqwest::Version::HTTP_11 => Version::HttpOneOne,
                                        reqwest::Version::HTTP_2 => Version::HttpTwoZero,
                                        reqwest::Version::HTTP_3 => Version::HttpThreeZero,
                                        _ => unimplemented!(),
                                    },
                                    headers: resp
                                        .headers()
                                        .iter()
                                        .map(|(header_name, header_value)| {
                                            Ok(Header {
                                                key: header_name.to_string(),
                                                value: header_value
                                                    .to_str()
                                                    .map(|s| s.to_string())?,
                                            })
                                        })
                                        .collect::<Result<Vec<_>>>()?,
                                    content_length: resp.content_length(),
                                    url: resp.url().to_string(),
                                    body: resp.bytes().await?.to_vec(),
                                }),
                                Err(err) => Err(Error {
                                    url: err.url().map(|url| url.to_string()),
                                    kind: if err.is_builder() {
                                        Some(Kind::Builder)
                                    } else if err.is_request() {
                                        Some(Kind::Request)
                                    } else if err.is_body() {
                                        Some(Kind::Body)
                                    } else if err.is_decode() {
                                        Some(Kind::Decode)
                                    } else if err.is_redirect() {
                                        Some(Kind::Redirect)
                                    } else if err.is_status() {
                                        Some(Kind::Status(err.status().unwrap().as_u16()))
                                    } else {
                                        None
                                    },
                                }),
                            }),
                        })
                    }
                })
                .await
        }
        _ => unreachable!(),
    }
}
