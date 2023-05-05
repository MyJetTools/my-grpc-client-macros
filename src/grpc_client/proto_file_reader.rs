use std::{
    io::{BufRead, BufReader},
    str::FromStr,
};

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
#[derive(Debug)]
pub struct ProtoRpc {
    pub name: String,
    input_param: String,
    output_param: String,
}

impl ProtoRpc {
    pub fn get_fn_name_as_token(&self) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::from_str(&into_snake_case(self.name.as_str())).unwrap()
    }

    pub fn get_input_param(&self) -> Option<ParamType> {
        ParamType::parse(&self.input_param)
    }

    pub fn get_output_param(&self) -> Option<ParamType> {
        ParamType::parse(&self.input_param)
    }
}

#[derive(Debug)]
pub struct ProtoFile {
    pub service_name: String,
    pub rpc: Vec<ProtoRpc>,
}

impl ProtoFile {
    pub fn get_service_name_as_token(&self) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::from_str(&self.service_name).unwrap()
    }
}

pub enum CurrentToken {
    None,
    Service,
    Rpc,
    RpcExpectingInputParameter,
    RpcExpectingOutputParameter,
}

pub fn read_proto_file(file_name: String) -> ProtoFile {
    let file = std::fs::File::open(file_name.as_str());

    if let Err(err) = file {
        panic!("Can not open file: {}. Error: {:?}", file_name, err);
    }

    let file = file.unwrap();

    let reader = BufReader::new(file);

    let mut service_name = None;

    let mut current_token = CurrentToken::None;

    let mut rpc_name = None;

    let mut input_param_name = String::new();

    let mut out_param_name = String::new();

    let mut rpc = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();

        for token in line.split_whitespace() {
            match current_token {
                CurrentToken::None => {
                    if token == "service" {
                        current_token = CurrentToken::Service;
                    }

                    if token == "rpc" {
                        current_token = CurrentToken::Rpc;
                    }
                }
                CurrentToken::Rpc => {
                    let (rpc_name_value, input_param_value) =
                        extract_input_param_from_rpc_name(token);
                    rpc_name = Some(rpc_name_value.to_string());

                    input_param_name.clear();
                    out_param_name.clear();

                    if let Some(input_param) = input_param_value {
                        input_param_name.push_str(input_param);
                    }

                    current_token = CurrentToken::RpcExpectingInputParameter;
                }
                CurrentToken::RpcExpectingInputParameter => {
                    if token == "returns" {
                        current_token = CurrentToken::RpcExpectingOutputParameter;
                    } else {
                        input_param_name.push(' ');
                        input_param_name.push_str(token);
                    }
                }

                CurrentToken::RpcExpectingOutputParameter => {
                    let the_end = token.ends_with(";");
                    if the_end {
                        out_param_name.push_str(&token[..token.len() - 1]);
                    } else {
                        out_param_name.push_str(&token);
                    }

                    if the_end {
                        let name = rpc_name.as_ref().unwrap();

                        if name != "Ping" {
                            rpc.push(ProtoRpc {
                                name: name.to_string(),
                                input_param: extract_param(input_param_name.as_str()),
                                output_param: extract_param(out_param_name.as_str()),
                            });
                        }
                        current_token = CurrentToken::None;
                    }
                }
                CurrentToken::Service => {
                    service_name = Some(format!("{}Client", token));
                    current_token = CurrentToken::None;
                }
            }
        }
    }

    if service_name.is_none() {
        panic!("Can not find service name in proto file: {}", file_name);
    }

    ProtoFile {
        service_name: service_name.unwrap().to_string(),
        rpc,
    }
}

fn extract_input_param_from_rpc_name<'s>(token: &'s str) -> (&'s str, Option<&'s str>) {
    let index = token.find("(");

    if index.is_none() {
        return (token, None);
    }

    let index = index.unwrap();

    (&token[..index], (&token[index..]).into())
}

fn extract_param(token: &str) -> String {
    let end = token.find(")").unwrap();
    token[1..end].to_string()
}

fn into_snake_case(src: &str) -> String {
    let mut result = String::new();

    for (index, ch) in src.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                result.push('_');
            }

            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_into_camel_case() {
        assert_eq!(super::into_snake_case("HelloWorld"), "hello_world");
    }

    #[test]
    fn extract_input_param_from_rpc_name_with_param_name() {
        let result = super::extract_input_param_from_rpc_name("test(MyParam)");

        assert_eq!(result.0, "test");
        assert_eq!(result.1.unwrap(), "MyParam");
    }

    #[test]
    fn extract_input_param_from_rpc_name_with_no_param_name() {
        let result = super::extract_input_param_from_rpc_name("test");

        assert_eq!(result.0, "test");
        assert!(result.1.is_none());
    }
}
