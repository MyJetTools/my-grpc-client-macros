use std::collections::HashMap;

use proc_macro2::TokenStream;
use types_reader::ParamsListAsTokens;

pub struct FnOverride {
    pub retries: usize,
    pub token_stream: TokenStream,
}

impl FnOverride {
    pub fn new(attributes: &ParamsListAsTokens) -> Result<HashMap<String, Self>, syn::Error> {
        let overrides = attributes.try_get_named_param("overrides");

        if overrides.is_none() {
            return Ok(HashMap::new());
        }

        let overrides = overrides.unwrap().unwrap_as_object_list()?;

        let mut result = HashMap::new();

        for item in overrides.iter() {
            let name = item
                .get_named_param("fn_name")?
                .get_str_value()?
                .to_string();
            result.insert(
                name,
                FnOverride {
                    retries: item.get_named_param("retries")?.get_number_value()? as usize,
                    token_stream: item.get_token_stream(),
                },
            );
        }

        Ok(result)
    }
}
