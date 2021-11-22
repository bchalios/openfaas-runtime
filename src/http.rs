use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper::{Body, Response};

use log::{debug, error, info};

use serde::Serialize;

use crate::{Error, Handler};

pub type Request = hyper::Request<Body>;

pub async fn run<Resp, F>(handler: F)
where
    F: Handler<Request, Resp> + Clone + Send + Sync + 'static,
    <F as Handler<Request, Resp>>::Error: std::fmt::Display,
    <F as Handler<Request, Resp>>::Fut: Send,
    Resp: Serialize,
    Error: Into<crate::Error> + Send,
{
    let make_service = make_service_fn(move |conn: &AddrStream| {
        let client_addr = conn.remote_addr();

        let handler = handler.clone();
        let service = service_fn(move |req: Request| {
            let handler = handler.clone();

            async move {
                debug!("New HTTP request from {}", client_addr);
                let resp = handler.call(req).await;
                match resp {
                    Ok(response) => match serde_json::to_vec(&response) {
                        Ok(resp) => Ok::<_, Infallible>(Response::new(Body::from(resp))),
                        Err(err) => {
                            error!("Could not serialize response");
                            Ok(Response::new(Body::from(format!("{}", err))))
                        }
                    },
                    Err(err) => Ok(Response::new(Body::from(format!("Runtime error: {}", err)))),
                }
            }
        });

        async move { Ok::<_, Infallible>(service) }
    });

    info!("Starting service");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make_service);

    info!("Server awaiting for requests at {}", addr);
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}
