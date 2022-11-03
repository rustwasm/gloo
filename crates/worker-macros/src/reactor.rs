use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ReturnType, Signature, Type};

use crate::worker_fn::{WorkerFn, WorkerFnType, WorkerName};

pub struct ReactorFn {}

impl WorkerFnType for ReactorFn {
    type OutputType = Type;
    type RecvType = Type;

    fn attr_name() -> &'static str {
        "reactor"
    }

    fn worker_type_name() -> &'static str {
        "reactor"
    }

    fn parse_recv_type(sig: &Signature) -> syn::Result<Self::RecvType> {
        let mut inputs = sig.inputs.iter();
        let arg = inputs
            .next()
            .ok_or_else(|| syn::Error::new_spanned(&sig.ident, "expected 1 argument"))?;

        let ty = Self::extract_fn_arg_type(arg)?;

        Self::assert_no_left_argument(inputs, 1)?;

        Ok(ty)
    }

    fn parse_output_type(sig: &Signature) -> syn::Result<Self::OutputType> {
        let ty = match &sig.output {
            ReturnType::Default => {
                return Err(syn::Error::new_spanned(
                    &sig.ident,
                    "expected a stream as the return type",
                ));
            }
            ReturnType::Type(_, ty) => *ty.clone(),
        };

        Ok(ty)
    }
}

pub fn reactor_impl(
    name: WorkerName,
    mut worker_fn: WorkerFn<ReactorFn>,
) -> syn::Result<TokenStream> {
    worker_fn.merge_worker_name(name)?;

    if worker_fn.is_async {
        return Err(syn::Error::new_spanned(
            &worker_fn.name,
            "reactor workers cannot be asynchronous",
        ));
    }

    let struct_attrs = worker_fn.filter_attrs_for_worker_struct();
    let reactor_impl_attrs = worker_fn.filter_attrs_for_worker_impl();
    let phantom_generics = worker_fn.phantom_generics();
    let reactor_name = worker_fn.worker_name();
    let fn_name = worker_fn.inner_fn_ident();
    let inner_fn = worker_fn.print_inner_fn();

    let WorkerFn {
        recv_type,
        output_type,
        generics,
        vis,
        ..
    } = worker_fn;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fn_generics = ty_generics.as_turbofish();

    let inputs_ident = Ident::new("inputs", Span::mixed_site());

    let fn_call = quote! { #fn_name #fn_generics (#inputs_ident) };
    let crate_name = WorkerFn::<ReactorFn>::worker_crate_name();

    let quoted = quote! {
        #(#struct_attrs)*
        #[allow(unused_parens)]
        #vis struct #reactor_name #generics #where_clause {
            _marker: ::std::marker::PhantomData<(#phantom_generics)>,
        }

        // we cannot disable any lints here because it will be applied to the function body
        // as well.
        #(#reactor_impl_attrs)*
        impl #impl_generics ::#crate_name::reactor::Reactor for #reactor_name #ty_generics #where_clause {
            type InputStream = #recv_type;
            type OutputStream = #output_type;

            fn create(#inputs_ident: Self::InputStream) -> Self::OutputStream {
                #inner_fn
                #fn_call
            }
        }

        impl #impl_generics ::#crate_name::Registrable for #reactor_name #ty_generics #where_clause {
            type Registrar = ::#crate_name::reactor::ReactorRegistrar<Self>;

            fn registrar() -> Self::Registrar {
                ::#crate_name::reactor::ReactorRegistrar::<Self>::new()
            }
        }

        impl #impl_generics ::#crate_name::Spawnable for #reactor_name #ty_generics #where_clause {
            type Spawner = ::#crate_name::reactor::ReactorSpawner<Self>;

            fn spawner() -> Self::Spawner {
                ::#crate_name::reactor::ReactorSpawner::<Self>::new()
            }
        }
    };

    Ok(quoted)
}
