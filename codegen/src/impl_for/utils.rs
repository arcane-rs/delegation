//! Utils for `impl_for!` macro expansion.

use std::collections::HashMap;

use quote::ToTokens as _;
use syn::{
    parse_quote,
    visit_mut::{self, VisitMut},
};

/// Extension of [`syn::Generics`].
pub(super) trait GenericsExt {
    /// Binds each [`GenPar`]ameter to its corresponding [`GenArg`]ument.
    fn bind_arguments(
        &mut self,
        args: &syn::AngleBracketedGenericArguments,
    ) -> syn::Result<HashMap<GenPar, GenArg>>;
}

impl GenericsExt for syn::Generics {
    fn bind_arguments(
        &mut self,
        args: &syn::AngleBracketedGenericArguments,
    ) -> syn::Result<HashMap<GenPar, GenArg>> {
        let mut generics = HashMap::new();

        if self.params.len() != args.args.len() {
            return Err(syn::Error::new_spanned(
                args,
                "Wrong number of generic arguments",
            ));
        }

        for (param, arg) in self.params.iter().zip(args.args.iter()) {
            let param = GenPar::from(param);
            let arg = GenArg::try_from(arg).map_err(|_| {
                syn::Error::new_spanned(
                    arg,
                    "Generic argument must be a type, constant \
                     or a lifetime",
                )
            })?;

            let _ = generics.insert(param, arg);
        }

        Ok(generics)
    }
}

/// Binder for replacing [`GenPar`]ameters with their corresponding
/// [`GenArg`]uments.
pub(super) struct GenericBinder<'g> {
    /// Map of [`GenPar`]ameters to their corresponding [`GenArg`]uments.
    pub(super) generics: &'g HashMap<GenPar, GenArg>,
}

impl VisitMut for GenericBinder<'_> {
    fn visit_lifetime_mut(&mut self, lt: &mut syn::Lifetime) {
        if let Some(GenArg::Lifetime(l)) =
            self.generics.get(&GenPar::from(&*lt))
        {
            *lt = l.clone();
        } else {
            visit_mut::visit_lifetime_mut(self, lt);
        };
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let val = GenPar::try_from(&*block)
            .ok()
            .and_then(|ty| self.generics.get(&ty));

        match val {
            Some(GenArg::Type(t)) => {
                block.stmts = vec![parse_quote! { #t }];
            }
            Some(GenArg::Const(b)) => *block = b.clone(),
            Some(GenArg::Lifetime(_)) | None => {
                visit_mut::visit_block_mut(self, block)
            }
        }
    }

    fn visit_type_mut(&mut self, arg: &mut syn::Type) {
        if let Some(GenArg::Type(t)) = GenPar::try_from(&*arg)
            .ok()
            .and_then(|ty| self.generics.get(&ty))
        {
            *arg = t.clone();
        } else {
            visit_mut::visit_type_mut(self, arg);
        };
    }
}

/// Generic type parameter to be replaced with [`GenArg`]ument.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) enum GenPar {
    Lifetime(syn::Ident),
    Type(syn::Ident),
    Const(syn::Ident),
}

impl<'a> From<&'a syn::GenericParam> for GenPar {
    fn from(param: &'a syn::GenericParam) -> Self {
        match param {
            syn::GenericParam::Lifetime(lt) => {
                Self::Lifetime(lt.lifetime.ident.clone())
            }
            syn::GenericParam::Type(ty) => Self::Type(ty.ident.clone()),
            syn::GenericParam::Const(c) => Self::Const(c.ident.clone()),
        }
    }
}

impl<'a> From<&'a syn::Lifetime> for GenPar {
    fn from(lt: &'a syn::Lifetime) -> Self {
        Self::Lifetime(lt.ident.clone())
    }
}

impl<'a> TryFrom<&'a syn::Type> for GenPar {
    type Error = ();

    fn try_from(ty: &'a syn::Type) -> Result<Self, Self::Error> {
        syn::parse2::<syn::Ident>(ty.to_token_stream())
            .map(Self::Type)
            .map_err(|_| ())
    }
}

impl<'a> TryFrom<&'a syn::Block> for GenPar {
    type Error = ();

    fn try_from(block: &'a syn::Block) -> Result<Self, Self::Error> {
        block
            .stmts
            .first()
            .and_then(|stmt| {
                syn::parse2::<syn::Ident>(stmt.to_token_stream()).ok()
            })
            .map(Self::Const)
            .ok_or(())
    }
}

/// Generic argument to replace a [`GenPar`]ameter.
#[derive(Clone, Debug)]
pub(super) enum GenArg {
    Lifetime(syn::Lifetime),
    Type(syn::Type),
    Const(syn::Block),
}

impl<'a> TryFrom<&'a syn::GenericArgument> for GenArg {
    type Error = syn::Error;

    fn try_from(arg: &'a syn::GenericArgument) -> Result<Self, Self::Error> {
        use syn::GenericArgument as A;

        Ok(match arg {
            A::Lifetime(lt) => Self::Lifetime(lt.clone()),
            A::Type(ty) => Self::Type(ty.clone()),
            A::Const(syn::Expr::Block(syn::ExprBlock {
                attrs: _,
                label: _,
                block,
            })) => Self::Const(block.clone()),
            A::Const(_)
            | A::Constraint(_)
            | A::AssocType(_)
            | A::AssocConst(_) => {
                return Err(syn::Error::new_spanned(
                    arg,
                    "Unsupported generic argument",
                ));
            }
            // TODO: Use `non_exhaustive_omitted_patterns`, once stabilized.
            //       https://github.com/rust-lang/rust/issues/89554
            // #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
            arg => panic!("{arg:#?} not covered"),
        })
    }
}

/// Trait for eliding [`Lifetime`]s.
///
/// [`Lifetime`]: struct@syn::Lifetime
pub(super) trait ElideLifetimes {
    /// Replaces all [`Lifetime`]s in this type with `'_`.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    fn elide_lifetimes(&mut self);
}

impl ElideLifetimes for syn::Type {
    fn elide_lifetimes(&mut self) {
        ReplaceLifetimes {
            replace_with: &parse_quote! { '_ },
        }
        .visit_type_mut(self);
    }
}

impl ElideLifetimes for syn::Path {
    fn elide_lifetimes(&mut self) {
        ReplaceLifetimes {
            replace_with: &parse_quote! { '_ },
        }
        .visit_path_mut(self);
    }
}

/// Replacer of the [`Lifetime`]s with the `replace_with` one.
///
/// [`Lifetime`]: struct@syn::Lifetime
struct ReplaceLifetimes<'r> {
    /// [`Lifetime`] to replace with.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    replace_with: &'r syn::Lifetime,
}

impl VisitMut for ReplaceLifetimes<'_> {
    fn visit_lifetime_mut(&mut self, l: &mut syn::Lifetime) {
        *l = self.replace_with.clone();
        visit_mut::visit_lifetime_mut(self, l);
    }
}
