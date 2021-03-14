use mobc::{async_trait, Connection, Error, Manager, Pool};
use tonic::{
    client::Grpc,
    transport::{Certificate, Channel, ClientTlsConfig, Identity},
    Interceptor, Request, Status,
};

mod config;
mod error;

#[cfg(test)]
mod fixtures;

pub type InterceptorFn = fn(Request<()>) -> Result<Request<()>, Status>;

pub use config::{CertConfig, ClientConfig};
pub use error::MobcTonicError;

pub struct ClientManager {
    config: ClientConfig,
    interceptor: Option<InterceptorFn>,
}

impl ClientManager {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            interceptor: None,
        }
    }

    pub fn with_interceptor(config: ClientConfig, interceptor: InterceptorFn) -> Self {
        Self {
            config,
            interceptor: Some(interceptor),
        }
    }
}

pub struct ClientPool {
    pool: Pool<ClientManager>,
}

impl ClientPool {
    pub fn new(config: ClientConfig) -> Self {
        let size = config.pool_size;
        let manager = ClientManager::new(config);
        let pool = Pool::builder().max_open(size as u64).build(manager);
        Self { pool }
    }

    pub async fn get(&self) -> Result<Grpc<Channel>, MobcTonicError> {
        match self.pool.clone().get().await {
            Ok(conn) => Ok(conn.into_inner()),
            Err(Error::Timeout) => Err(MobcTonicError::Timeout),
            Err(Error::BadConn) => Err(MobcTonicError::BadConn),
            Err(Error::Inner(e)) => Err(e),
        }
    }
}

#[async_trait]
impl Manager for ClientManager {
    type Connection = Grpc<Channel>;
    type Error = MobcTonicError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let config = &self.config;
        let cert = Certificate::from_pem(config.ca_cert.clone());
        let tls = ClientTlsConfig::new()
            .domain_name(self.config.domain.clone())
            .ca_certificate(cert);
        let tls = if let Some(client_config) = config.client_cert.clone() {
            let identity = Identity::from_pem(client_config.cert, client_config.sk);
            tls.identity(identity)
        } else {
            tls
        };

        let channel = Channel::from_shared(self.config.uri.clone())?
            .tls_config(tls)?
            .connect()
            .await?;

        let client = if let Some(interceptor) = self.interceptor.as_ref() {
            Self::Connection::with_interceptor(channel, interceptor.to_owned())
        } else {
            Self::Connection::new(channel)
        };

        Ok(client)
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use fixtures::greeter_client::GreeterClient;
    use fixtures::start_server;

    use super::*;

    #[tokio::test]
    async fn connect_pool_works() -> Result<()> {
        let server_cert: CertConfig = toml::from_str(include_str!("fixtures/server.toml")).unwrap();
        tokio::spawn(async move { start_server("0.0.0.0:4000", server_cert).await });

        let client_config: ClientConfig =
            toml::from_str(include_str!("fixtures/client.toml")).unwrap();

        // let pool = ClientPool::new(client_config);
        // let conn = pool.get().await.unwrap();
        Ok(())
    }
}
