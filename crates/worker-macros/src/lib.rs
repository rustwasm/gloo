use proc_macro::TokenStream;
use syn::parse_macro_input;

mod oneshot;
mod reactor;
mod worker_fn;

use oneshot::{oneshot_impl, OneshotFn};
use reactor::{reactor_impl, ReactorFn};
use worker_fn::{WorkerFn, WorkerName};

#[proc_macro_attribute]
pub fn reactor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as WorkerFn<ReactorFn>);
    let attr = parse_macro_input!(attr as WorkerName);

    reactor_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn oneshot(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as WorkerFn<OneshotFn>);
    let attr = parse_macro_input!(attr as WorkerName);

    oneshot_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
