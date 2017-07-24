#![recursion_limit="128"]
extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;
use syn::{
    Body,
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

#[proc_macro_derive(HasContour)]
pub fn has_contour(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let name = &ast.ident;
    let (impl_g, ty_g, where_g) = ast.generics.split_for_impl();
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
                impl #impl_g HasContour for #name #ty_g #where_g {
                    fn contour() -> Contour {
                        Contour::Struct {
                            name: stringify!(#name),
                            size: ::std::mem::size_of::<#name #ty_g>(),
                            type_id: ::std::any::TypeId::of::<#name #ty_g>(),
                            fields: vec![#(#fields),*],
                        }
                    }

                    unsafe extern "C" fn enum_variant(_self: *const u8) -> isize {
                        -1
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
                impl #impl_g HasContour for #name #ty_g #where_g {
                    fn contour() -> Contour {
                        Contour::Tuple {
                            name: stringify!(#name),
                            size: ::std::mem::size_of::<#name #ty_g>(),
                            type_id: ::std::any::TypeId::of::<#name #ty_g>(),
                            fields: vec![#(#fields),*],
                        }
                    }

                    unsafe extern "C" fn enum_variant(_self: *const u8) -> isize {
                        -1
                    }
                }
            }
        },
        Body::Struct(VariantData::Unit) => {
            quote! {
                impl HasContour for #name {
                    fn contour() -> Contour {
                        Contour::Unit {
                            name: stringify!(#name),
                            type_id: ::std::any::TypeId::of::<#name>(),
                        }
                    }

                    unsafe extern "C" fn enum_variant(_self: *const u8) -> isize {
                        -1
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
                                            #name::#vname { ref #fname, ..} =>
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

            let enum_variants: Vec<_> = variants.iter()
                .enumerate()
                .map(|(i, v)| {
                    let vname = &v.ident;
                    match v.data {
                        VariantData::Struct(..) => quote!(#name::#vname {..} => #i as isize),
                        VariantData::Tuple(..) => quote!(#name::#vname(..) => #i as isize),
                        VariantData::Unit => quote!(#name::#vname => #i as isize),
                    }
                })
                .collect();

            quote! {
                impl #impl_g HasContour for #name #ty_g #where_g {
                    fn contour() -> Contour {
                        Contour::Enum {
                            name: stringify!(#name),
                            size: ::std::mem::size_of::<#name>(),
                            type_id: ::std::any::TypeId::of::<#name>(),
                            variants: vec![#(#variant_fields),*],
                        }
                    }

                    unsafe extern "C" fn enum_variant(s: *const u8) -> isize {
                        let s = s as *const Self;
                        match *s { #(#enum_variants),* }
                    }
                }
            }
        },
    };

    gen.parse().unwrap()
}
