use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_quote, Attribute, FnArg, Generics, Ident, Item, ItemFn, Signature, Type, Visibility,
};

pub trait WorkerFnType {
    type RecvType;
    type OutputType;

    fn attr_name() -> &'static str;
    fn worker_type_name() -> &'static str;
    fn parse_recv_type(sig: &Signature) -> syn::Result<Self::RecvType>;
    fn parse_output_type(sig: &Signature) -> syn::Result<Self::OutputType>;

    fn extract_fn_arg_type(arg: &FnArg) -> syn::Result<Type> {
        let ty = match arg {
            FnArg::Typed(arg) => arg.ty.clone(),

            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    arg,
                    format!(
                        "{} workers can't accept a receiver",
                        Self::worker_type_name()
                    ),
                ));
            }
        };

        Ok(*ty)
    }

    fn assert_no_left_argument<I, T>(rest_inputs: I, expected_len: usize) -> syn::Result<()>
    where
        I: ExactSizeIterator + IntoIterator<Item = T>,
        T: ToTokens,
    {
        // Checking after param parsing may make it a little inefficient
        // but that's a requirement for better error messages in case of receivers
        // `>0` because first one is already consumed.
        if rest_inputs.len() > 0 {
            let params: TokenStream = rest_inputs
                .into_iter()
                .map(|it| it.to_token_stream())
                .collect();
            return Err(syn::Error::new_spanned(
                params,
                format!(
                    "{} worker can accept at most {} argument{}",
                    Self::worker_type_name(),
                    expected_len,
                    if expected_len > 1 { "s" } else { "" }
                ),
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct WorkerFn<F>
where
    F: WorkerFnType + 'static,
{
    pub recv_type: F::RecvType,
    pub output_type: F::OutputType,
    pub generics: Generics,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub worker_name: Option<Ident>,
    pub is_async: bool,

    pub func: ItemFn,
}

impl<F> Parse for WorkerFn<F>
where
    F: WorkerFnType + 'static,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed: Item = input.parse()?;

        let func = match parsed {
            Item::Fn(m) => m,

            item => {
                return Err(syn::Error::new_spanned(
                    item,
                    format!(
                        "`{}` attribute can only be applied to functions",
                        F::attr_name()
                    ),
                ))
            }
        };

        let ItemFn {
            attrs, vis, sig, ..
        } = func.clone();

        if sig.generics.lifetimes().next().is_some() {
            return Err(syn::Error::new_spanned(
                sig.generics,
                format!(
                    "{} workers can't have generic lifetime parameters",
                    F::worker_type_name()
                ),
            ));
        }

        if sig.constness.is_some() {
            return Err(syn::Error::new_spanned(
                sig.constness,
                format!("const functions can't be {} workers", F::worker_type_name()),
            ));
        }

        if sig.abi.is_some() {
            return Err(syn::Error::new_spanned(
                sig.abi,
                format!(
                    "extern functions can't be {} workers",
                    F::worker_type_name()
                ),
            ));
        }
        let recv_type = F::parse_recv_type(&sig)?;
        let output_type = F::parse_output_type(&sig)?;

        let is_async = sig.asyncness.is_some();

        Ok(Self {
            recv_type,
            output_type,
            generics: sig.generics,
            is_async,
            vis,
            attrs,
            name: sig.ident,
            worker_name: None,
            func,
        })
    }
}

impl<F> WorkerFn<F>
where
    F: WorkerFnType + 'static,
{
    /// Filters attributes that should be copied to worker definition.
    pub fn filter_attrs_for_worker_struct(&self) -> Vec<Attribute> {
        self.attrs
            .iter()
            .filter_map(|m| {
                m.path()
                    .get_ident()
                    .and_then(|ident| match ident.to_string().as_str() {
                        "doc" | "allow" => Some(m.clone()),
                        _ => None,
                    })
            })
            .collect()
    }

    /// Filters attributes that should be copied to the worker impl block.
    pub fn filter_attrs_for_worker_impl(&self) -> Vec<Attribute> {
        self.attrs
            .iter()
            .filter_map(|m| {
                m.path()
                    .get_ident()
                    .and_then(|ident| match ident.to_string().as_str() {
                        "allow" => Some(m.clone()),
                        _ => None,
                    })
            })
            .collect()
    }

    pub fn phantom_generics(&self) -> Punctuated<Ident, Comma> {
        self.generics
            .type_params()
            .map(|ty_param| ty_param.ident.clone()) // create a new Punctuated sequence without any type bounds
            .collect::<Punctuated<_, Comma>>()
    }

    pub fn merge_worker_name(&mut self, name: WorkerName) -> syn::Result<()> {
        if let Some(ref m) = name.worker_name {
            if m == &self.name {
                return Err(syn::Error::new_spanned(
                    m,
                    format!(
                        "the {} must not have the same name as the function",
                        F::worker_type_name()
                    ),
                ));
            }
        }

        self.worker_name = name.worker_name;

        Ok(())
    }

    pub fn inner_fn_ident(&self) -> Ident {
        if self.worker_name.is_some() {
            self.name.clone()
        } else {
            Ident::new("inner", Span::mixed_site())
        }
    }

    pub fn worker_name(&self) -> Ident {
        self.worker_name
            .clone()
            .unwrap_or_else(|| self.name.clone())
    }

    pub fn print_inner_fn(&self) -> ItemFn {
        let mut func = self.func.clone();
        func.sig.ident = self.inner_fn_ident();

        func.vis = Visibility::Inherited;

        func
    }

    // Sometimes users use gloo_worker directly, sometimes it is imported via gloo.
    // We inspect Cargo.toml to find out.
    pub fn worker_crate_name() -> syn::Path {
        if let Ok(m) = crate_name("gloo-worker") {
            return match m {
                FoundCrate::Itself => parse_quote!(gloo_worker),
                FoundCrate::Name(m) => {
                    let crate_name = Ident::new(&m, Span::mixed_site());

                    parse_quote!(#crate_name)
                }
            };
        }

        if let Ok(m) = crate_name("gloo") {
            return match m {
                FoundCrate::Itself => parse_quote!(gloo::worker),
                FoundCrate::Name(m) => {
                    let crate_name = Ident::new(&m, Span::mixed_site());

                    parse_quote!(#crate_name::worker)
                }
            };
        }

        panic!("failed to find crate!")
    }
}

pub struct WorkerName {
    worker_name: Option<Ident>,
}

impl Parse for WorkerName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { worker_name: None });
        }

        let worker_name = input.parse()?;

        Ok(Self {
            worker_name: Some(worker_name),
        })
    }
}
