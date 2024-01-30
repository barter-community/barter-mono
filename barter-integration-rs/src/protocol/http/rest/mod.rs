use crate::metric::Tag;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, marker::PhantomData, time::Duration};

/// Configurable [`client::RestClient`] capable of executing signed [`RestRequest`]s and parsing
/// responses.
pub mod client;

/// Default Http [`reqwest::Request`] timeout Duration.
const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Http REST request that can be executed by a [`RestClient`](self::client::RestClient).
pub trait RestRequest {
    /// Expected response type if this request was successful.
    type Response: DeserializeOwned;

    /// Serialisable query parameters type - use unit struct () if not required for this request.
    // type QueryParams: Serialize;

    /// Serialisable Body type - use unit struct () if not required for this request.
    type Body: Serialize;

    /// Additional [`Url`](url::Url) path to the resource.
    fn path(&self) -> &'static str;

    /// Http [`reqwest::Method`] of this request.
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }

    /// [`Metric`](crate::metric::Metric) [`Tag`](crate::metric::Tag) that identifies this request.
    fn metric_tag(&self) -> Tag;

    /// Optional query parameters for this request.
    fn query_params(&self) -> Option<&QueryParams> {
        None
    }

    /// Optional Body for this request.
    fn body(&self) -> Option<&Self::Body> {
        None
    }

    /// Http request timeout [`Duration`].
    fn timeout(&self) -> Duration {
        DEFAULT_HTTP_REQUEST_TIMEOUT
    }
}

#[derive(Debug)]
pub struct ApiRequest<Response, Body> {
    pub path: &'static str,
    pub method: reqwest::Method,
    pub tag_method: &'static str,
    pub body: Option<Body>,
    pub query_params: Option<QueryParams>,
    pub response: PhantomData<Response>,
}

impl<Response, Body> ApiRequest<Response, Body> {
    pub const fn new(
        path: &'static str,
        method: reqwest::Method,
        tag_method: &'static str,
    ) -> Self {
        Self {
            path,
            method,
            tag_method,
            body: None,
            query_params: None,
            response: PhantomData,
        }
    }
}

impl<Response, Body> RestRequest for ApiRequest<Response, Body>
where
    Response: DeserializeOwned,
    Body: Serialize,
{
    type Response = Response; // Define Response type
    type Body = Body; // FetchBalances does not require any Body

    fn path(&self) -> &'static str {
        self.path
    }

    fn method(&self) -> reqwest::Method {
        self.method.clone()
    }

    fn metric_tag(&self) -> Tag {
        Tag::new("method", self.tag_method)
    }

    fn body(&self) -> Option<&Body> {
        match self.body {
            Some(ref body) => Some(body),
            None => None,
        }
    }

    fn query_params(&self) -> Option<&QueryParams> {
        match self.query_params {
            Some(ref query_params) => Some(query_params),
            None => None,
        }
    }
}

type QueryKey = &'static str;

#[derive(Debug, Serialize)]
pub struct QueryParams(Vec<(String, String)>);

impl QueryParams {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn add_kv(&mut self, key: QueryKey, value: impl Display + Sized) {
        self.0.push((key.to_owned(), value.to_string()));
    }
}
