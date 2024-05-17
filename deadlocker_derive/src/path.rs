use quote::ToTokens;

pub trait PathAugment {
    /// Check if the [syn::Path] is equal to some [&str]
    fn str_equals(&self, str: &str) -> bool;
}

impl PathAugment for syn::Path {
    fn str_equals(&self, str: &str) -> bool {
        self.to_token_stream().to_string() == *str
    }
}
