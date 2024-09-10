extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use std::ffi::c_char;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{parse_macro_input, Token, Ident, TraitItemFn, Visibility, FnArg, Pat};
use syn::punctuated::Punctuated;

struct Interface {
    vis: Visibility,
    ident: Ident,
    colon_token: Option<Token![:]>,
    parent: Option<Ident>,
    methods: Vec<TraitItemFn>
}

impl Parse for Interface {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<Visibility>()?;
        let _ = input.parse::<Token![trait]>()?;
        let ident = input.parse::<Ident>()?;

        let colon_token: Option<Token![:]> = input.parse()?;

        let mut parent = None;
        if colon_token.is_some() {
            parent = Some(input.parse::<Ident>()?);
        }

        let content: ParseBuffer;

        syn::braced!(content in input);
        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(content.parse::<TraitItemFn>()?);
        }

        Ok(Self {
            vis,
            ident,
            colon_token,
            parent,
            methods
        })
    }
}

impl Interface {
    fn gen_tokens(&self, uid: [c_char; 16]) -> syn::Result<TokenStream2> {
        let vis = &self.vis;
        let ident = &self.ident;
        let vtable_name = quote::format_ident!("{}_Vtbl", ident);
        // let guid = guid.to_tokens()?;
        // let implementation = self.gen_implementation();
        let com_trait = self.get_trait(uid);
        let vtable = self.gen_vtable(&vtable_name);
        // let conversions = self.gen_conversions();

        Ok(quote! {
            #com_trait
            #vtable
        })
    }

    fn get_trait(&self, uid: [c_char; 16]) -> TokenStream2 {
        let vis = &self.vis;
        let ident = &self.ident;

        let vtable_name = quote::format_ident!("{}_Vtbl", ident);
        let impl_name = quote::format_ident!("{}_Impl", ident);

        let impl_parent = &self.parent.clone().map_or(quote!{}, |p| {
            let impl_name = quote::format_ident!("{}_Impl", p);
            quote! {
                impl #impl_name for #ident {}
            }
        });

        let methods = self.methods
            .iter()
            .map(|method| {
                let method_ident = &method.sig.ident;
                let method_impl_ident = quote::format_ident!("{}_impl", &method.sig.ident);
                let args = &method.sig.inputs
                    .iter()
                    .filter(|arg| matches!(arg, FnArg::Typed(..)))
                    .cloned()
                    .collect::<Punctuated<FnArg, Token![,]>>();

                let arg_inputs = args.iter().map(|arg| {
                    match arg {
                        FnArg::Typed(pat) => {
                            *(pat.pat).clone()
                        }

                        _ => {
                            panic!("")
                        }
                    }
                }).collect::<Punctuated<Pat, Token![,]>>();

                let output = &method.sig.output;

                quote! {
                    unsafe fn #method_impl_ident(&mut self, #args) #output {
                        ((*(self.vtable)).#method_ident)(self, #arg_inputs)
                    }
                }
            })
            .collect::<Vec<_>>();

        let trait_methods = self.methods
            .iter()
            .map(|method| {
                let method_ident = &method.sig.ident;
                let method_impl_ident = quote::format_ident!("{}_impl", &method.sig.ident);
                let args = &method.sig.inputs
                    .iter()
                    .filter(|arg| matches!(arg, FnArg::Typed(..)))
                    .cloned()
                    .collect::<Punctuated<FnArg, Token![,]>>();

                let arg_inputs = args.iter().map(|arg| {
                    match arg {
                        FnArg::Typed(pat) => {
                            *(pat.pat).clone()
                        }

                        _ => {
                            panic!("")
                        }
                    }
                }).collect::<Punctuated<Pat, Token![,]>>();

                let output = &method.sig.output;

                quote! {
                    unsafe fn #method_ident(&mut self, #args) #output {
                         (*(self as *mut _ as *mut #ident)).#method_impl_ident(#arg_inputs)
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            #[repr(C)]
            #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
            pub struct #ident {
                pub vtable: &'static #vtable_name,
            }

            impl #ident {
                #(#methods)*
            }

            pub trait #impl_name {
                #(#trait_methods)*
            }

            impl #impl_name for #ident {}

            #impl_parent

            impl Interface for #ident {
                type VTable = #vtable_name;
                const iid: FUID = [#(#uid),*];

                fn vtable(&self) -> &'static Self::VTable {
                    &self.vtable
                }
            }
        }
    }

