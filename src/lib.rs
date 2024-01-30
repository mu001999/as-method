#![doc = include_str!("../README.md")]

use syn::visit_mut::VisitMut;

/// Call function with the method syntax!
#[proc_macro_attribute]
pub fn as_method(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(proc_macro2::Span::call_site(), "unexpected attr(s)")
            .into_compile_error()
            .into();
    }

    let mut func: syn::ItemFn = match syn::parse(item) {
        Ok(func) => func,
        Err(err) => return err.into_compile_error().into(),
    };

    let mut visitor = ImplTraitReplace(Vec::new());
    let Some(syn::FnArg::Typed(self_ty)) = func.sig.inputs.first_mut() else {
        return syn::Error::new(
            func.sig.paren_token.span.open(),
            "expected at least one parameter",
        )
        .into_compile_error()
        .into();
    };
    visitor.visit_type_mut(&mut *self_ty.ty);

    for type_param in visitor.0 {
        func.sig
            .generics
            .params
            .push(syn::GenericParam::Type(type_param));
    }

    let self_ty = self_ty.ty.clone();

    let vis = &func.vis;
    let name = &func.sig.ident;
    let ret_ty = &func.sig.output;

    let mut arg_tys = Vec::new();
    for input in func.sig.inputs.iter().skip(1) {
        match input {
            syn::FnArg::Typed(pat_type) => arg_tys.push(&*pat_type.ty),
            syn::FnArg::Receiver(receiver) => {
                return syn::Error::new(receiver.self_token.span, "unexpected self receiver")
                    .into_compile_error()
                    .into()
            }
        }
    }

    let args = (1..=arg_tys.len())
        .map(|i| quote::format_ident!("x{}", i))
        .collect::<Vec<_>>();

    let (impl_generics, ty_generics, where_clause) = func.sig.generics.split_for_impl();

    quote::quote! {
        #func

        #[allow(non_camel_case_types)]
        #vis trait #name #ty_generics #where_clause {
            fn #name(self, #(#args: #arg_tys),*) #ret_ty;
        }

        impl #impl_generics #name #ty_generics for #self_ty #where_clause {
            fn #name(self, #(#args: #arg_tys),*) #ret_ty {
                #name(self, #(#args),*)
            }
        }
    }
    .into()
}

struct ImplTraitReplace(Vec<syn::TypeParam>);

impl VisitMut for ImplTraitReplace {
    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        if let syn::Type::ImplTrait(type_impl_trait) = node {
            let ident = quote::format_ident!("AS_METHOD_SELF_T{}", self.0.len());
            self.0.push(syn::TypeParam {
                attrs: Vec::new(),
                ident: ident.clone(),
                colon_token: None,
                bounds: type_impl_trait.bounds.clone(),
                eq_token: None,
                default: None,
            });

            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident,
                arguments: syn::PathArguments::None,
            });

            *node = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments,
                },
            });
        } else {
            syn::visit_mut::visit_type_mut(self, node);
        }
    }
}
