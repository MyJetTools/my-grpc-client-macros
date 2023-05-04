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
    service_name: String,
    rpc: Vec<ProtoRpc>,
}

pub fn read_proto_file(file_name: String) -> ProtoFile {
    let file = std::fs::File::open(file_name.as_str());

    if let Err(err) = file {
        panic!("Can not open file: {}. Error: {:?}", file_name, err);
    }

    ProtoFile {
        service_name: String::from(""),
        rpc: Vec::new(),
    }
}
