# mobc-tonic

A connection pool for tonic GRPC client.

Usage:

First, instantiate the implementation for your client:

```rust
use mobc_tonic::{
    instantiate_client_pool, ClientConfig, Error, InterceptorFn, Manager, MobcTonicError, Pool,
};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

use gen::greeter_client::GreeterClient;

instantiate_client_pool!(GreeterClient<Channel>);
```

This will generate the connection pool manager for your GRPC client.

Then you could use the generated `ClientPool`:

```rust
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
```

## License

`mobc-tonic` is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

Copyright 2021 Tyr Chen
