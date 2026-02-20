use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ImplItem, ItemImpl, PatType, Type};

#[proc_macro_derive(Ipc)]
pub fn derive_ipc(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote! {
        impl babel::internal::ipc::Ipc for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    }.into()
}

#[proc_macro_derive(Notification)]
pub fn derive_notifiaction(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl Notification for #name {}
    }.into()
}

#[proc_macro_derive(Message, attributes(message_id))]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let message_id = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("message_id"))
        .expect("Missing attribute message_id for Message")
        .parse_args::<syn::LitInt>()
        .expect("Invalid message_id type. Expected u16")
        .base10_parse::<u16>()
        .expect("Invalid message_id type. Expected u16");
    
    quote! {
        impl babel::internal::message::Message for #name {
            const ID: babel::internal::message::MessageId = babel::internal::message::MessageId(#message_id);
        }
    }.into()
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
pub fn message_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn shutdown_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
    let mut message_handler_registrations = vec![];
    let mut shutdown_handler = quote! { None };

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;

            for attr in &method.attrs {
                if let Some(ident) = attr.path().get_ident() {
                    let handler = ident.to_string();
                    match handler.as_str() {
                        "request_handler" | "reply_handler" | "notification_handler" => {
                            if let Some(event_type) = extract_event_type(&method.sig) {
                                let handler_code = quote! {
                                    handlers.insert(
                                        std::any::TypeId::of::<#event_type>(),
                                        Box::new(|protocol: &mut dyn std::any::Any, ipc: &dyn babel::internal::ipc::Ipc, sender: ProtocolId, handle: ProtocolHandle| {
                                            let protocol = protocol
                                                .downcast_mut::<#self_ty>()
                                                .expect("Protocol type mismatch");

                                            if let Some(typed_ipc) = ipc.as_any().downcast_ref::<#event_type>() {
                                                protocol.#method_name(typed_ipc, sender, handle);
                                            }
                                        }) as babel::internal::event::IpcHandlerFn
                                    );
                                };

                                match handler.as_str() {
                                    "request_handler" => {
                                        request_handler_registrations.push(handler_code)
                                    }
                                    "reply_handler" => {
                                        reply_handler_registrations.push(handler_code)
                                    }
                                    "notification_handler" => {
                                        let sub_code = quote! {subscriptions.push(std::any::TypeId::of::<#event_type>());};
                                        subscription_registrations.push(sub_code);
                                        notification_handler_registrations.push(handler_code)
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                panic!(
                                    "Invalid handler '{}' for Ipc event attributes {}. Expected signature: (&mut self, &Ipc, ProtocolId, ProtocolHandle)",
                                    method_name,
                                    method
                                        .attrs
                                        .iter()
                                        .map(|attr| attr.path().get_ident().unwrap().to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                );
                            }
                        }
                        "message_handler" => {
                            if let Some(event_type) = extract_event_type(&method.sig) {
                                let handler_code = quote! {
                                    handlers.insert(
                                        std::any::TypeId::of::<#event_type>(),
                                        //&mut dyn Any, &dyn AnyMessage, SocketAddr, ProtocolId, ProtocolHandle
                                        Box::new(|protocol: &mut dyn std::any::Any, message: &dyn babel::internal::message::AnyMessage, from: SocketAddr, source: ProtocolId, handle: ProtocolHandle| {
                                            let protocol = protocol
                                                .downcast_mut::<#self_ty>()
                                                .expect("Protocol type mismatch");

                                            if let Some(typed_message) = message.as_any().downcast_ref::<#event_type>() {
                                                protocol.#method_name(typed_message, sender, handle);
                                            }
                                        }) as babel::internal::event::MessageHandlerFn
                                    );
                                };

                                message_handler_registrations.push(handler_code)
                            }
                        }
                        "shutdown_handler" => {
                            let handler_code = quote! {
                            Some(Box::new( | protocol: & mut dyn std::any::Any, handle: ProtocolHandle | {
                                let protocol = protocol
                                .downcast_mut::< # self_ty> ()
                                .expect("Protocol type mismatch");

                                protocol.# method_name(handle);
                                }) as babel::internal::event::ShutdownHandlerFn)
                            };
                            shutdown_handler = handler_code;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let expanded = quote! {
        #impl_block

        impl babel::protocol::ProtocolHandlers for #self_ty {
            fn get_request_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::internal::event::IpcHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#request_handler_registrations)*
                handlers
            }

            fn get_reply_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::internal::event::IpcHandlerFn> {
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

            fn get_notification_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::internal::event::IpcHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#notification_handler_registrations)*
                handlers
            }

            fn get_message_handlers(&self) -> std::collections::HashMap<std::any::TypeId, babel::internal::event::MessageHandlerFn> {
                use std::collections::HashMap;
                let mut handlers = HashMap::new();
                #(#message_handler_registrations)*
                handlers
            }

            fn get_shutdown_handler(&self) -> Option<babel::internal::event::ShutdownHandlerFn> {
                #shutdown_handler
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
