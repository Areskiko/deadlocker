use std::fmt::Display;

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Field;

use crate::field::FieldAugment;

#[derive(PartialEq, Eq, Debug)]
pub struct State {
    pub struct_ident: Ident,
    pub active: Vec<Field>,
    pub complements: Vec<Field>,
    pub all_ordered: Vec<Field>,
}

impl State {
    /// Computes the state resulting from adding a [syn::Field] to the existing state
    pub fn add_state(&self, field: &Field) -> State {
        let reduced_complement = self
            .complements
            .iter()
            .filter(|existing_field| *existing_field != field)
            .map(Field::to_owned)
            .collect();

        let mut expanded_active = self.active.clone();
        expanded_active.push(field.to_owned());

        let active_ordered = self
            .all_ordered
            .iter()
            .filter(|possible| expanded_active.contains(possible))
            .map(Field::to_owned)
            .collect();

        State {
            struct_ident: self.struct_ident.clone(),
            active: active_ordered,
            complements: reduced_complement,
            all_ordered: self.all_ordered.clone(),
        }
    }

    /// Returns a [proc_macro2::TokenStream] containing the assignments of the lock results for each field in the state
    pub fn locked_fields(&self) -> Vec<TokenStream> {
        self.active
            .iter()
            .map(|f| {
                let ident = &f.ident;
                let res = if f.is_result() {
                    quote! {?}
                } else {
                    quote! {}
                };
                let lock_method = &f.lock_method();
                quote! {
                    let #ident = Box::new(self.#ident.#lock_method #res);
                }
                .into_token_stream()
            })
            .collect()
    }

    /// Returns whether any of the fields are asynchronous
    ///
    /// See [FieldHelpers::is_async]
    pub fn is_async(&self) -> bool {
        self.active.iter().any(Field::is_async)
    }

    /// Return a formatted identifier for the state
    pub fn ident(&self) -> Ident {
        format_ident!("{}Locker{}", self.struct_ident, self.to_string())
    }

    /// Returns a [Vec] of all possible sub-[State]s, including this [State]
    pub fn into_substates(self) -> Vec<State> {
        let mut explored = Vec::new();
        for field in &self.complements {
            let new_state = self.add_state(field);

            for possible_state in new_state.into_substates() {
                if !explored.contains(&possible_state) {
                    explored.push(possible_state);
                }
            }
        }

        if !explored.contains(&self) {
            explored.push(self);
        }

        explored
    }
}

impl From<State> for String {
    fn from(val: State) -> Self {
        val.to_string()
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self
            .active
            .iter()
            .filter_map(|state| state.ident.as_ref().map(|i| i.to_string()))
            .map(|state| {
                let mut c = state.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .map(|state| {
                let mut modified = state;
                while let Some(pos) = modified.find('_') {
                    modified = modified
                        .chars()
                        .enumerate()
                        .map(|(i, c)| {
                            if i == pos + 1 {
                                c.to_uppercase().collect()
                            } else {
                                c.to_string()
                            }
                        })
                        .collect();
                    modified = modified.replacen('_', "Underscore", 1);
                }
                modified
            })
            .join("");
        if name.is_empty() {
            f.write_str("Empty")
        } else {
            f.write_str(name.as_str())
        }
    }
}