    fn gen_vtable(&self, vtable_name: &Ident) -> TokenStream2 {
        let name = &self.ident;

        let methods = self.methods
            .iter()
            .map(|method| {
                let ident = &method.sig.ident;
                let args = &method.sig.inputs
                    .iter()
                    .filter(|arg| matches!(arg, FnArg::Typed(..)))
                    .cloned()
                    .collect::<Punctuated<FnArg, Token![,]>>();
                let output = &method.sig.output;

                quote! {
                    #ident: unsafe extern "thiscall" fn(this: *mut #name, #args) #output,
                }
            })
            .collect::<Vec<_>>();

        let parent = &self.parent.clone().map(|p| {
            let vtable_name = quote::format_ident!("{}_Vtbl", p);
            quote! {
                pub _base: #vtable_name,
            }
        });

        quote! {
            #[repr(C)]
            #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
            pub struct #vtable_name {
                #parent
                #(#methods)*
            }
        }
    }
}

fn inline_uid(l1: u32, l2: u32, l3: u32, l4: u32) -> [c_char; 16] {
    [
        ((l1 & 0x000000FF)) as c_char,
        ((l1 & 0x0000FF00) >> 8) as c_char,
        ((l1 & 0x00FF0000) >> 16) as c_char,
        ((l1 & 0xFF000000) >> 24) as c_char,

        ((l2 & 0x00FF0000) >> 16) as c_char,
        ((l2 & 0xFF000000) >> 24) as c_char,
        ((l2 & 0x000000FF)) as c_char,
        ((l2 & 0x0000FF00) >> 8) as c_char,

        ((l3 & 0xFF000000) >> 24) as c_char,
        ((l3 & 0x00FF0000) >> 16) as c_char,
        ((l3 & 0x0000FF00) >> 8) as c_char,
        ((l3 & 0x000000FF)) as c_char,

        ((l4 & 0xFF000000) >> 24) as c_char,
        ((l4 & 0x00FF0000) >> 16) as c_char,
        ((l4 & 0x0000FF00) >> 8) as c_char,
        ((l4 & 0x000000FF)) as c_char,
    ]

    // [
    //     ((l1 & 0xFF000000) >> 24) as c_char,
    //     ((l1 & 0x00FF0000) >> 16) as c_char,
    //     ((l1 & 0x0000FF00) >>  8) as c_char,
    //     ((l1 & 0x000000FF)      ) as c_char,
    //
    //     ((l2 & 0xFF000000) >> 24) as c_char,
    //     ((l2 & 0x00FF0000) >> 16) as c_char,
    //     ((l2 & 0x0000FF00) >>  8) as c_char,
    //     ((l2 & 0x000000FF)      ) as c_char,
    //
    //     ((l3 & 0xFF000000) >> 24) as c_char,
    //     ((l3 & 0x00FF0000) >> 16) as c_char,
    //     ((l3 & 0x0000FF00) >>  8) as c_char,
    //     ((l3 & 0x000000FF)      ) as c_char,
    //
    //     ((l4 & 0xFF000000) >> 24) as c_char,
    //     ((l4 & 0x00FF0000) >> 16) as c_char,
    //     ((l4 & 0x0000FF00) >>  8) as c_char,
    //     ((l4 & 0x000000FF)      ) as c_char,
    // ]
}

#[proc_macro_attribute]
pub fn interface(args: TokenStream, input: TokenStream) -> TokenStream {
    let clone = input.clone();
    let uid = parse_macro_input!(args with Punctuated::<syn::LitInt, Token![,]>::parse_terminated);
    let interface = parse_macro_input!(clone as Interface);

    let l1 = uid.get(0).unwrap().base10_parse().unwrap();
    let l2 = uid.get(1).unwrap().base10_parse().unwrap();
    let l3 = uid.get(2).unwrap().base10_parse().unwrap();
    let l4 = uid.get(3).unwrap().base10_parse().unwrap();

    let uid = inline_uid(l1, l2, l3, l4);

    // for i in uid {
    //     println!("attr: \"{i}\"");
    // }
    // println!("item: \"{input}\"");


    let tokens = match interface.gen_tokens(uid) {
        Ok(t) => t,
        Err(e) => return e.to_compile_error().into(),
    };

    println!("{}", tokens);

    tokens.into()
}