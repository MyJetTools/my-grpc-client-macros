use std::{
    io::{BufRead, BufReader},
    str::FromStr,
};

pub enum ParamType {
    Single(String),
    Vector(String),
}

pub struct ProtoRpc {
    name: String,
    input_param: Option<ParamType>,
    output_param: Option<ParamType>,
}

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

    for line in reader.lines() {
        let line = line.unwrap();

        for token in line.split_whitespace() {
            match current_token {
                CurrentToken::None => {
                    if token == "service" {
                        current_token = CurrentToken::Service;
                    }
                }
                CurrentToken::Service => {
                    service_name = Some(token.to_string());
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
        rpc: Vec::new(),
    }
}
