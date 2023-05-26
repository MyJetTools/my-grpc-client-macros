

use std::str::FromStr;

use proc_macro::TokenStream;

use types_reader::{ ParamsListAsTokens};

use crate::grpc_client::{fn_override::FnOverride, proto_file_reader::into_snake_case};

use super::proto_file_reader::ProtoServiceDescription;

pub fn generate(
    attr: TokenStream,
    input: TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {

    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let attr_input: proc_macro2::TokenStream = attr.into();

    
    let attributes = ParamsListAsTokens::new(attr_input)?;

    let timeout_sec = attributes.get_named_param("timeout_sec")?;
    let timeout_sec = timeout_sec.get_number_value_token()?;


    let proto_file = attributes.get_named_param("proto_file")?;
    let proto_file = proto_file.get_str_value()?;
    let proto_file = ProtoServiceDescription::read_proto_file(proto_file);

    let grpc_service_name = &proto_file.service_name;
    let grpc_service_name_token = proto_file.get_service_name_as_token();

    let interfaces = super::generate_interfaces_implementations(struct_name, &proto_file);

    let retries = attributes.get_named_param("retries")?;
    let retries = retries.get_number_value()?;


    let overrides = FnOverride::new(&attributes)?;


    let crate_ns = attributes.get_named_param("crate_ns")?.get_str_value_token()?;
    let mut use_name_spaces = Vec::new();
    use_name_spaces.push(quote::quote! {#crate_ns::*});

    let ns_of_client = format!("{}_client", into_snake_case(&grpc_service_name));
    let ns_of_client = proc_macro2::TokenStream::from_str(ns_of_client.as_str()).unwrap();
    use_name_spaces.push(quote::quote! {#crate_ns::#ns_of_client::*});
    


    for (override_fn_name, fn_override) in &overrides{
        if !proto_file.has_method(override_fn_name){
            return Err(syn::Error::new_spanned(
                fn_override.token_stream.clone(),
                format!("Method {} is not found in proto file for service {}", override_fn_name, grpc_service_name),
            ));
        }
    }
    
    let grpc_methods = super::generate_grpc_methods(&proto_file, retries as usize, &overrides);

    
    

    Ok(quote::quote! {
        #(*use_name_spaces);

        type TGrpcService = #grpc_service_name_token<tonic::codegen::InterceptedService<tonic::transport::Channel, my_grpc_extensions::GrpcClientInterceptor>>;

        struct MyGrpcServiceFactory;

        #[async_trait::async_trait]
        impl my_grpc_extensions::GrpcServiceFactory<TGrpcService> for MyGrpcServiceFactory {
        fn create_service(&self, channel: tonic::transport::Channel, ctx: &my_telemetry::MyTelemetryContext) -> TGrpcService {
            #grpc_service_name_token::with_interceptor(
              channel,
              my_grpc_extensions::GrpcClientInterceptor::new(ctx.clone()),
            )
        }

        fn get_service_name(&self) -> &'static str {
            #struct_name::get_service_name()
        }

        async fn ping(&self, mut service: TGrpcService) {
           service.ping(()).await.unwrap();
        }
      }

      #ast

      impl #struct_name{
        pub fn new(get_grpc_address: std::sync::Arc<dyn my_grpc_extensions::GrpcClientSettings + Send + Sync + 'static>,) -> Self {
            Self {
                channel: my_grpc_extensions::GrpcChannel::new(
                    get_grpc_address,
                    std::sync::Arc::new(MyGrpcServiceFactory),
                    std::time::Duration::from_secs(#timeout_sec),
                ),
            }
        }

        pub fn get_service_name() -> &'static str {
            #grpc_service_name
        }

        #(#grpc_methods)*  
      }

      #(#interfaces)*  
    }
    .into())
}



