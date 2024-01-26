use crate::metric::Tag;
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, time::Duration};

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
    type QueryParams: Serialize;

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
    fn query_params(&self) -> Option<&Self::QueryParams> {
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
pub struct ApiRequest<Response, QueryParams, Body> {
    pub path: &'static str,
    pub method: reqwest::Method,
    pub tag_method: &'static str,
    pub body: Option<Body>,
    pub query_params: Option<QueryParams>,
    pub response: PhantomData<Response>,
}

impl<Response, QueryParams, Body> RestRequest for ApiRequest<Response, QueryParams, Body>
where
    Response: DeserializeOwned,
    QueryParams: Serialize,
    Body: Serialize,
{
    type Response = Response; // Define Response type
    type QueryParams = QueryParams; // FetchBalances does not require any QueryParams
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

#[derive(Debug)]
pub struct SimpleGetRequest<Response> {
    pub path: &'static str,
    pub tag_method: &'static str,
    pub response: PhantomData<Response>,
}

impl<Response> RestRequest for SimpleGetRequest<Response>
where
    Response: DeserializeOwned,
{
    type Response = Response; // Define Response type
    type QueryParams = (); // FetchBalances does not require any QueryParams
    type Body = (); // FetchBalances does not require any Body

    fn path(&self) -> &'static str {
        self.path
    }

    fn metric_tag(&self) -> Tag {
        Tag::new("method", self.tag_method)
    }
}
