use std::{
    io::{BufRead, BufReader},
    str::FromStr,
};

#[derive(Debug)]
pub enum ParamType {
    Single(String),
    Vector(String),
}
#[derive(Debug)]
pub struct ProtoRpc {
    name: String,
    input_param: String,
    output_param: String,
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
    RpcExpectingReturnsToken,
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

    let mut rpc_name: Option<String> = None;

    let mut input_param_name = None;

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
                    rpc_name = Some(rpc_name_value);

                    input_param_name = input_param_value;

                    if input_param_name.is_some() {
                        current_token = CurrentToken::RpcExpectingReturnsToken;
                    } else {
                        current_token = CurrentToken::RpcExpectingInputParameter;
                    }
                }
                CurrentToken::RpcExpectingReturnsToken => {
                    if token == "returns" {
                        current_token = CurrentToken::RpcExpectingOutputParameter;
                    } else {
                        panic!(
                            "Expecting 'returns' token after rpc name: {}",
                            rpc_name.unwrap()
                        );
                    }
                }
                CurrentToken::RpcExpectingInputParameter => {
                    input_param_name = Some(extract_param(token));
                    current_token = CurrentToken::RpcExpectingReturnsToken;
                }
                CurrentToken::RpcExpectingOutputParameter => {
                    let out_param_name = extract_param(token);
                    rpc.push(ProtoRpc {
                        name: rpc_name.as_ref().unwrap().to_string(),
                        input_param: input_param_name.as_ref().unwrap().to_string(),
                        output_param: out_param_name,
                    });
                    current_token = CurrentToken::None;
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

fn extract_input_param_from_rpc_name(token: &str) -> (String, Option<String>) {
    let index = token.find("(");

    if index.is_none() {
        return (token.to_string(), None);
    }

    let index = index.unwrap();
    match token.find(")") {
        Some(end) => (
            token[..index].to_string(),
            token[index + 1..end].to_string().into(),
        ),
        None => (
            token[..index].to_string(),
            token[index + 1..].to_string().into(),
        ),
    }
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
