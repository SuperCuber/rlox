extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{self, DataEnum, Fields, FieldsNamed, FieldsUnnamed, Ident, Variant};

#[proc_macro_derive(EncodeInstruction)]
pub fn encode_instruction_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_encode_instruction(&ast)
}

fn impl_encode_instruction(ast: &syn::DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    match &ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let variants_encode: Vec<_> = variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| encode_variant(idx as u8, ident, variant))
                .collect();

            let variants_decode: Vec<_> = variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| decode_variant(idx as u8, ident, variant))
                .collect();

            quote! {
                impl ::encode_instruction::EncodeInstruction for #ident {
                    fn encode(self, buf: &mut Vec<u8>) {
                        match self {
                            #(#variants_encode),*
                        }
                    }

                    fn decode(buf: &[u8]) -> Option<(Self, usize)> {
                        let mut __length = 1;
                        let ans = match buf.get(0)? {
                            #(#variants_decode),*,
                            _ => return None,
                        };
                        Some((ans, __length))
                    }
                }
            }
            .into()
        }
        syn::Data::Struct(_) => todo!(),
        syn::Data::Union(_) => panic!("Unions are not supported"),
    }
}

fn encode_variant(idx: u8, enum_ident: &Ident, variant: &Variant) -> impl ToTokens {
    let ident = &variant.ident;
    let idx = idx as u8;

    match &variant.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let names: Vec<_> = named.into_iter().map(|f| &f.ident).collect();
            quote! {
                #enum_ident::#ident { #(#names),* } => {
                    buf.push(#idx);
                    #(
                        ::encode_instruction::EncodeInstruction::encode(#names, buf);
                    )*
                }
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let names: Vec<_> = (0..unnamed.len())
                .map(|idx| format_ident!("field{idx}"))
                .collect();
            quote! {
                #enum_ident::#ident ( #(#names),* ) => {
                    buf.push(#idx);
                    #(
                        ::encode_instruction::EncodeInstruction::encode(#names, buf);
                    )*
                }
            }
        }
        Fields::Unit => quote! {
            #enum_ident::#ident => {
                buf.push(#idx);
            }
        },
    }
}

fn decode_variant(idx: u8, enum_ident: &Ident, variant: &Variant) -> impl ToTokens {
    let ident = &variant.ident;
    let idx = idx as u8;

    match &variant.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let names: Vec<_> = named.into_iter().map(|f| &f.ident).collect();
            quote! {
                #idx => {
                    #(
                        let (#names, __current_field_length) = ::encode_instruction::EncodeInstruction::decode(&buf[__length..])?;
                        __length += __current_field_length;
                    )*
                    #enum_ident::#ident { #(#names),* }
                }
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let names: Vec<_> = (0..unnamed.len())
                .map(|idx| format_ident!("field{idx}"))
                .collect();
            quote! {
                #idx => {
                    #(
                        let (#names, __current_field_length) = ::encode_instruction::EncodeInstruction::decode(&buf[__length..])?;
                        __length += __current_field_length;
                    )*
                    #enum_ident::#ident ( #(#names),* )
                }
            }
        }
        Fields::Unit => quote! {
            #idx => #enum_ident::#ident
        },
    }
}
