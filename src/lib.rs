use std::convert::Infallible;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper::{Body, Request, Response};

use log::{debug, error, info};

extern crate serde_json;

/// Errors that can be returned by the function handler
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A type for handling incoming requests and producing responses
pub trait Handler<Req, Resp> {
    /// Errors returned by the handler
    type Error;
    /// Handle the incoming request
    fn call(self, req: Req) -> Result<Resp, Self::Error>;
}

/// [`Handler`] implementation for `FnOnce`
impl<Req, Resp, Error, F> Handler<Req, Resp> for F
where
    F: FnOnce(Req) -> Result<Resp, Error>,
{
    type Error = Error;
    fn call(self, req: Req) -> Result<Resp, Self::Error> {
        (self)(req)
    }
}

pub async fn run<Req, Resp, F>(handler: F) -> Result<(), Error>
where
    F: Handler<Req, Resp> + Clone + Send + Sync + 'static,
    <F as Handler<Req, Resp>>::Error: std::fmt::Display,
    Req: for<'de> Deserialize<'de> + Send,
    Resp: Serialize,
    Error: Into<crate::Error>,
{
    let make_service = make_service_fn(move |conn: &AddrStream| {
        let client_addr = conn.remote_addr();

        let handler = handler.clone();
        let service = service_fn(move |req: Request<Body>| {
            let handler = handler.clone();
            async move {
                debug!("New request from {}", client_addr);
                let body = req.into_body();
                let body = hyper::body::to_bytes(body).await;
                if let Err(_) = body {
                    error!("Could not parse body");
                    return Err("Runtime error");
                }

                let body = serde_json::from_slice(&body.unwrap());
                if let Err(err) = body {
                    error!("Could not de-serialize request: {}", err);
                    return Ok(Response::new(Body::from("Runtime error")));
                }

                match handler.call(body.unwrap()) {
                    Ok(resp) => match serde_json::to_vec(&resp) {
                        Ok(resp) => Ok(Response::new(Body::from(resp))),
                        Err(_) => {
                            error!("Could not serialize response");
                            return Ok(Response::new(Body::from("Runtime error")));
                        }
                    },
                    Err(err) => Ok(Response::new(Body::from(format!("{}", err)))),
                }
            }
        });

        async move { Ok::<_, Infallible>(service) }
    });

    info!("Starting service");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make_service);

    info!("Server awaiting for requests at {}", addr);
    server.await?;

    Ok(())
}
