use quote::ToTokens;

use crate::path::PathAugment;

pub trait AttributeAugment {
    /// Extracts the value in a path-style attribute such as in `#[inner_type = "usize"]`
    fn extract_val(&self) -> String;

    fn str_equals(&self, str: &str) -> bool;
}

impl AttributeAugment for syn::Attribute {
    fn extract_val(&self) -> String {
        match &self.meta {
            syn::Meta::NameValue(name_value) => name_value
                .value
                .to_token_stream()
                .to_string()
                .replace([' ', '"'], ""),
            _ => panic!("Options must be assigned to an expression"),
        }
    }

    fn str_equals(&self, str: &str) -> bool {
        self.path().str_equals(str)
    }
}
