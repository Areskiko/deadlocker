use field::FieldAugment;
use generators::{
    generate_builder_struct, generate_impl_for_all_states, generate_state_struct_declarations,
    generate_trait_implementation,
};

use quote::{format_ident, quote};

use state::State;
use syn::Field;

mod attribute;
mod field;
mod generators;
mod path;
mod state;

const DEFAULT_OUTER_TYPE: &str = "Arc<Mutex<(.*)>>";
const OUTER_TYPE: &str = "outer_type";
const INNER_TYPE: &str = "inner_type";
const ASYNC: &str = "is_async";
const RESULT: &str = "result";
const LOCK_METHOD: &str = "lock_method";
const INCLUDE: &str = "include";
const EXCLUDE: &str = "exclude";

#[proc_macro_derive(
    Locker,
    attributes(
        outer_type,
        inner_type,
        is_async,
        lock_method,
        result,
        include,
        exclude
    )
)]
pub fn locker_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_locker_macro(&ast)
}

/// Main driver function extracting fields and generating states to use in invocation of the generator functions
fn impl_locker_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let ident = &ast.ident;

    let fields = if let syn::Data::Struct(parsed_struct) = &ast.data {
        &parsed_struct.fields
    } else {
        panic!("Only applicable to structs")
    };

    let all_ordered = if fields.iter().any(Field::is_included) {
        fields
            .iter()
            .filter(|f| f.is_included())
            .map(Field::to_owned)
            .collect::<Vec<Field>>()
    } else {
        fields
            .iter()
            .filter(|f| !f.is_excluded())
            .map(Field::to_owned)
            .collect::<Vec<Field>>()
    };

    let empty = State {
        struct_ident: ident.clone(),
        active: Vec::new(),
        complements: all_ordered.clone(),
        all_ordered: all_ordered.clone(),
    };

    let empty_name = empty.ident();
    let states = empty.into_substates();

    let name = format_ident!("{}Locker", ident);

    let state_struct_declarations = generate_state_struct_declarations(&states);
    let builder_struct = generate_builder_struct(&name, fields.iter());
    let impl_states = generate_impl_for_all_states(&name, &states, fields);
    let trait_implementation = generate_trait_implementation(ident, &name, &empty_name, fields);

    quote! {
        #state_struct_declarations
        #builder_struct
        #impl_states
        #trait_implementation
    }
    .into()
}
