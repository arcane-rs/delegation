//! `#[delegate]` macro expansion on types (structs or enums).

use std::iter;

use itertools::Itertools as _;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
#[cfg(doc)]
use syn::{Attribute, Generics, Index, Path, Type, WhereClause};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    token,
};

use crate::{
    MacroPath,
    util::{GenericsExt as _, WhereClauseExt as _},
};

/// Arguments for `#[delegate]` macro expansion on types (structs or enums).
struct Args {
    /// `derive` attribute argument, specifying derived traits.
    derive: Punctuated<DeriveTrait, token::Comma>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut this = Self { derive: Punctuated::new() };

        if input.is_empty() {
            return Ok(this);
        }

        input
            .parse::<syn::Ident>()
            .ok()
            .and_then(|i| (i == "derive").then_some(()))
            .ok_or_else(|| {
                syn::Error::new(input.span(), "unexpected attribute argument")
            })?;
        let args;
        _ = syn::parenthesized!(args in input);

        this.derive = Punctuated::parse_terminated(&args)?;

        Ok(this)
    }
}

/// Arguments for `#[delegate]` attribute on structs fields or enums variants.
struct InnerArgs {
    /// `as` attribute argument, specifying the external type this field/variant
    /// is referencing to.
    r#as: Option<syn::Type>,
}

impl InnerArgs {
    /// Creates new [`InnerArgs`] from the provided [`Attribute`]s and removes
    /// the corresponding attributes from the provided [`Attribute`]s.
    fn from_attrs(
        attrs: &mut Vec<syn::Attribute>,
    ) -> syn::Result<Option<Self>> {
        let args = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("delegate"))
            .map(syn::Attribute::parse_args::<Self>)
            .at_most_one()
            .map_err(|_err| {
                syn::Error::new(
                    Span::call_site(),
                    "expected exactly one `#[delegate(..)]` attribute",
                )
            })?
            .transpose()?;

        attrs.retain(|attr| !attr.path().is_ident("delegate"));

        Ok(args)
    }
}

impl Parse for InnerArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut this = Self { r#as: None };

        if input.is_empty() {
            return Ok(this);
        }

        _ = input.parse::<token::As>().map_err(|e| {
            syn::Error::new(e.span(), "unexpected attribute argument")
        })?;
        _ = input.parse::<token::Eq>()?;
        this.r#as = Some(input.parse()?);

        Ok(this)
    }
}

/// Definition of `#[delegate]` macro expansion on types (structs or enums).
#[derive(Debug)]
pub(crate) struct Definition {
    /// Type identifier of this [`Definition`].
    ident: syn::Ident,

    /// [`Generics`] of this [`Definition`].
    generics: syn::Generics,

    /// Delegated enum [`Variant`]s or a single struct [`Field`].
    delegated: DelegatedTypes,

    /// Traits to derive.
    derived_traits: Vec<DeriveTrait>,

    /// Item of this [`Definition`].
    item: Item,

    /// Path to the macro definitions.
    macro_path: MacroPath,
}

impl ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
        self.impl_convert().to_tokens(tokens);
        self.derive_traits().to_tokens(tokens);
    }
}

impl Definition {
    /// Parses a [`Definition`] from the provided [`syn::ItemEnum`].
    pub(crate) fn parse_enum(
        mut item: syn::ItemEnum,
        args: TokenStream,
    ) -> syn::Result<Self> {
        let args = syn::parse2::<Args>(args)?;

        Ok(Self {
            ident: item.ident.clone(),
            generics: item.generics.clone(),
            delegated: DelegatedTypes::Variants(
                item.variants
                    .iter_mut()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            ),
            derived_traits: args.derive.into_iter().collect(),
            item: Item::Enum(item),
            macro_path: MacroPath::default(),
        })
    }

    /// Parses a [`Definition`] from the provided [`syn::ItemStruct`].
    pub(crate) fn parse_struct(
        mut item: syn::ItemStruct,
        args: TokenStream,
    ) -> syn::Result<Self> {
        let args = syn::parse2::<Args>(args)?;

        Ok(Self {
            ident: item.ident.clone(),
            generics: item.generics.clone(),
            delegated: DelegatedTypes::Field((&mut item.fields).try_into()?),
            derived_traits: args.derive.into_iter().collect(),
            item: Item::Struct(item),
            macro_path: MacroPath::default(),
        })
    }

