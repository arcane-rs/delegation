//! Code generation of `impl_for!` macro.

mod util;

use std::mem;

use proc_macro2::TokenStream;
use quote::ToTokens;
#[cfg(doc)]
use syn::{Generics, Type};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, token,
    visit_mut::VisitMut as _,
};

use self::util::{ElideLifetimes as _, GenericBinder, GenericsExt as _};
use crate::MacroPath;

/// Definition of `impl_for!` macro expansion.
#[derive(Debug)]
pub(super) struct Definition {
    /// Template to fill real values into.
    template: syn::ItemImpl,

    /// [`Generics`] to override the [`template`] with.
    ///
    /// [`template`]: Definition::template
    generics: syn::Generics,

    /// Trait to override the [`template`] with.
    ///
    /// [`template`]: Definition::template
    trait_path: syn::Path,

    /// [`Type`] to override the [`template`] with.
    ///
    /// [`template`]: Definition::template
    self_ty: syn::Type,

    /// Wrapper around the [`self_ty`] to override the [`template`] with.
    ///
    /// [`self_ty`]: Definition::self_ty
    /// [`template`]: Definition::template
    wrapper_ty: syn::Path,

    /// [`Path`] to the macro definitions.
    ///
    /// [`Path`]: syn::Path
    macro_path: MacroPath,
}

impl Parse for Definition {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let template = input.parse()?;

        _ = input.parse::<token::Impl>()?;
        let mut generics = input.parse::<syn::Generics>()?;

        let trait_path = input.parse()?;

        _ = input.parse::<token::As>()?;
        let wrapper_ty = input.parse()?;

        _ = input.parse::<token::For>()?;
        let self_ty = input.parse()?;

        if let Some(where_clause) = input.parse::<Option<syn::WhereClause>>()? {
            generics.where_clause = Some(where_clause);
        }

        let mut this = Self {
            template,
            generics,
            trait_path,
            self_ty,
            wrapper_ty,
            macro_path: MacroPath::default(),
        };

        this.specify_type();
        this.specify_trait()?;
        this.specify_methods();
        this.specify_generics();

        Ok(this)
    }
}

impl ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.template.to_tokens(tokens);
    }
}

impl Definition {
    /// Indicates whether the target trait is crate-local.
    fn is_local_trait(&self) -> bool {
        let macro_path = &self.macro_path;
        self.wrapper_ty == parse_quote! { #macro_path ::Wrapper }
    }

    /// Replaces template's `Self` [`Type`] with the specified one.
    fn specify_type(&mut self) {
        self.template.self_ty = self.self_ty.clone().into();
    }

    /// Replaces template's trait with the specified one.
    fn specify_trait(&mut self) -> syn::Result<()> {
        self.template.trait_ =
            Some((None, self.trait_path.clone(), token::For::default()));

        if let syn::PathArguments::AngleBracketed(trait_generic_args) = &self
            .trait_path
            .segments
            .last()
            .unwrap_or_else(|| unreachable!("empty trait path"))
            .arguments
        {
            let mut binder = GenericBinder {
                generics: &self
                    .template
                    .generics
                    .bind_arguments(trait_generic_args)?,
            };

            for i in &mut self.template.items {
                if let syn::ImplItem::Fn(m) = i {
                    binder.visit_impl_item_fn_mut(m);
                }
            }
        }

        Ok(())
    }

    /// Replaces templates in method impls.
    fn specify_methods(&mut self) {
        let macro_path = &self.macro_path;
        let wrapper_ty = &self.wrapper_ty;
        let is_external = !self.is_local_trait();

        let mut self_ty = self.self_ty.clone();
        self_ty.elide_lifetimes();

        let mut trait_path = self.trait_path.clone();
        trait_path.elide_lifetimes();

        let wrapper: syn::Type = if is_external {
            parse_quote! {
                <#macro_path::External as #wrapper_ty<#self_ty> >::Wrapper
            }
        } else {
            parse_quote! { #wrapper_ty < #self_ty > }
        };

        // Replace
        // `<<External as WrapperTemplate<Self>>::Bind as TypeOf>::T` with
        // `<<External as Wrapper<Self>>::Bind as TypeOf>::T`.
        let replace_sig_type = |ty: &mut syn::Type| {
            // `<qself as TypeOf>::T`
            //   |    |            |
            // qself  path seg 1   path seg 2 (position)
            let syn::Type::Path(syn::TypePath {
                qself: Some(syn::QSelf { ty: qself_ty, .. }),
                ..
            }) = ty
            else {
                return;
            };

            // `<_ as WrapperTemplate<Self>>::Bind
            //   |    |                       |
            // qself  path seg 1              path seg 2 (position)
            let syn::Type::Path(syn::TypePath {
                qself: Some(syn::QSelf { position, .. }),
                path,
            }) = qself_ty.as_mut()
            else {
                return;
            };

            let prev_path =
                mem::replace(path, parse_quote! { #wrapper_ty <Self> });

            *position = path.segments.len();

            for seg in prev_path.segments.into_iter().skip(1) {
                path.segments.push(seg);
            }
        };

        let replace_selfcall = |block: &mut syn::Block| {
            let Some(syn::Stmt::Expr(
                syn::Expr::Call(syn::ExprCall { func, .. }),
                _,
            )) = block.stmts.first_mut()
            else {
                return;
            };

            let syn::Expr::Path(syn::ExprPath {
                qself: Some(qself), path, ..
            }) = func.as_mut()
            else {
                return;
            };

            // 1. Replace `<WrapperTemplate as TraitTemplate>::trait_method`
            //    with `<WrapperTemplate as Trait>::trait_method`.
            let orig_path = mem::replace(path, trait_path.clone());
            path.segments
                .extend(orig_path.segments.into_iter().skip(qself.position));

            // 2. Replace `<WrapperTemplate as Trait>::trait_method`
            //    with `<Wrapper as Trait>::trait_method`.
            *qself = syn::QSelf {
                lt_token: qself.lt_token,
                ty: wrapper.clone().into(),
                position: trait_path.segments.len(),
                as_token: qself.as_token,
                gt_token: qself.gt_token,
            };
        };

        for i in &mut self.template.items {
            if let syn::ImplItem::Fn(m) = i {
                // Replace only for external traits because local ones
                // uses `Self` without fully qualified paths.
                if is_external {
                    for arg in &mut m.sig.inputs {
                        if let syn::FnArg::Typed(syn::PatType { ty, .. }) = arg
                        {
                            replace_sig_type(ty);
                        }
                    }

                    if let syn::ReturnType::Type(_, ret_ty) = &mut m.sig.output
                    {
                        replace_sig_type(&mut *ret_ty);
                    }
                }

                replace_selfcall(&mut m.block);
            }
        }
    }

    /// Overrides template's [`Generics`] with the specified ones, if new
    /// [`Generics`] are provided.
    fn specify_generics(&mut self) {
        self.template.generics = self.generics.clone();
    }
}
