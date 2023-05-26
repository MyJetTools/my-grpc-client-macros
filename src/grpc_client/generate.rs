

use proc_macro::TokenStream;

use types_reader::{ ParamsListAsTokens};

use crate::grpc_client::fn_override::FnOverride;

use super::proto_file_reader::ProtoServiceDescription;

pub fn generate(
    attr: TokenStream,
    input: TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {

    println!("{:#?}", attr);

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

        pub const SERVICE_NAME: &str = #grpc_service_name;
        type TGrpcService = #grpc_service_name_token<tonic::codegen::InterceptedService<tonic::transport::Channel, my_grpc_extensions::GrpcClientInterceptor>>;

        struct MyGrpcServiceFactory;

        #[async_trait::async_trait]
        impl my_grpc_extensions::GrpcServiceFactory<TGrpcService> for MyGrpcServiceFactory {
        fn create_service(&self, channel: Channel, ctx: &my_telemetry::MyTelemetryContext) -> TGrpcService {
            #grpc_service_name_token::with_interceptor(
              channel,
              my_grpc_extensions::GrpcClientInterceptor::new(ctx.clone()),
            )
        }

        fn get_service_name() -> &'static str {
            SERVICE_NAME
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
                    Duration::from_secs(#timeout_sec),
                ),
            }
        }

        #(#grpc_methods)*  
      }

      #(#interfaces)*  
    }
    .into())
}



