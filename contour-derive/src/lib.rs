#![recursion_limit="128"]
extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use proc_macro::TokenStream;
use syn::{
    Body,
    Ident,
    VariantData,
};
use quote::{
    ToTokens,
    Tokens,
};

struct TupleField(usize);
impl ToTokens for TupleField {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(&format!("{}", self.0));
    }
}

#[proc_macro_derive(Introspectable)]
pub fn introspectable(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let name = &ast.ident;
    let (impl_g, ty_g, where_g) = ast.generics.split_for_impl();
    let chart_children = match ast.body {
        Body::Struct(VariantData::Struct(ref fields)) => fields.iter()
            .map(|f| {let ty = &f.ty; quote!({#ty::chart(map);})})
            .collect(),
        Body::Struct(VariantData::Tuple(ref fields)) => fields.iter()
            .map(|f| {let ty = &f.ty; quote!({#ty::chart(map);})})
            .collect(),
        Body::Struct(VariantData::Unit) => vec![],
        Body::Enum(ref variants) => {
            variants.iter()
                .flat_map(|variant| match variant.data {
                    VariantData::Struct(ref fields) => fields.iter()
                        .map(|f| {let ty = &f.ty; quote!({#ty::chart(map);})})
                        .collect(),
                    VariantData::Tuple(ref fields) => fields.iter()
                        .map(|f| {let ty = &f.ty; quote!({#ty::chart(map);})})
                        .collect(),
                    VariantData::Unit => vec![],
                })
                .collect()
        },
    };

    let gen = match ast.body {
        Body::Struct(VariantData::Struct(ref fields)) => {
            let fields: Vec<_> = fields.iter()
                .map(|f| {
                    let ident = f.ident.as_ref().expect("Unnamed struct field?");
                    let ty = &f.ty;
                    quote! {{
                        let _bomb: #name #ty_g = unsafe {::std::mem::uninitialized()};
                        let _base = &_bomb as *const _ as *const u8;
                        let _us = &_bomb.#ident as *const _ as *const u8;
                        let offset = _base.offset_to(_us).unwrap() as usize;
                        ::std::mem::forget(_bomb);
                        StructField {
                            name: stringify!(#ident),
                            type_id: ::std::any::TypeId::of::<#ty>(),
                            offset: offset,
                        }
                    }}
                })
                .collect();
            quote! {
                impl #impl_g Introspectable for #name #ty_g #where_g {
                    fn chart<CM: ContourMap>(map: &CM) {
                        let contour = #name #ty_g ::contour();
                        if map.register(contour) {
                            return;
                        }
                        #(#chart_children)*
                    }

                    fn contour() -> Contour {
                        Contour::Struct {
                            name: stringify!(#name),
                            size: ::std::mem::size_of::<#name #ty_g>(),
                            type_id: ::std::any::TypeId::of::<#name #ty_g>(),
                            fields: vec![#(#fields),*],
                        }
                    }
                }
            }
        },
        Body::Struct(VariantData::Tuple(ref fields)) => {
            let fields: Vec<_> = fields.iter()
                .enumerate()
                .map(|(i, f)| {
                    let field = TupleField(i);
                    let ty = &f.ty;
                    quote! {{
                        let _bomb: #name #ty_g = unsafe {::std::mem::uninitialized()};
                        let _base = &_bomb as *const _ as *const u8;
                        let _us = &_bomb.#field as *const _ as *const u8;
                        let offset = _base.offset_to(_us).unwrap() as usize;
                        ::std::mem::forget(_bomb);
                        TupleField {
                            ix: #i,
                            type_id: ::std::any::TypeId::of::<#ty>(),
                            offset: offset,
                        }
                    }}
                })
                .collect();
            quote! {
                impl #impl_g Introspectable for #name #ty_g #where_g {
                    fn chart<CM: ContourMap>(map: &CM) {
                        let contour = #name #ty_g ::contour();
                        if map.register(contour) {
                            return;
                        }
                        #(#chart_children)*
                    }

                    fn contour() -> Contour {
                        Contour::Tuple {
                            name: stringify!(#name),
                            size: ::std::mem::size_of::<#name #ty_g>(),
                            type_id: ::std::any::TypeId::of::<#name #ty_g>(),
                            fields: vec![#(#fields),*],
                        }
                    }
                }
            }
        },
        Body::Struct(VariantData::Unit) => {
            quote! {
                impl Introspectable for #name {
                    fn chart<CM: ContourMap>(map: &CM) {
                        let contour = #name #ty_g ::contour();
                        if map.register(contour) {
                            return;
                        }
                        #(#chart_children)*
                    }

                    fn contour() -> Contour {
                        Contour::Unit {
                            name: stringify!(#name),
                            type_id: ::std::any::TypeId::of::<#name>(),
                        }
                    }
                }
            }
        },
        Body::Enum(ref variants) => {
            let variant_fields: Vec<_> = variants.iter()
                .map(|variant| {
                    let vname = &variant.ident;
                    match variant.data {
                        VariantData::Struct(ref fields) => {
                            let initializer: Vec<_> = fields.iter()
                                .map(|field| field.ident.as_ref().unwrap())
                                .map(|f| quote! {#f: unsafe {::std::mem::uninitialized()}})
                                .collect();

                            let fields: Vec<_> = fields.iter()
                                .map(|field| {
                                    let fname = field.ident.as_ref().unwrap();
                                    let ty = &field.ty;
                                    let _initializer = initializer.clone();
                                    quote! {{
                                        let _bomb = #name::#vname {
                                            #(#_initializer),*
                                        };
                                        let _base = &_bomb as *const _ as *const u8;
                                        let _us = match _bomb {
                                            #name::#vname { ref #fname, .. } =>
                                                #fname as *const _ as *const u8,
                                            _ => panic!("Wrong variant"),
                                        };
                                        let offset = _base.offset_to(_us).unwrap() as usize;
                                        ::std::mem::forget(_bomb);
                                        StructField {
                                            name: stringify!(#fname),
                                            type_id: ::std::any::TypeId::of::<#ty>(),
                                            offset: offset,
                                        }
                                    }}
                                })
                                .collect();
                            quote! {
                                Variant {
                                    name: stringify!(#vname),
                                    fields: VariantFields::Struct(vec![#(#fields),*])
                                }
                            }
                        },
                        VariantData::Tuple(ref fields) => {
                            let n = fields.len();
                            let initializer: Vec<_> = (0..n)
                                .map(|_| quote! {unsafe {::std::mem::uninitialized()}})
                                .collect();

                            let fields: Vec<_> = fields.iter()
                                .enumerate()
                                .map(|(i, field)| {
                                    let _initializer = initializer.clone();
                                    let ty = &field.ty;

                                    let mut pat = vec![];
                                    pat.extend((0..i).map(|_| quote!(_)));
                                    pat.push(quote!(ref us));
                                    pat.extend(((i+1)..n).map(|_| quote!(_)));

                                    quote! {{
                                        let _bomb = #name::#vname(#(#_initializer),*);
                                        let _base = &_bomb as *const _ as *const u8;
                                        let _us = match _bomb {
                                            #name::#vname( #(#pat),* ) =>
                                                us as *const _ as *const u8,
                                            _ => panic!("Wrong variant"),
                                        };
                                        let offset = _base.offset_to(_us).unwrap() as usize;
                                        ::std::mem::forget(_bomb);
                                        TupleField {
                                            ix: #i,
                                            type_id: ::std::any::TypeId::of::<#ty>(),
                                            offset: offset,
                                        }
                                    }}
                                })
                                .collect();
                            quote! {
                                Variant {
                                    name: stringify!(#vname),
                                    fields: VariantFields::Tuple(vec![#(#fields),*]),
                                }
                            }
                        },
                        VariantData::Unit => quote! {
                            Variant {
                                name: stringify!(#vname),
                                fields: VariantFields::Unit,
                            }
                        }
                    }
                })
                .collect();

            // Is this sufficient?
            let mut hasher = DefaultHasher::new();
            hasher.write(format!("{:?}", ast).as_bytes());
            let fn_name = Ident::from(format!("gettag_{:x}", hasher.finish()));
            let enum_variants: Vec<_> = variants.iter()
                .enumerate()
                .map(|(i, v)| {
                    let vname = &v.ident;
                    match v.data {
                        VariantData::Struct(..) => quote!(#name::#vname {..} => #i),
                        VariantData::Tuple(..) => quote!(#name::#vname(..) => #i),
                        VariantData::Unit => quote!(#name::#vname => #i),
                    }
                })
                .collect();
            quote! {
                unsafe extern "C" fn #fn_name(_self: *const u8) -> usize {
                    let s = _self as *const #name #ty_g;
                    match *s { #(#enum_variants),* }
                }
                impl #impl_g Introspectable for #name #ty_g #where_g {
                    fn chart<CM: ContourMap>(map: &CM) {
                        let contour = #name #ty_g ::contour();
                        if map.register(contour) {
                            return;
                        }
                        #(#chart_children)*
                    }

                    fn contour() -> Contour {
                        Contour::Enum {
                            name: stringify!(#name),
                            size: ::std::mem::size_of::<#name>(),
                            type_id: ::std::any::TypeId::of::<#name>(),
                            variants: vec![#(#variant_fields),*],
                            tag: #fn_name,
                        }
                    }
                }
            }
        },
    };


    gen.parse().unwrap()
}
