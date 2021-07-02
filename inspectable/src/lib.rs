#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate proc_macro;
use proc_macro::TokenStream;

mod expand_struct;

#[proc_macro_derive(Inspectable, attributes(inspectable))]
pub fn derive_inspectable(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => expand_struct::expand_struct(&input, data).into(),
        syn::Data::Enum(data) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}