    /// Implements the `Convert` trait for the delegated type.
    fn impl_convert(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let ident = &self.ident;

        let (impl_gens, ty_gens, where_clause) = self.generics.split_for_impl();

        let lifetime = parse_quote! { '__delegate };
        let either_owned =
            self.generate_either(self.delegated.types(), None, false);
        let either_ref = self.generate_either(
            self.delegated.types(),
            Some(&lifetime),
            false,
        );
        let either_ref_mut =
            self.generate_either(self.delegated.types(), Some(&lifetime), true);

        let mut either_where_clause: syn::WhereClause = parse_quote! { where };
        either_where_clause.predicates.extend(self.delegated.types().map(
            |ty| -> syn::WherePredicate {
                parse_quote! { #ty: #lifetime }
            },
        ));

        let (convert_owned, convert_ref, convert_ref_mut) = match &self
            .delegated
        {
            DelegatedTypes::Variants(variants) => {
                let arms = self.generate_match(variants);
                (arms.clone(), arms.clone(), arms)
            }
            DelegatedTypes::Field(field) => {
                let l = token::And::default();
                let m = token::Mut::default();
                (
                    self.generate_field_convert_impl(field, None, None),
                    self.generate_field_convert_impl(field, Some(l), None),
                    self.generate_field_convert_impl(field, Some(l), Some(m)),
                )
            }
        };

        quote! {
            #[automatically_derived]
            impl #impl_gens #macro_path::Convert for #ident #ty_gens
                 #where_clause
            {
                type Owned = #either_owned;
                type Ref<#lifetime> = #either_ref #either_where_clause;
                type RefMut<#lifetime> = #either_ref_mut #either_where_clause;

                fn convert_owned(
                    self
                ) -> <Self as #macro_path::Convert>::Owned {
                    #convert_owned
                }

                fn convert_ref(
                    &self
                ) -> <Self as #macro_path::Convert>::Ref<'_> {
                    #convert_ref
                }

                fn convert_ref_mut(
                    &mut self
                ) -> <Self as #macro_path::Convert>::RefMut<'_> {
                    #convert_ref_mut
                }
            }
        }
    }

    /// Derives traits specified in the `derive(..)` attribute argument for this
    /// type.
    fn derive_traits(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let ident = &self.ident;
        let (_, ty_gens, _) = self.generics.split_for_impl();

        self.derived_traits
            .iter()
            .map(|p| {
                let macro_rules_path = p.macro_rules_path();
                let trait_path = &p.path;
                let gens = self
                    .generics
                    .merge(p.generics.as_ref())
                    .merge_where_clause(p.where_clause.as_ref());
                let (impl_gens, _, where_clause) = gens.split_for_impl();

                let wrapper = p.wrapper_ty.as_ref().map_or_else(
                    || quote! { #macro_path ::Wrapper },
                    ToTokens::to_token_stream,
                );

                quote! {
                    #macro_rules_path!(
                        impl #impl_gens #trait_path as #wrapper
                        for #ident #ty_gens
                        #where_clause
                    );
                }
            })
            .collect()
    }

    /// Generates an `Either` type like
    /// `Either<Ty1, <... Either<TyN, Void>> ...>` with optionally added maybe
    /// mutable reference before each `TyN`.
    fn generate_either<'ty, I>(
        &self,
        types: I,
        lifetime: Option<&'ty syn::Lifetime>,
        is_mutable: bool,
    ) -> TokenStream
    where
        I: IntoIterator<Item = &'ty syn::Type>,
    {
        let mut tokens = TokenStream::new();

        let macro_path = &self.macro_path;
        let either = quote! { #macro_path::Either };
        let void = quote! { #macro_path::Void };

        let mut i = 0;
        for ty in types {
            i += 1;

            either.to_tokens(&mut tokens);
            token::Lt::default().to_tokens(&mut tokens);

            if let Some(lifetime) = lifetime {
                token::And::default().to_tokens(&mut tokens);
                lifetime.to_tokens(&mut tokens);

                if is_mutable {
                    token::Mut::default().to_tokens(&mut tokens);
                }
            }

            ty.to_tokens(&mut tokens);

            token::Comma::default().to_tokens(&mut tokens);
        }

        void.to_tokens(&mut tokens);

        for _ in 0..i {
            token::Gt::default().to_tokens(&mut tokens);
        }

        tokens
    }

    /// Generates a `match` expression that converts delegated types into the
    /// `Either` generated by the [`Self::generate_either()`] method.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match self {
    ///     Self::Variant1(v) => Either::Left(v),
    ///     Self::Variant2 { field } =>  Either::Right(
    ///         Either::Left(field)
    ///     ),
    ///     // ...
    ///     Self::VariantN(v) => Either::Right(v),
    /// }
    /// ```
    fn generate_match(&self, variants: impl AsRef<[Variant]>) -> TokenStream {
        fn sequence(
            tokens: &mut TokenStream,
            count: usize,
            expr: &TokenStream,
            macro_path: &MacroPath,
        ) {
            let ident = if count == 0 { "Left" } else { "Right" };

            quote! { #macro_path::Either }.to_tokens(tokens);
            token::PathSep::default().to_tokens(tokens);
            syn::Ident::new(ident, Span::call_site()).to_tokens(tokens);
            token::Paren::default().surround(tokens, |toks| {
                if count == 0 {
                    expr.to_tokens(toks);
                } else {
                    sequence(toks, count - 1, expr, macro_path);
                }
            });
        }

        let mut tokens = TokenStream::new();

        token::Match::default().to_tokens(&mut tokens);
        token::SelfValue::default().to_tokens(&mut tokens);
        token::Brace::default().surround(&mut tokens, |toks| {
            for (i, variant) in variants.as_ref().iter().enumerate() {
                token::SelfType::default().to_tokens(toks);
                token::PathSep::default().to_tokens(toks);
                variant.ident.to_tokens(toks);

                let val = if let Some(ident) = &variant.field_ident {
                    token::Brace::default().surround(toks, |t| {
                        ident.to_tokens(t);
                    });
                    ident.clone()
                } else {
                    let ident = syn::Ident::new("v", Span::call_site());
                    token::Paren::default().surround(toks, |t| {
                        ident.to_tokens(t);
                    });
                    ident
                };

                let expr = variant.wrapper_ty.as_ref().map_or_else(
                    || val.to_token_stream(),
                    |as_ty| {
                        let ty = &variant.ty;
                        quote! {
                            <#as_ty as ::core::convert::From< #ty >>::from(#val)
                        }
                    },
                );

                token::FatArrow::default().to_tokens(toks);
                token::Brace::default().surround(toks, |t| {
                    sequence(t, i, &expr, &self.macro_path);
                });
            }
        });

        tokens
    }

    /// Generates `convert_*` method body for the provided [`Field`].
    fn generate_field_convert_impl(
        &self,
        field: &Field,
        ref_tok: Option<token::And>,
        mut_tok: Option<token::Mut>,
    ) -> TokenStream {
        let macro_path = &self.macro_path;
        let ident = field.ident();

        field.wrapper_ty().map_or_else(|| {
            quote! { #macro_path::Either::Left(#ref_tok #mut_tok self. #ident) }
        }, |as_ty| {
            let ty = field.ty();

            quote! {
                #macro_path::Either::Left(<
                    #ref_tok #mut_tok #as_ty
                        as ::core::convert::From< #ref_tok #mut_tok #ty >
                >::from(#ref_tok #mut_tok self. #ident))
            }
        })
    }
}

/// Delegated enum's [`Variant`]s or a single struct [`Field`].
#[derive(Clone, Debug)]
enum DelegatedTypes {
    /// [`Variant`]s of the enum.
    Variants(Vec<Variant>),

    /// [`Field`] of the struct.
    Field(Field),
}

impl DelegatedTypes {
    /// Returns an [`Iterator`] over these [`DelegatedTypes`].
    fn types(&self) -> impl Iterator<Item = &syn::Type> {
        use itertools::Either::{Left, Right};

        match self {
            Self::Variants(variants) => Left(
                variants
                    .iter()
                    .map(|var| var.wrapper_ty.as_ref().unwrap_or(&var.ty)),
            ),
            Self::Field(field) => Right(iter::once(
                field.wrapper_ty().unwrap_or_else(|| field.ty()),
            )),
        }
    }
}

/// Field of a struct.
#[derive(Clone, Debug)]
enum Field {
    /// [`Field`] of named struct.
    Named {
        /// [`Ident`] of this [`Field`].
        ///
        /// [`Ident`]: struct@syn::Ident
        ident: syn::Ident,

        /// [`Type`] of this [`Field`].
        ty: Box<syn::Type>,

        /// Wrapper [`Type`] for external delegation.
        wrapper_ty: Option<syn::Type>,
    },

    /// [`Field`] of tuple struct.
    Unnamed {
        /// [`Index`] of this [`Field`].
        index: syn::Index,

        /// [`Type`] of this [`Field`].
        ty: Box<syn::Type>,

        /// Wrapper [`Type`] for external delegation.
        wrapper_ty: Option<syn::Type>,
    },
}

impl Field {
    /// Returns an [`Ident`] or an [`Index`] to access this [`Field`].
    ///
    /// [`Ident`]: struct@syn::Ident
    fn ident(&self) -> TokenStream {
        match self {
            Self::Named { ident, .. } => ident.to_token_stream(),
            Self::Unnamed { index, .. } => index.to_token_stream(),
        }
    }

    /// Returns a [`Type`] of this [`Field`].
    const fn ty(&self) -> &syn::Type {
        match self {
            Self::Named { ty, .. } | Self::Unnamed { ty, .. } => ty,
        }
    }

    /// Returns wrapper [`Type`] for external delegation, if any.
    const fn wrapper_ty(&self) -> Option<&syn::Type> {
        match self {
            Self::Named { wrapper_ty, .. }
            | Self::Unnamed { wrapper_ty, .. } => wrapper_ty.as_ref(),
        }
    }
}

impl TryFrom<&mut syn::Fields> for Field {
    type Error = syn::Error;

    fn try_from(fields: &mut syn::Fields) -> Result<Self, Self::Error> {
        let span = fields.span();
        let field = fields.iter_mut().at_most_one().ok().flatten().ok_or_else(
            || syn::Error::new(span, "struct must have exactly one field"),
        )?;
        let args = InnerArgs::from_attrs(field.attrs.as_mut())?;
        let wrapper_ty = args.and_then(|a| a.r#as);

        Ok(match field.ident.as_ref() {
            Some(ident) => Self::Named {
                ident: ident.clone(),
                ty: Box::new(field.ty.clone()),
                wrapper_ty,
            },
            None => Self::Unnamed {
                index: syn::Index { index: 0, span: field.span() },
                ty: Box::new(field.ty.clone()),
                wrapper_ty,
            },
        })
    }
}

/// Variant of an enum.
#[derive(Clone, Debug)]
struct Variant {
    /// [`Ident`] of this [`Variant`].
    ///
    /// [`Ident`]: struct@syn::Ident
    ident: syn::Ident,

    /// [`Ident`] of the field contained in this variant.
    ///
    /// [`None`] means variant field is unnamed.
    ///
    /// [`Ident`]: struct@syn::Ident
    field_ident: Option<syn::Ident>,

    /// [`Type`] of this [`Variant`].
    ty: syn::Type,

    /// Wrapper [`Type`] for external delegation.
    wrapper_ty: Option<syn::Type>,
}

impl TryFrom<&mut syn::Variant> for Variant {
    type Error = syn::Error;

    fn try_from(variant: &mut syn::Variant) -> Result<Self, Self::Error> {
        let args = InnerArgs::from_attrs(variant.attrs.as_mut())?;

        variant
            .fields
            .iter()
            .at_most_one()
            .ok()
            .flatten()
            .map(|f| Self {
                ident: variant.ident.clone(),
                field_ident: f.ident.clone(),
                ty: f.ty.clone(),
                wrapper_ty: args.and_then(|a| a.r#as),
            })
            .ok_or_else(|| {
                syn::Error::new(
                    variant.fields.span(),
                    "enum variant must have exactly one field",
                )
            })
    }
}

/// Trait to be derived for a delegated type.
#[derive(Clone, Debug)]
struct DeriveTrait {
    /// Path of the trait to be derived.
    path: syn::Path,

    /// Type of external trait wrapper.
    wrapper_ty: Option<syn::Path>,

    /// [`Generics`] to be used in `impl` block.
    generics: Option<syn::Generics>,

    /// [`WhereClause`] to be used in `impl` block.
    where_clause: Option<syn::WhereClause>,
}

impl DeriveTrait {
    /// Returns [`Path`] to the macro implementing this trait.
    fn macro_rules_path(&self) -> syn::Path {
        if let Some(wrapper_ty) = &self.wrapper_ty {
            return wrapper_ty.clone();
        }

        let mut path = self.path.clone();
        if let Some(seg) = path.segments.last_mut() {
            seg.arguments = syn::PathArguments::None;
        }
        path
    }
}

impl Parse for DeriveTrait {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let generics = input
            .peek(token::For)
            .then(|| {
                _ = input.parse::<token::For>()?;
                input.parse::<syn::Generics>()
            })
            .transpose()?;
        let path = input.parse()?;
        let wrapper_ty = input
            .peek(token::As)
            .then(|| {
                _ = input.parse::<token::As>()?;
                input.parse()
            })
            .transpose()?;
        let where_clause = syn::WhereClause::parse_thrifty_opt(input)?;

        Ok(Self { path, wrapper_ty, generics, where_clause })
    }
}

/// Possible items passed to a [`Definition`].
#[derive(Clone, Debug)]
enum Item {
    /// Item is an enum.
    Enum(syn::ItemEnum),

    /// Item is a struct.
    Struct(syn::ItemStruct),
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(item) => item.to_tokens(tokens),
            Self::Struct(item) => item.to_tokens(tokens),
        }
    }
}
