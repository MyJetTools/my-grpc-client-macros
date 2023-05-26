use std::str::FromStr;

use proc_macro::TokenStream;

use types_reader::{ComplexAttrParams, ParamContent};

use super::proto_file_reader::ProtoServiceDescription;



pub fn generate(
    attr: TokenStream,
    input: TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {

    println!("{:#?}", attr);

    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let attr_input: proc_macro2::TokenStream = attr.into();

    let mut attr_as_str = attr_input.to_string();
    
    let attributes = ComplexAttrParams::new(&mut attr_as_str);

    let timeout_sec = attributes.get_named_param("timeout_sec");
    let timeout_sec = into_err(timeout_sec, &attr_input)?;
    let timeout_sec = timeout_sec.as_str();
    let timeout_sec = proc_macro2::TokenStream::from_str(timeout_sec).unwrap();

    let proto_file = attributes.get_named_param("proto_file");
    let proto_file = into_err(proto_file, &attr_input)?;
    let proto_file = proto_file.as_str();
    let proto_file = ProtoServiceDescription::read_proto_file(proto_file);

    let grpc_service_name = &proto_file.service_name;
    let grpc_service_name_token = proto_file.get_service_name_as_token();

    let interfaces = super::generate_interfaces_implementations(struct_name, &proto_file);

    let retries = attributes.get_named_param("retries");
    let retries = into_err(retries, &attr_input)?;
    let grpc_methods = super::generate_grpc_methods(&proto_file, retries.get_value(None).unwrap());

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
        pub fn new(get_grpc_address: std::sync::Arc<dyn my_grpc_extensions::GrpcClientSettings + Send + Sync + 'static>,) -> Self {
            Self {
                channel: GrpcChannel::new(
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


fn into_err<'s>(src: Result<ParamContent<'s>, String>, token:&proc_macro2::TokenStream)->Result<ParamContent<'s>, syn::Error>{
    match src {
        Ok(value) => Ok(value),
        Err(err) => Err(syn::Error::new_spanned(token.clone(), err)),
    }
}