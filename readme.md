The most general Use case of auto generating the GRPC Client with Retries is here:

```rust

pub const GRPC_SERVICE_NAME: &str = "keyvalue";

#[generate_grpc_client(
    proto_file: "./proto/KeyValueFlows.proto",
    timeout_sec: 5,
    retries: 3,
    overrides: [
        {fn_name:"Get", retries:2}
    ]
)]
pub struct KeyValueGrpcClient {
    channel: my_grpc_extensions::GrpcChannel<TGrpcService>,
}

```

As Well use generated code to read settings:


```rust
#[async_trait::async_trait]
impl my_grpc_extensions::GrpcClientSettings for SettingsReader {
    async fn get_grpc_url(&self, name: &'static str) -> String {
        if name == KeyValueGrpcClient::get_service_name() {
            let read_access = self.settings.read().await;
            return read_access.key_value_grpc_url.clone();
        }

        panic!("Unknown grpc service name: {}", name)
    }
}


```