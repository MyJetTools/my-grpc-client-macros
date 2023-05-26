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