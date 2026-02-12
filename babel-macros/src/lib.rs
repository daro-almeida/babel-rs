use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, FnArg, ImplItem, ItemImpl, PatType, Type, parse_macro_input};

#[proc_macro_derive(IPC)]
pub fn derive_ipc(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl babel::event::IPCEvent for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn request_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn reply_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn notification_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn protocol(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    let self_ty = &impl_block.self_ty;
    let mut request_handler_registrations = vec![];
    let mut reply_handler_registrations = vec![];
    let mut subscription_registrations = vec![];
    let mut notification_handler_registrations = vec![];

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;

            // check which attribute is present
            let is_request = method
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("request_handler"));
            let is_reply = method
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("reply_handler"));
            let is_notification = method
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("notification_handler"));

            // extract event type from second parameter
            if let Some(event_type) = extract_event_type(&method.sig) {
                let handler_code = quote! {
                    handlers.insert(
                        std::any::TypeId::of::<#event_type>(),
                        Box::new(|protocol: &mut dyn std::any::Any, req: &dyn babel::event::IPCEvent, sender: ProtocolId, handle: &ProtocolHandle| {
                            let protocol = protocol
                                .downcast_mut::<#self_ty>()
                                .expect("Protocol type mismatch");

                            if let Some(typed_req) = req.as_any().downcast_ref::<#event_type>() {
                                protocol.#method_name(typed_req, sender, handle);
                            }
                        }) as babel::event::IPCHandlerFn
                    );
                };

                if is_request {
                    request_handler_registrations.push(handler_code);
                } else if is_reply {
                    reply_handler_registrations.push(handler_code);
                } else if is_notification {
                    let sub_code =
                        quote! {subscriptions.push(std::any::TypeId::of::<#event_type>())};
                    subscription_registrations.push(sub_code);
                    notification_handler_registrations.push(handler_code);
                }
            }
        }
    }

    let expanded = quote! {
        #impl_block

        impl babel::protocol::ProtocolHandlers for #self_ty {
            fn get_request_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::event::IPCHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#request_handler_registrations)*
                handlers
            }

            fn get_reply_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::event::IPCHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#reply_handler_registrations)*
                handlers
            }
            
            fn get_subscriptions(&self) -> std::vec::Vec<std::any::TypeId> {
                use std::vec::Vec;
                let mut subscriptions = Vec::new();
                #(#subscription_registrations)*
                subscriptions
            }

            fn get_notification_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::event::IPCHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#notification_handler_registrations)*
                handlers
            }
        }
    };

    TokenStream::from(expanded)
}

/// extract the event type from the second parameter
fn extract_event_type(sig: &syn::Signature) -> Option<&Type> {
    let inputs: Vec<_> = sig.inputs.iter().collect();

    if let Some(FnArg::Typed(PatType { ty, .. })) = inputs.get(1) {
        if let Type::Reference(type_ref) = &**ty {
            return Some(&*type_ref.elem);
        }
    }

    None
}
