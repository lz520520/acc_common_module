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

    if !has_fn(&input, "clear_cache") {
        let method = quote! {
            fn clear_cache(&self, tag: &str) {
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }

    if !has_fn(&input, "close") {
        let method = quote! {
            fn close(&self) -> agent_module_trait::AgentModuleResult<()> {
                Ok(())
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }
    if !has_fn(&input, "is_channel") {
        let method = quote! {
            fn is_channel(&self) -> bool {
                false
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }

    if !has_fn(&input, "to_string") {
        let method = quote! {
            fn to_string(&self) -> String {
                "".to_string()
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }



    if !has_fn(&input, "new") {
        let method = quote! {
            fn new() -> Self {
                Self::default()
            }
        };
        input.items.push(syn::parse2(method).unwrap());
    }
    if !has_fn(&input, "new_instance") {
        let method = quote! {
            fn new_instance(&self) -> std::sync::Arc<dyn AgentModule> {
                let handle = std::sync::Arc::new(Self::new());
                handle
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

    let struct_name = &input.ident;

    // 生成代码
    let expanded = quote! {
        #[derive(Default)]
        #input
        impl Drop for #struct_name {
            fn drop(&mut self) {
                let _ = self.close();
            }
        }
    };

    TokenStream::from(expanded)
}