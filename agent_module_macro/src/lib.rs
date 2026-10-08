extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemImpl};

fn has_fn(input: &ItemImpl, name: &str) -> bool {
    input.items.iter().any(|item| match item {
        syn::ImplItem::Fn(method) => method.sig.ident == name,
        _ => false,
    })
}

#[proc_macro_attribute]
pub fn handle_impl(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    if !has_fn(&input, "new") {
        let method = quote! {
            fn new() -> Self {
                Self::default()
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }
    if !has_fn(&input, "create_task") {
        let method = quote! {
            fn create_task(&self) -> std::sync::Arc<dyn agent_module_trait::AgentModuleTask> {
                std::sync::Arc::new(Self::new())
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn handle_struct_impl(_args: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入的 struct
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(Default)]
        #input
    };

    TokenStream::from(expanded)
}
