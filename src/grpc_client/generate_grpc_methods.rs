use std::str::FromStr;

use super::{proto_file_reader::ProtoFile, ParamType};

pub fn generate_grpc_methods(proto_file: &ProtoFile) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();

    for rpc in &proto_file.rpc {
        let fn_name = rpc.get_fn_name_as_token();

        let input_param = rpc.get_input_param();

        let output_param = rpc.get_output_param();

        let input_data_type = get_func_data_type(input_param.as_ref());

        let output_data_type = get_func_data_type(output_param.as_ref());

        let request_fn_name = get_request_fn_name(input_param.as_ref());
        let response_fn_name = get_response_fn_name(output_param.as_ref());

        let item = quote::quote! {
            pub async fn #fn_name(
                &self,
                input_data: #input_data_type,
                ctx: &MyTelemetryContext,
            ) -> Result<#output_data_type, my_grpc_extensions::GrpcReadError> {
                let channel = self.channel.get_channel(ctx).await.unwrap();

                channel
                    .#request_fn_name(input_data)
                    .with_retries(3)
                    .#response_fn_name(self)

            }
        };

        result.push(item);
    }

    result
}

fn get_request_fn_name(input_param: Option<&super::ParamType<'_>>) -> proc_macro2::TokenStream {
    match input_param {
        Some(input_param) => {
            if input_param.is_stream() {
                quote::quote! {start_request_with_input_prams_as_stream}
            } else {
                quote::quote! {start_request}
            }
        }
        None => {
            quote::quote! {start_request}
        }
    }
}

fn get_response_fn_name(input_param: Option<&super::ParamType<'_>>) -> proc_macro2::TokenStream {
    match input_param {
        Some(input_param) => {
            if input_param.is_stream() {
                quote::quote! {get_streamed_response.await?
                .as_vec()
                .await?}
            } else {
                quote::quote! {get_response.await?}
            }
        }
        None => {
            quote::quote! {get_response.await?}
        }
    }
}

fn get_func_data_type(data_type: Option<&super::ParamType<'_>>) -> proc_macro2::TokenStream {
    match data_type {
        Some(input_param) => match input_param {
            ParamType::Single(name) => proc_macro2::TokenStream::from_str(name).unwrap(),
            ParamType::Stream(name) => {
                let param = proc_macro2::TokenStream::from_str(name).unwrap();
                quote::quote!(Vec<#param>)
            }
        },
        None => {
            quote::quote! {()}
        }
    }
}
