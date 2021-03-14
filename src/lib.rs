use tonic::{Request, Status};

mod config;
mod error;

#[cfg(test)]
mod fixtures;

pub type InterceptorFn = fn(Request<()>) -> Result<Request<()>, Status>;

pub use config::{CertConfig, ClientConfig};
pub use error::MobcTonicError;

#[allow(dead_code)]
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

#[macro_export]
macro_rules! instantiate_client_pool {
    ($type:ty) => {
        use mobc::{async_trait, Error, Manager, Pool};
        use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

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

            pub async fn get(&self) -> Result<$type, MobcTonicError> {
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
            type Connection = $type;
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
    };
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use fixtures::{
        greeter_client::GreeterClient, start_server, start_server_verify_client_cert, HelloRequest,
    };

    use super::*;

    instantiate_client_pool!(GreeterClient<Channel>);

    #[tokio::test]
    async fn connect_pool_should_work() -> Result<()> {
        let server_cert: CertConfig = toml::from_str(include_str!("fixtures/server.toml")).unwrap();
        tokio::spawn(async move { start_server("0.0.0.0:4000", server_cert).await });
        sleep(10).await;

        let client_config: ClientConfig =
            toml::from_str(include_str!("fixtures/client.toml")).unwrap();

        let pool = ClientPool::new(client_config);
        let mut client = pool.get().await.unwrap();
        let reply = client
            .say_hello(HelloRequest {
                name: "Tyr".to_owned(),
            })
            .await
            .unwrap()
            .into_inner();

        assert_eq!(reply.message, "Hello Tyr!");
        Ok(())
    }

    #[tokio::test]
    async fn connect_pool_with_client_cert_should_work() -> Result<()> {
        tracing_subscriber::fmt::init();
        let server_cert: CertConfig = toml::from_str(include_str!("fixtures/server.toml")).unwrap();
        tokio::spawn(
            async move { start_server_verify_client_cert("0.0.0.0:4001", server_cert).await },
        );
        sleep(10).await;

        let client_config: ClientConfig =
            toml::from_str(include_str!("fixtures/client_with_cert.toml")).unwrap();

        let pool = ClientPool::new(client_config);
        let mut client = pool.get().await.unwrap();
        let reply = client
            .say_hello(HelloRequest {
                name: "Tyr".to_owned(),
            })
            .await
            .unwrap()
            .into_inner();

        assert_eq!(reply.message, "Hello Tyr!");
        Ok(())
    }

    #[tokio::test]
    async fn connect_pool_with_invalid_client_cert_should_fail() -> Result<()> {
        let server_cert: CertConfig = toml::from_str(include_str!("fixtures/server.toml")).unwrap();
        tokio::spawn(
            async move { start_server_verify_client_cert("0.0.0.0:4002", server_cert).await },
        );
        sleep(10).await;

        let client_config: ClientConfig =
            toml::from_str(include_str!("fixtures/client_with_invalid_cert.toml")).unwrap();

        let pool = ClientPool::new(client_config);
        let mut client = pool.get().await.unwrap();
        let reply = client
            .say_hello(HelloRequest {
                name: "Tyr".to_owned(),
            })
            .await;

        assert!(reply.is_err());
        Ok(())
    }

    async fn sleep(duration: u64) {
        tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
    }
}
