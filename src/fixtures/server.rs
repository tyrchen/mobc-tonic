use anyhow::Result;
use std::net::SocketAddr;
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};

use crate::CertConfig;

use super::greeter_server::{Greeter, GreeterServer};
use super::{HelloReply, HelloRequest};

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

pub async fn start_server(addr: &str, cert_config: CertConfig) -> Result<()> {
    let addr = addr.parse().unwrap();
    println!("GreeterServer listening on {}", &addr);

    let svc = MyGreeter::default();
    let identity = Identity::from_pem(cert_config.cert, cert_config.sk);

    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(GreeterServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
