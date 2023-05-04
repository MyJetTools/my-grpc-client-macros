use proc_macro::TokenStream;
mod grpc_client;

#[proc_macro_attribute]
pub fn generate_grpc_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    crate::grpc_client::generate(attr, item)
}
