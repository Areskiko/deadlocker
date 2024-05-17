use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Field;

use crate::{field::FieldAugment, state::State};

/// Generate the token stream for the builder struct definition
pub fn generate_builder_struct(
    name: &Ident,
    fields: syn::punctuated::Iter<'_, Field>,
) -> TokenStream {
    let mut field_declarations = Vec::new();

    for field in fields {
        let field_ident = &field.ident;
        let ty = &field.ty;

        field_declarations.push(quote! {
            #field_ident: &'a mut #ty
        });
    }

    quote! {
        pub struct #name<'a, State> {
            _phantom: std::marker::PhantomData<State>,
            #(#field_declarations),*
        }
    }
}

/// Generate the token stream for the various impl blocks in the builder pattern chain
pub fn generate_impl_for_all_states(
    locker_struct_name: &Ident,
    states: &[State],
    all_fields: &syn::Fields,
) -> TokenStream {
    let mut impl_states = TokenStream::new();
    for state in states {
        let state_ident = state.ident();

        let mut functions = Vec::new();
        for complement in &state.complements {
            let complement_state = state.add_state(complement);
            let complement_fn = format_ident!(
                "{}",
                complement.ident.as_ref().expect("Fields must be named")
            );
            let complement_ident = complement_state.ident();
            let fields_iter = all_fields.iter().map(|f| &f.ident);
            let fields_iter2 = all_fields.iter().map(|f| &f.ident);

            functions.push(quote! {
                pub fn #complement_fn(self) -> #locker_struct_name<'a, #complement_ident<'a>> {
                    #locker_struct_name{
                        _phantom: std::marker::PhantomData,
                        #(#fields_iter: self.#fields_iter2),*
                    }
                }
            });
        }

        let idents: Vec<&Ident> = state
            .active
            .iter()
            .map(|a| a.ident.as_ref().expect("All fields must be named"))
            .collect();

        let locked_fields = state.locked_fields();

        let parameters = if !state.active.is_empty() {
            quote! {<'a>}
        } else {
            quote! {}
        };

        let asyncrocity = if state.is_async() {
            quote! {async}
        } else {
            quote! {}
        };

        let (result_left, result_right, return_statement) =
            if state.active.iter().any(|f| f.is_result()) {
                (
                    quote! {
                        Result<
                    },
                    quote! {
                        , Box<dyn std::error::Error + 'a>>
                    },
                    quote! {
                        Ok(
                        #state_ident{#(#idents),*}
                        )
                    },
                )
            } else {
                (quote! {}, quote! {}, quote! {#state_ident{#(#idents),*}})
            };
        let lock_method = if !state.active.is_empty() {
            quote! {
                pub #asyncrocity fn lock(self) -> #result_left #state_ident #parameters #result_right {
                    #(#locked_fields)*
                    #return_statement
                }
            }
        } else {
            quote! {}
        };

        quote! {
            impl<'a> #locker_struct_name<'a, #state_ident #parameters> {
                #(#functions)*

                #lock_method
            }
        }
        .to_tokens(&mut impl_states)
    }

    impl_states
}

/// Generate the token stream for the implementation of the `Locker` trait
pub fn generate_trait_implementation(
    struct_identifier: &Ident,
    locker_struct_name: &Ident,
    empty_identifier: &Ident,
    all_fields: &syn::Fields,
) -> TokenStream {
    let struct_fields = all_fields
        .iter()
        .filter_map(|f| Some(format_ident!("{}", f.ident.as_ref()?)));

    quote! {
        impl<'a> Locker<'a> for #struct_identifier {
            type LockBuilder=#locker_struct_name<'a, #empty_identifier>;
            fn locker(&'a mut self) -> Self::LockBuilder {
                Self::LockBuilder{_phantom: std::marker::PhantomData,#(#struct_fields: &mut self.#struct_fields),*}
            }
        }
    }
}

/// Generate the token stream for the output struct declarations
pub fn generate_state_struct_declarations(states: &[State]) -> TokenStream {
    let mut definitions = TokenStream::new();

    for state in states {
        let state_name = state.ident();
        let state_fields_ident = state.active.iter().map(|f| {
            f.ident
                .as_ref()
                .expect("All fields must be named")
                .to_owned()
        });

        let state_fields_return_type = state.active.iter().map(Field::return_type);
        let deref_trait = if true {
            quote! {std::ops::DerefMut}
        } else {
            quote! {std::ops::Deref}
        };

        let parameters = if !state.active.is_empty() {
            quote! {<'a>}
        } else {
            quote! {}
        };

        quote! {
            pub struct #state_name #parameters {
                #(pub #state_fields_ident: Box<dyn #deref_trait<Target = #state_fields_return_type> + 'a>),*
            }
        }.to_tokens(&mut definitions)
    }

    definitions
}
