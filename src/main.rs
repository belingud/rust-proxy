mod helpers;

use crate::helpers::{check_address_block};
use axum::{
    body::{self, Body},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use hyper::upgrade::Upgraded;
use std::net::SocketAddr;
use axum::extract::ConnectInfo;
use tokio::net::TcpStream;
use tower::{make::Shared, ServiceExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let router_svc = Router::new().route("/", get(|| async { "Hello, World!" }));

    let service = tower::service_fn(move |req: Request<Body>| {
        let router_svc = router_svc.clone();
        async move {
            let client_addr = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());
            tracing::info!("Client address: {}", client_addr);
            if req.method() == Method::CONNECT {
                proxy(req, client_addr).await
            } else {
                router_svc.oneshot(req).await.map_err(|err| match err {})
            }
        }
    });
    // read port from env
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(Shared::new(service))
        .await
        .unwrap();
}

async fn proxy(req: Request<Body>, client_addr: String) -> Result<Response, hyper::Error> {
    tracing::info!(?req);

    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        if check_address_block(&host_addr) == true {
            //  return 400 if blocked
            tracing::info!("This site is blocked: {}", host_addr);
            return Ok((StatusCode::BAD_REQUEST, "This site is blocked").into_response());
        }
        // let client_addr = get_ip_addr(req.extensions().get::<SocketAddr>());
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, host_addr, client_addr).await {
                        tracing::warn!("server io error: {}", e);
                    };
                }
                Err(e) => tracing::warn!("upgrade error: {}", e),
            }
        });
        Ok(Response::new(body::boxed(body::Empty::new())))
    } else {
        tracing::warn!("CONNECT host is not socket addr: {:?}", req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        )
            .into_response())
    }
}

async fn tunnel(mut upgraded: Upgraded, addr: String, client_addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    // get request client ipv4 address

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    tracing::info!(
        "Client {} wrote {} bytes and received {} bytes",
        client_addr,
        from_client,
        from_server
    );

    Ok(())
}
