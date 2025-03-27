#![feature(extract_if)]

use proc_macro::TokenStream as InterfaceTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{DeriveInput, parse_quote};

#[proc_macro_derive(RustState, attributes(state_root))]
pub fn derive_prototype_element(token_stream: InterfaceTokenStream) -> InterfaceTokenStream {
    let DeriveInput {
        ident,
        attrs,
        data,
        generics,
        ..
    } = syn::parse(token_stream).expect("failed to parse token stream");

    let is_root = attrs
        .iter()
        .filter_map(|attribute| syn::parse::<syn::Ident>(attribute.meta.to_token_stream().into()).ok())
        .any(|ident| ident.to_string().as_str() == "state_root");

    let root_impl = is_root.then(|| impl_for_root(ident.clone(), generics.clone()));
    let inner_impl = impl_for_inner(ident, data, generics);

    quote! {
        #root_impl

        #inner_impl
    }
    .into()
}

fn impl_for_root(ident: syn::Ident, generics: syn::Generics) -> TokenStream {
    let lifetimes = generics.lifetimes().map(|lifetime| quote!(&#lifetime ())).collect::<Vec<_>>();
    let type_params = generics.type_params().map(|type_param| quote!(#type_param)).collect::<Vec<_>>();

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let extension_trait_name = syn::Ident::new(&format!("{}RootExt", ident), ident.span());

    quote_spanned! { Span::mixed_site() =>
        impl #impl_generics rust_state::StateMarker for #ident #type_generics #where_clause {}

        pub trait #extension_trait_name {
            fn path() -> impl rust_state::Path<#ident, #ident> {
                struct AnonymousPath #type_generics #where_clause {
                    _marker: std::marker::PhantomData<(#(#lifetimes,)* #(#type_params,)*)>,
                }

                impl #impl_generics Clone for AnonymousPath #type_generics #where_clause {
                    fn clone(&self) -> Self {
                        Self { _marker: std::marker::PhantomData }
                    }
                }

                impl #impl_generics Copy for AnonymousPath #type_generics #where_clause {}

                impl #impl_generics rust_state::Path<#ident, #ident> for AnonymousPath #type_generics #where_clause {
                    fn follow<'a>(&self, state: &'a #ident) -> Option<&'a #ident> {
                        Some(state)
                    }

                    fn follow_mut<'a>(&self, state: &'a mut #ident) -> Option<&'a mut #ident> {
                        Some(state)
                    }
                }

                impl #impl_generics rust_state::Selector<#ident, #ident> for AnonymousPath #type_generics #where_clause {
                    fn select<'a>(&'a self, state: &'a #ident) -> Option<&'a #ident> {
                        Some(state)
                    }
                }

                AnonymousPath { _marker: std::marker::PhantomData }
            }
        }

        impl #impl_generics #extension_trait_name for #ident #type_generics #where_clause {}
    }
}

fn impl_for_inner(ident: syn::Ident, data: syn::Data, generics: syn::Generics) -> TokenStream {
    let (impl_generics, type_generics, _where_clause) = generics.split_for_impl();

    let lifetimes = generics.lifetimes().map(|lifetime| quote!(&#lifetime ())).collect::<Vec<_>>();
    let type_params = generics.type_params().map(|type_param| quote!(#type_param)).collect::<Vec<_>>();

    let mut struct_generics = generics.clone();
    struct_generics.params.push(parse_quote!(S: 'static));
    struct_generics.params.push(parse_quote!(P));
    struct_generics.params.push(parse_quote!(const SAFE: bool));
    let (struct_impl_generics, struct_type_generics, struct_where_clause) = struct_generics.split_for_impl();

    let mut struct_creation_generics = generics.clone();
    struct_creation_generics.params.push(parse_quote!(S));
    struct_creation_generics.params.push(parse_quote!(P));
    struct_creation_generics.params.push(parse_quote!(const SAFE: bool));

    let mut clone_generics = struct_generics.clone();
    let clone_where_clause = clone_generics.make_where_clause();
    clone_where_clause.predicates.push(parse_quote!(P: Copy));

    let mut path_generics = generics.clone();
    path_generics.params.push(parse_quote!(S: 'static));
    path_generics
        .params
        .push(parse_quote!(P: rust_state::Path<S, #ident #type_generics, SAFE>));
    path_generics.params.push(parse_quote!(const SAFE: bool));
    let (path_impl_generics, _, path_where_clause) = path_generics.split_for_impl();

    let mut selector_generics = generics.clone();
    selector_generics.params.push(parse_quote!(S: 'static));
    selector_generics
        .params
        .push(parse_quote!(P: rust_state::Path<S, #ident #type_generics, SAFE>));
    selector_generics.params.push(parse_quote!(const SAFE: bool));
    let (selector_impl_generics, _, selector_where_clause) = selector_generics.split_for_impl();

    let extension_trait_name = syn::Ident::new(&format!("{}PathExt", ident), ident.span());

    let mut extension_trait_methods = Vec::new();

    match data {
        syn::Data::Struct(data_struct) => {
            for field in data_struct.fields.into_iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = field.ty;

                extension_trait_methods.push(quote_spanned! { Span::mixed_site() =>
                    fn #field_name(self) -> impl rust_state::Path<StateTwo, #field_type, SAFE> {
                        pub struct AnonymousPath #struct_creation_generics #struct_where_clause {
                            path: P,
                            _marker: std::marker::PhantomData<(S, #(#lifetimes,)* #(#type_params,)*)>,
                        }

                        impl #struct_impl_generics Clone for AnonymousPath #struct_type_generics #clone_where_clause {
                            fn clone(&self) -> Self {
                                Self {
                                    path: self.path,
                                    _marker: std::marker::PhantomData,
                                }
                            }
                        }

                        impl #struct_impl_generics Copy for AnonymousPath #struct_type_generics #clone_where_clause {}

                        impl #path_impl_generics rust_state::Path<S, #field_type, SAFE> for AnonymousPath #struct_type_generics #path_where_clause {
                            fn follow<'a>(&self, state: &'a S) -> Option<&'a #field_type> {
                                Some(&self.path.follow(state)?.#field_name)
                            }

                            fn follow_mut<'a>(&self, state: &'a mut S) -> Option<&'a mut #field_type> {
                                Some(&mut self.path.follow_mut(state)?.#field_name)
                            }
                        }

                        impl #selector_impl_generics rust_state::Selector<S, #field_type, SAFE> for AnonymousPath #struct_type_generics #selector_where_clause {
                            fn select<'a>(&'a self, state: &'a S) -> Option<&'a #field_type> {
                                <Self as rust_state::Path<S, #field_type, SAFE>>::follow(self, state)
                            }
                        }

                        AnonymousPath { path: self, _marker: std::marker::PhantomData }
                    }
                });
            }
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }

    quote_spanned! { Span::mixed_site() =>
        pub trait #extension_trait_name<StateTwo: 'static, const SAFE: bool>: rust_state::Path<StateTwo, #ident, SAFE> {
            #(#extension_trait_methods)*
        }

        impl<StateTwo, T, const SAFE: bool> #impl_generics #extension_trait_name<StateTwo, SAFE> for T
            where
                StateTwo: 'static,
                T: rust_state::Path<StateTwo, #ident, SAFE>,
            {}
    }
}
