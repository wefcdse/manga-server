use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Error, Response, Server,
};
use tokio::{fs, io};
#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8081);
    let make_svc = make_service_fn(|_| async {
        Ok::<_, Error>(service_fn(|req| async move {
            let response = match manga_server::request_resolver::resolve(req).await {
                Ok(r) => r,
                Err(e) => {
                    let f = fs::read("res/html/404.html").await.unwrap();
                    Response::new(Body::from(f))
                }
            };
            Ok::<_, Error>(response)
        }))
    });
    let server = Server::bind(&addr).serve(make_svc);

    let s = server.await;

    println!("Hello, world!");
}
