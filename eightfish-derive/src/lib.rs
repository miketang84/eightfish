use proc_macro::{self, TokenStream};
use quote::quote;
//use syn::{parse_macro_input, DeriveInput};
use syn::{parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed};

#[proc_macro_derive(EightFishHelper)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let field_names = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let idents = named.iter().map(|f| &f.ident);
                format!(
                    "{}",
                    quote! {#(#idents),*}
                )
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let field_placeholders = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let mut placeholders: Vec<String> = 
                    named.iter().enumerate().map(|(i, _)| (i + 1).to_string()).collect::<Vec<String>>();
                placeholders.push(placeholders.len().to_string());
                
                format!(
                    "{}",
                    quote! {#($#placeholders),*}
                )
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let idents = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let idents = named.iter().map(|f| &f.ident);
                idents
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    let idents2 = idents.clone();

    let orders = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                named.iter().enumerate().map(|(i, _)| i+1)
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let output = quote! {
        impl #ident {
            fn field_names() -> String {
                format!("{}", #field_names)
            }
            fn row_placeholders() -> String {
                #field_placeholders.to_string().replace("\"", "")
            }
            fn to_vec(&self) -> Vec<String> {
                let mut field_vec: Vec<String> = Vec::new();
                #(
                    field_vec.push(self.#idents.to_string());
                )*
                field_vec
            }
            fn to_row(&self, hash: String) -> Vec<String> {
                let mut row = self.to_vec();
                row.insert(0, hash);
                row
            }
            fn from_row(row: Vec<String>) -> #ident {
                let mut settings = #ident::default();
                #(
                    settings.#idents2 = row[#orders].to_string();
                )*
                settings
            }
            fn get_hash_from_row(row: Vec<String>) -> String {
                row[0].to_string()
            }
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
