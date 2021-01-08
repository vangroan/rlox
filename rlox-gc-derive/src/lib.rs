use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{self, parse_macro_input, Data, DeriveInput, Lit};

#[proc_macro_derive(Scan)]
pub fn derive_gc_scan(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ty = ast.ident.clone();

    let trace_body = match ast.data {
        Data::Struct(ref data) => {
            data.fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    match field.ident.clone() {
                        Some(ident) => {
                            quote! { self.#ident.scan(ctx); }
                        }
                        None => {
                            // Tuple index
                            let idx = Lit::new(Literal::usize_unsuffixed(index));
                            quote! { self.#idx.scan(ctx); }
                        }
                    }
                })
                .collect::<Vec<_>>()
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => unimplemented!("Untagged unions not implemented yet"),
    };

    let root_body = match ast.data {
        Data::Struct(ref data) => {
            data.fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    match field.ident.clone() {
                        Some(ident) => {
                            quote! { self.#ident.root(); }
                        }
                        None => {
                            // Tuple index
                            let idx = Lit::new(Literal::usize_unsuffixed(index));
                            quote! { self.#idx.root(); }
                        }
                    }
                })
                .collect::<Vec<_>>()
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => unimplemented!("Untagged unions not implemented yet"),
    };

    let unroot_body = match ast.data {
        Data::Struct(ref data) => {
            data.fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    match field.ident.clone() {
                        Some(ident) => {
                            quote! { self.#ident.unroot(); }
                        }
                        None => {
                            // Tuple struct
                            let idx = Lit::new(Literal::usize_unsuffixed(index));
                            quote! { self.#idx.unroot(); }
                        }
                    }
                })
                .collect::<Vec<_>>()
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => unimplemented!("Untagged unions not implemented yet"),
    };

    let gen = quote! {
        unsafe impl rlox_gc::scan::Scan for #ty {
            fn scan(&self, ctx: &mut rlox_gc::context::Context<'_>) {
                use rlox_gc::scan::Scan;
                #(#trace_body)*
            }

            fn root(&self) {
                use rlox_gc::scan::Scan;
                #(#root_body)*
            }

            fn unroot(&self) {
                use rlox_gc::scan::Scan;
                #(#unroot_body)*
            }
        }
    };

    // println!("{}", gen);

    gen.into()
}
