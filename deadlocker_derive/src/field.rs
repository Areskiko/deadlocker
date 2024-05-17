use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use regex::Regex;
use syn::Field;

use crate::{
    attribute::AttributeAugment, ASYNC, DEFAULT_OUTER_TYPE, EXCLUDE, INCLUDE, INNER_TYPE,
    LOCK_METHOD, OUTER_TYPE, RESULT,
};

/// Helper functions for the [syn::Field] type
pub trait FieldAugment {
    /// Returns whether the fields has a `is_async` attribute
    fn is_async(&self) -> bool;
    ///
    /// Returns whether the fields has a `result` attribute
    fn is_result(&self) -> bool;

    /// Returns whether the fields has a `include` attribute
    fn is_included(&self) -> bool;

    /// Returns whether the fields has a `exclude` attribute
    fn is_excluded(&self) -> bool;

    /// Returns the method for locking the outer lock
    fn lock_method(&self) -> TokenStream;

    /// Returns the inner type for the field
    ///
    /// This is what the user is attempting to interact with through the lock
    fn return_type(&self) -> syn::TypePath;
}

impl FieldAugment for Field {
    fn is_async(&self) -> bool {
        for attr in &self.attrs {
            if attr.str_equals(ASYNC) {
                return true;
            }
        }
        false
    }

    fn is_result(&self) -> bool {
        for attr in &self.attrs {
            if attr.str_equals(RESULT) {
                return true;
            }
        }
        false
    }

    fn is_included(&self) -> bool {
        for attr in &self.attrs {
            if attr.str_equals(INCLUDE) {
                return true;
            }
        }
        false
    }

    fn is_excluded(&self) -> bool {
        for attr in &self.attrs {
            if attr.str_equals(EXCLUDE) {
                return true;
            }
        }
        false
    }

    fn lock_method(&self) -> TokenStream {
        let mut lock_method = None;
        for attr in &self.attrs {
            if attr.str_equals(LOCK_METHOD) {
                lock_method = Some(
                    syn::parse_str::<TokenStream>(attr.extract_val().as_str())
                        .expect("Failed to parse lock method"),
                );
            }
        }

        match lock_method {
            Some(l) => l,
            None => {
                if self.is_async() {
                    quote! {lock().await}
                } else {
                    quote! {lock()}
                }
            }
        }
    }

    fn return_type(&self) -> syn::TypePath {
        let mut outer_type: String = DEFAULT_OUTER_TYPE.to_string();
        let path = self.ty.to_token_stream().to_string().replace(' ', "");

        for attr in &self.attrs {
            if attr.str_equals(INNER_TYPE) {
                return syn::parse_str::<syn::TypePath>(attr.extract_val().as_str())
                    .expect("Expected type");
            } else if attr.str_equals(OUTER_TYPE) {
                outer_type = attr.extract_val().to_string().replace(' ', "");
            }
        }

        let re = Regex::new(outer_type.as_str()).unwrap();

        if let Some((_, [inner])) = re.captures_iter(path.as_str()).map(|c| c.extract()).next() {
            return syn::parse_str::<syn::TypePath>(inner).expect("Expected type");
        }

        panic!("Could not find inner type by removing outer type")
    }
}
