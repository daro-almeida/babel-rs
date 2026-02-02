use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ImplItem, ItemImpl, PatType, Type};

#[proc_macro_derive(Request)]
pub fn derive_request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl Request for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn protocol(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    let self_ty = &impl_block.self_ty;
    let mut handler_registrations = vec![];

    // iterate through methods
    for item in &impl_block.items {
        match item {
            ImplItem::Fn(method) => {
                let method_name = &method.sig.ident;

                // check if second parameter exists and is a reference
                let mut inputs = method.sig.inputs.iter();
                inputs.next(); // skip &mut self

                if let Some(FnArg::Typed(PatType { ty, .. })) = inputs.next() {
                    if let Type::Reference(type_ref) = &**ty {
                        let req_type = &type_ref.elem;

                        handler_registrations.push(quote! {
                            handlers.insert(
                                std::any::TypeId::of::<#req_type>(),
                                Box::new(|protocol: &mut dyn std::any::Any, req: &dyn Request, sender: ProtocolId, handle: &ProtocolHandle| {
                                    let protocol = protocol
                                        .downcast_mut::<#self_ty>()
                                        .expect("Protocol type mismatch");

                                    if let Some(typed_req) = req.as_any().downcast_ref::<#req_type>() {
                                        protocol.#method_name(typed_req, sender, handle);
                                    }
                                }) as RequestHandlerFn
                            );
                        });
                    }
                }
            }
            _ => {}
        }
    }

    let expanded = quote! {
        #impl_block

        impl ProtocolHandlers for #self_ty {
            fn get_request_handlers() -> std::collections::HashMap<std::any::TypeId, RequestHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#handler_registrations)*
                handlers
            }
        }
    };

    TokenStream::from(expanded)
}