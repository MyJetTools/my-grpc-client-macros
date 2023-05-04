use proc_macro::TokenStream;

pub fn generate(attr: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    quote::quote! {
        #ast
    }
    .into()
}
