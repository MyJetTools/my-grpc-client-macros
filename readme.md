client generator depends on my-grpc-extensions version 0.4.0 or higher

```toml
my-grpc-extensions = { tag = "0.4.0", git = "https://github.com/MyJetTools/my-grpc-extensions.git" }
```

The most general Use case of auto generating the GRPC Client with Retries is here:

```rust

use my_grpc_client_macros::generate_grpc_client;

#[generate_grpc_client(
    proto_file: "./proto/KeyValueFlows.proto",
    crate_ns: "crate::keyvalue_grpc",
    retries: 3,
    request_timeout_sec: 5,
    ping_timeout_sec: 5,
    ping_interval_sec: 5,
    overrides: [
        {fn_name:"Get", retries:2}
    ]
)]
pub struct KeyValueGrpcClient {
    channel: my_grpc_extensions::GrpcChannel<TGrpcService>,
}


```

### Parameters description:

* request_timeout_sec: timeout of any grpc request;
* ping_timeout_sec: timeout of background ping request, which is used to determine channel disconnect in the background;
* ping_interval_sec: how frequent background ping request a repeated on loop;
* crate_ns: name of the module which is used to plug grpc code generated by tonic;
* proto_file: path to a proto file;
* retires: amount of retries, which is used to retry request once disconnect is happened.

### PING Loop

Ping loop happens in a background to detect channel disconnects and reconnect them in the background.

### Settings setup
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
