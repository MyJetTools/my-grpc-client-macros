use std::str::FromStr;

#[derive(Debug)]
pub enum ParamType<'s> {
    Single(&'s str),
    Stream(&'s str),
}

impl<'s> ParamType<'s> {
    pub fn parse(src: &'s str) -> Option<Self> {
        let mut is_vec = false;

        let mut name = None;

        for param in src.split_ascii_whitespace() {
            if param == "stream" {
                is_vec = true;
                continue;
            } else {
                name = Some(param);
                break;
            }
        }

        let name = name?;

        if is_vec {
            Self::Stream(name).into()
        } else {
            Self::Single(name).into()
        }
    }

    pub fn is_stream(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Stream(_) => true,
        }
    }

    pub fn get_name_as_token(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Single(name) => proc_macro2::TokenStream::from_str(name).unwrap(),
            Self::Stream(name) => proc_macro2::TokenStream::from_str(name).unwrap(),
        }
    }

    pub fn get_input_param_type_token(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Single(name) => proc_macro2::TokenStream::from_str(name).unwrap(),
            Self::Stream(name) => {
                let param = proc_macro2::TokenStream::from_str(name).unwrap();
                quote::quote!(Vec<#param>)
            }
        }
    }

    pub fn get_input_param_invoke_token(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Single(_) => proc_macro2::TokenStream::from_str("input_data").unwrap(),
            Self::Stream(_) => {
                quote::quote!(futures::stream::iter(input_data))
            }
        }
    }

    pub fn get_output_param_type_token(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Single(name) => proc_macro2::TokenStream::from_str(name).unwrap(),
            Self::Stream(name) => {
                let param = proc_macro2::TokenStream::from_str(name).unwrap();
                quote::quote!(tonic::Streaming<#param>)
            }
        }
    }
}
