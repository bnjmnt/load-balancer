use crate::balancer::Balancer;
use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;

pub type ProxyClient = Client<HttpConnector, Incoming>;
type ResponseBody = BoxBody<Bytes, hyper::Error>;

fn error_body(msg: &'static str) -> ResponseBody {
    Full::new(Bytes::from(msg))
        .map_err(|never| match never {})
        .boxed()
}

pub async fn handle_request(
    req: Request<Incoming>,
    balancer: Arc<Balancer>,
    client: ProxyClient,
) -> Result<Response<ResponseBody>, hyper::Error> {
    let (backend_addr, _guard) = match balancer.pick() {
        Some(picked) => picked,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(error_body("no healthy backends available"))
                .unwrap());
        }
    };

    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let uri_string = format!("http://{backend_addr}{path_and_query}");
    let uri: hyper::Uri = match uri_string.parse() {
        Ok(u) => u,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(error_body("invalid backend uri"))
                .unwrap());
        }
    };

    let (mut parts, body) = req.into_parts();
    parts.uri = uri;
    let new_req = Request::from_parts(parts, body);

    match client.request(new_req).await {
        Ok(resp) => Ok(resp.map(|b| b.boxed())),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(error_body("backend request failed"))
            .unwrap()),
    }
}
