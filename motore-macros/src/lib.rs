#![doc(
    html_logo_url = "https://github.com/cloudwego/motore/raw/main/.github/assets/logo.png?sanitize=true"
)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, ItemImpl, PatType, Type};

/// This macro can help you to write a `Service` in a more efficient way.
///
/// # Example
///
/// ```rust
/// use motore::{service, Service};
///
/// pub struct S<I> {
///     inner: I,
/// }
///
/// #[service]
/// impl<Cx, Req, I> Service<Cx, Req> for S<I>
/// where
///     Req: Send + 'static,
///     I: Send + 'static + Service<Cx, Req> + Sync,
///     Cx: Send + 'static,
/// {
///     async fn call(&self, cx: &mut Cx, req: Req) -> Result<I::Response, I::Error> {
///         self.inner.call(cx, req).await
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn service(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemImpl);

    if let Err(err) = expand(&mut item) {
        return err.into_compile_error().into();
    }

    TokenStream::from(quote!(#item))
}

fn expand(item: &mut ItemImpl) -> Result<(), syn::Error> {
    let generic_params: &syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma> =
        &item.generics.params;
    let call_method = item
        .items
        .iter_mut()
        .find_map(|i| match i {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        })
        .expect("`call` method is required");

    let sig = &mut call_method.sig;

    if sig.asyncness.is_none() {
        return Err(syn::Error::new(
            call_method.span(),
            "call method should be async",
        ));
    }

    if sig.inputs.len() != 3 {
        return Err(syn::Error::new(
            call_method.span(),
            "`call` method expects 3 arg",
        ));
    }

    let cx_type = match &mut sig.inputs[1] {
        syn::FnArg::Typed(PatType { ty, .. }) => match &mut **ty {
            Type::Reference(ty) if ty.mutability.is_some() => (*ty.elem).clone(),
            _ => {
                return Err(syn::Error::new(
                    sig.inputs[1].span(),
                    "context type not match",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                sig.inputs[1].span(),
                "context type not match",
            ))
        }
    };

    let _cx_is_generic = generic_params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(t),
            _ => None,
        })
        .any(|t| matches!(&cx_type, Type::Path(p) if p.path.segments.len() == 1 && p.path.segments[0].ident == t.ident));

    let (res_ty, err_ty) = match &sig.output {
        syn::ReturnType::Type(_, ty) => match &**ty {
            Type::Path(p) => {
                let p = &p.path.segments[0];
                match &p.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        (args.args[0].clone(), args.args[1].clone())
                    }
                    _ => {
                        return Err(syn::Error::new(
                            sig.output.span(),
                            "the return type of `call` should be `Result`",
                        ))
                    }
                }
            }
            _ => {
                return Err(syn::Error::new(
                    sig.output.span(),
                    "the return type of `call` should be `Result`",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                sig.output.span(),
                "the return type of `call` should be `Result`",
            ))
        }
    };
    sig.asyncness = None;
    // sig.generics.where_clause = Some(parse_quote!(where 's: 'cx));
    #[cfg(feature = "service_send")]
    {
        sig.output = parse_quote!(-> impl ::std::future::Future<Output = Result<Self::Response, Self::Error>> + Send);
    }
    #[cfg(not(feature = "service_send"))]
    {
        sig.output = parse_quote!(-> impl ::std::future::Future<Output = Result<Self::Response, Self::Error>>);
    }
    sig.inputs[0] = parse_quote!(&self);
    let old_stmts = &call_method.block.stmts;
    call_method.block.stmts = vec![parse_quote!( { async move { #(#old_stmts)* }})];

    item.items.push(parse_quote!(
        type Response = #res_ty;
    ));

    item.items.push(parse_quote!(
        type Error = #err_ty;
    ));

    Ok(())
}
