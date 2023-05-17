pub struct ProtoTokensReader<'s> {
    content: &'s [u8],
    pos: usize,
}

impl<'s> ProtoTokensReader<'s> {
    pub fn new(content: &'s str) -> Self {
        Self {
            content: content.as_bytes(),
            pos: 0,
        }
    }

    pub fn get_next(&mut self) -> Option<&'s str> {
        let mut start_pos = None;
        while self.pos < self.content.len() {
            let b = self.content[self.pos];
            if b <= 32 {
                if let Some(start_pos) = start_pos {
                    return Some(std::str::from_utf8(&self.content[start_pos..self.pos]).unwrap());
                }
                self.pos += 1;
                continue;
            }

            if b == b'(' || b == b')' || b == b';' {
                if let Some(start_pos) = start_pos {
                    return Some(std::str::from_utf8(&self.content[start_pos..self.pos]).unwrap());
                }

                let result = std::str::from_utf8(&self.content[self.pos..self.pos + 1]).unwrap();

                self.pos += 1;

                return Some(result);
            }

            if start_pos.is_none() {
                start_pos = Some(self.pos);
            }

            self.pos += 1;
        }

        if let Some(start_pos) = start_pos {
            return Some(std::str::from_utf8(&self.content[start_pos..self.pos]).unwrap());
        }

        None
    }
}

impl<'s> Iterator for ProtoTokensReader<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next()
    }
}

#[cfg(test)]
mod tests {
    use super::ProtoTokensReader;

    #[test]
    fn test_basic_parse() {
        let src = "rpc Get(stream keyvalue.GetKeyValueGrpcRequestModel) returns (stream keyvalue.GetKeyValueGrpcResponseModel);";

        let result = ProtoTokensReader::new(src).collect::<Vec<_>>();

        assert_eq!(result.len(), 12);

        assert_eq!(result[0], "rpc");
        assert_eq!(result[1], "Get");
        assert_eq!(result[2], "(");
        assert_eq!(result[3], "stream");
        assert_eq!(result[4], "keyvalue.GetKeyValueGrpcRequestModel");
        assert_eq!(result[5], ")");
        assert_eq!(result[6], "returns");
        assert_eq!(result[7], "(");
        assert_eq!(result[8], "stream");
        assert_eq!(result[9], "keyvalue.GetKeyValueGrpcResponseModel");
        assert_eq!(result[10], ")");
        assert_eq!(result[11], ";");
    }
}
