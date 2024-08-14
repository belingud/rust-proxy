use hyper::{Body, Request, Response};

async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    tracing::info!(?req);

    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        if check_address_block(&host_addr) == true {
            //  return 400 if blocked
            tracing::info!("This site is blocked: {}", host_addr);
            return Ok((StatusCode::BAD_REQUEST, "This site is blocked").into_response());
        }
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, host_addr).await {
                        tracing::warn!("server io error: {}", e);
                    };
                }
                Err(e) => tracing::warn!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(body::boxed(body::Empty::new())))
    } else {
        // Handle cases where there is no authority part in the URI
        Ok((StatusCode::BAD_REQUEST, "Invalid request").into_response())
    }
}

#[tokio::main]
async fn main() {
    // read port from env
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = ([0, 0, 0, 0], port).into();
    let service = tower::service_fn(proxy);
    let server = hyper::Server::bind(&addr).serve(service);
    println!("Listening on {}", addr);
    server.await.unwrap();
}