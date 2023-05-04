use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::attribute_params::AttributeParams;

pub fn generate(
    attr: TokenStream,
    input: TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let attr_input: proc_macro2::TokenStream = attr.into();

    let attributes = AttributeParams::from_token_string(attr_input)?;

    let grpc_service_name = attributes.get_named_param("grpc_service_name")?;
    let grpc_service_name: String = grpc_service_name.get_value(None)?;

    let grpc_service_name_token =
        proc_macro2::TokenStream::from_str(grpc_service_name.as_str()).unwrap();

    let timeout_sec = attributes.get_named_param("timeout_sec")?;
    let timeout_sec: usize = timeout_sec.get_value(None)?;

    Ok(quote::quote! {

        pub const SERVICE_NAME: &str = #grpc_service_name;
        type TGrpcService = #grpc_service_name_token<InterceptedService<tonic::transport::Channel, my_grpc_extensions::GrpcClientInterceptor>>;

        struct MyGrpcServiceFactory;

        #[async_trait::async_trait]
        impl my_grpc_extensions::GrpcServiceFactory<TGrpcService> for MyGrpcServiceFactory {
        fn create_service(&self, channel: Channel, ctx: &MyTelemetryContext) -> TGrpcService {
            #grpc_service_name_token::with_interceptor(
              channel,
              my_grpc_extensions::GrpcClientInterceptor::new(ctx.clone()),
            )
        }

        fn get_service_name(&self) -> &'static str {
            SERVICE_NAME
        }

        async fn ping(&self, mut service: TGrpcService) {
           service.ping(()).await.unwrap();
        }
      }

      #ast

      impl #struct_name{
        pub fn new(get_grpc_address: Arc<dyn GrpcClientSettings + Send + Sync + 'static>) -> Self {
            Self {
                grpc_channel: GrpcChannel::new(
                    get_grpc_address,
                    GRPC_SERVICE_NAME,
                    Duration::from_secs(#timeout_sec),
                ),
            }
        }
      }
    }
    .into())
}
