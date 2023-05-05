use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::attribute_params::AttributeParams;

use super::proto_file_reader::ProtoFile;

pub fn generate(
    attr: TokenStream,
    input: TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let attr_input: proc_macro2::TokenStream = attr.into();

    let attributes = AttributeParams::from_token_string(attr_input)?;

    let timeout_sec = attributes.get_named_param("timeout_sec")?;
    let timeout_sec: String = timeout_sec.get_value(None)?;
    let timeout_sec = proc_macro2::TokenStream::from_str(timeout_sec.as_str()).unwrap();

    let proto_file = attributes.get_named_param("proto_file")?;
    let proto_file: String = proto_file.get_value(None)?;

    let proto_file = super::proto_file_reader::read_proto_file(proto_file);

    let grpc_service_name = &proto_file.service_name;
    let grpc_service_name_token = proto_file.get_service_name_as_token();

    let interfaces = generate_interfaces_implementations(&proto_file);

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
        pub async fn new(get_grpc_address: std::sync::Arc<dyn my_grpc_extensions::GrpcClientSettings + Send + Sync + 'static>,) -> Self {
            Self {
                channel: GrpcChannel::new(
                    get_grpc_address,
                    std::sync::Arc::new(MyGrpcServiceFactory),
                    Duration::from_secs(#timeout_sec),
                ),
            }
        }
      }

      #(#interfaces)*  
    }
    .into())
}

fn generate_interfaces_implementations(proto_file: &ProtoFile) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();


    for rpc in &proto_file.rpc{


        if let Some(input_param_type) = &rpc.get_input_param(){
            if let Some(output_param_type) = &rpc.get_output_param(){
                let input_param_type_token = input_param_type.get_input_param_type_token();

                let output_param_type_token = output_param_type.get_output_param_type_token();

                let input_param_name_token = input_param_type.get_name_as_token();
                let output_param_name_token = output_param_type.get_name_as_token();

                let interface_name = get_interface_name(&input_param_type, &output_param_type);
                

                let quote = quote::quote!{
                    #[async_trait::async_trait]
                    impl
                        #interface_name<
                            TGrpcService,
                            #input_param_name_token,
                            #output_param_name_token,
                        > for KeyValueGrpcClient
                    {
                        async fn execute(
                            &self,
                            mut service: TGrpcService,
                            input_data: #input_param_type_token,
                        ) -> Result<#output_param_type_token, tonic::Status> {
                            let result = service.get(stream::iter(input_data)).await?;
                            Ok(result.into_inner())
                        }
                    }
                };
        
                result.push(quote);

            }
            else{
                let input_param_type_token = input_param_type.get_input_param_type_token();

                let output_param_type_token = quote::quote!{()};

                let input_param_name_token = input_param_type.get_name_as_token();


                let interface_name = get_interface_name_with_input_param_only(&input_param_type);
                

                let quote = quote::quote!{
                    #[async_trait::async_trait]
                    impl
                        #interface_name<
                            TGrpcService,
                            #input_param_name_token,
                            #output_param_type_token,
                        > for KeyValueGrpcClient
                    {
                        async fn execute(
                            &self,
                            mut service: TGrpcService,
                            input_data: #input_param_type_token,
                        ) -> Result<#output_param_type_token, tonic::Status> {
                            let result = service.get(stream::iter(input_data)).await?;
                            Ok(result.into_inner())
                        }
                    }
                };
        
                result.push(quote); 
            }

        }else{
            if let Some(output_param_type) = &rpc.get_output_param(){

                let input_param_type_token = quote::quote!{()};

                let output_param_type_token = output_param_type.get_output_param_type_token();

                let output_param_name_token = output_param_type.get_name_as_token();

                let interface_name = get_interface_name_with_output_param_only(&output_param_type);
                

                let quote = quote::quote!{
                    #[async_trait::async_trait]
                    impl
                        #interface_name<
                            TGrpcService,
                            #input_param_type_token,
                            #output_param_name_token,
                        > for KeyValueGrpcClient
                    {
                        async fn execute(
                            &self,
                            mut service: TGrpcService,
                            input_data: #input_param_type_token,
                        ) -> Result<#output_param_type_token, tonic::Status> {
                            let result = service.get(stream::iter(input_data)).await?;
                            Ok(result.into_inner())
                        }
                    }
                };
        
                result.push(quote);
            }
            else{

                panic!("RPC {} Not supported if input_param and output_param are both empty", rpc.name)
            }
        }
    }


    result
}



fn get_interface_name(input_param: &super::proto_file_reader::ParamType<'_>, output_param: &super::proto_file_reader::ParamType<'_>)->proc_macro2::TokenStream{
    if input_param.is_stream(){
        if output_param.is_stream(){

            quote::quote!(RequestWithInputAsStreamWithResponseAsStreamGrpcExecutor)

        }else{
            quote::quote!(RequestWithInputAsStreamGrpcExecutor)
        }
    }else{
        if output_param.is_stream(){
            quote::quote!(RequestWithResponseAsStreamGrpcExecutor)
        }else{
            quote::quote!(RequestResponseGrpcExecutor)
        }
    }
}

fn get_interface_name_with_input_param_only(input_param: &super::proto_file_reader::ParamType<'_>)->proc_macro2::TokenStream{
    if input_param.is_stream(){
        quote::quote!(RequestWithInputAsStreamGrpcExecutor)
    }else{
        quote::quote!(RequestResponseGrpcExecutor)
    }
}



fn get_interface_name_with_output_param_only(output_param: &super::proto_file_reader::ParamType<'_>)->proc_macro2::TokenStream{
        if output_param.is_stream(){
            quote::quote!(RequestWithResponseAsStreamGrpcExecutor)
        }else{
            quote::quote!(RequestResponseGrpcExecutor)
        }
}