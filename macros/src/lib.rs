#![feature(extract_if)]

use case::CaseExt;
use proc_macro::TokenStream as InterfaceTokenStream;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
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

    let struct_name = syn::Ident::new(&format!("{}Path", ident), ident.span());
    let extension_trait_name = syn::Ident::new(&format!("{}RootExt", ident), ident.span());

    quote! {
        impl #impl_generics rust_state::StateMarker for #ident #type_generics #where_clause {}

        struct #struct_name #type_generics #where_clause {
            _marker: std::marker::PhantomData<(#(#lifetimes,)* #(#type_params,)*)>,
        }

        impl #impl_generics Clone for #struct_name #type_generics #where_clause {
            fn clone(&self) -> Self {
                Self { _marker: std::marker::PhantomData }
            }
        }

        impl #impl_generics Copy for #struct_name #type_generics #where_clause {}

        impl #impl_generics rust_state::Path<#ident, #ident> for #struct_name #type_generics #where_clause {
            fn follow<'_a>(&self, state: &'_a #ident) -> Option<&'_a #ident> {
                Some(state)
            }

            fn follow_mut<'_a>(&self, state: &'_a mut #ident) -> Option<&'_a mut #ident> {
                Some(state)
            }
        }

        impl #impl_generics rust_state::Selector<#ident, #ident> for #struct_name #type_generics #where_clause {
            fn select<'_a>(&'_a self, state: &'_a #ident) -> Option<&'_a #ident> {
                Some(state)
            }
        }

        pub trait #extension_trait_name {
            fn path() -> impl rust_state::Path<#ident, #ident> {
                #struct_name { _marker: std::marker::PhantomData }
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

    let mut base_getters = Vec::new();
    let mut extension_trait_methods = Vec::new();

    match data {
        syn::Data::Struct(data_struct) => {
            for field in data_struct.fields.into_iter() {
                let field_name = field.ident.as_ref().unwrap();
                let struct_name = syn::Ident::new(
                    &format!("{}{}Path", ident, field.ident.as_ref().unwrap().to_string().to_camel()),
                    field.ident.as_ref().unwrap().span(),
                );
                let field_type = field.ty;

                base_getters.push(quote! {
                    pub struct #struct_name #struct_creation_generics #struct_where_clause {
                        path: P,
                        _marker: std::marker::PhantomData<(S, #(#lifetimes,)* #(#type_params,)*)>,
                    }

                    impl #struct_impl_generics Clone for #struct_name #struct_type_generics #clone_where_clause {
                        fn clone(&self) -> Self {
                            Self {
                                path: self.path,
                                _marker: std::marker::PhantomData,
                            }
                        }
                    }

                    impl #struct_impl_generics Copy for #struct_name #struct_type_generics #clone_where_clause {}

                    impl #path_impl_generics rust_state::Path<S, #field_type, SAFE> for #struct_name #struct_type_generics #path_where_clause {
                        fn follow<'_a>(&self, state: &'_a S) -> Option<&'_a #field_type> {
                            Some(&self.path.follow(state)?.#field_name)
                        }

                        fn follow_mut<'_a>(&self, state: &'_a mut S) -> Option<&'_a mut #field_type> {
                            Some(&mut self.path.follow_mut(state)?.#field_name)
                        }
                    }

                    impl #selector_impl_generics rust_state::Selector<S, #field_type, SAFE> for #struct_name #struct_type_generics #selector_where_clause {
                        fn select<'_a>(&'_a self, state: &'_a S) -> Option<&'_a #field_type> {
                            <Self as rust_state::Path<S, #field_type, SAFE>>::follow(self, state)
                        }
                    }
                });

                extension_trait_methods.push(quote! {
                    fn #field_name(self) -> impl rust_state::Path<StateTwo, #field_type, SAFE> {
                        #struct_name { path: self, _marker: std::marker::PhantomData }
                    }
                });
            }
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }

    quote! {
        #(#base_getters)*

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
