use std::io::{BufRead, BufReader};

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

    let file = file.unwrap();

    let reader = BufReader::new(file);

    // Step 2: loop over lines and print them.
    for line in reader.lines() {
        println!("LINE: {}", line.unwrap());
    }

    ProtoFile {
        service_name: String::from(""),
        rpc: Vec::new(),
    }
}
