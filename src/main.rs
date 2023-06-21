use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Error, Response, Server,
};
use manga_server::{
    copy_manga, dmzj, eh,
    manga_list::{Backend, CONFIG},
};
use tokio::fs;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), CONFIG.port);
    let make_svc = make_service_fn(|_| async {
        Ok::<_, Error>(service_fn(|req| async move {
            let response = match {
                match CONFIG.backend {
                    Backend::DMZJ => {
                        manga_server::request_resolver::resolve::<dmzj::Dmzj>(req).await
                    }
                    Backend::CopyManga => {
                        manga_server::request_resolver::resolve::<copy_manga::CopyManga>(req).await
                    }
                    Backend::Eh => manga_server::request_resolver::resolve::<eh::Eh>(req).await,
                }
            } {
                Ok(r) => r,
                Err(_e) => {
                    let f = fs::read("res/html/404.html").await.unwrap();
                    Response::new(Body::from(f))
                }
            };
            Ok::<_, Error>(response)
        }))
    });
    let server = Server::bind(&addr).serve(make_svc);

    let _s = server.await;

    println!("Hello, world!");
}
