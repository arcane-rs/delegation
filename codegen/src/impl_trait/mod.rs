//! `#[delegate]` macro expansion on traits.

mod util;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash as _, Hasher as _},
    iter,
};

use itertools::Itertools as _;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
#[cfg(doc)]
use syn::{Generics, Path, Signature, Type, Visibility, WhereClause};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    token,
};

use self::util::{GenericsExt as _, SignatureExt as _};
use crate::{
    MacroPath,
    util::{GenericsExt as _, WhereClauseExt as _},
};

/// Arguments of `#[delegate]` macro expansion on traits.
struct Args {
    /// `for` attribute argument, specifying types for deriving.
    r#for: Punctuated<ForTy, token::Comma>,

    /// `as` attribute argument, specifying path to the trait this trait is
    /// referencing to.
    r#as: Option<syn::Path>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut this = Self { r#for: Punctuated::new(), r#as: None };

        if input.is_empty() {
            return Ok(this);
        }

        loop {
            if input.peek(token::For) {
                _ = input.parse::<token::For>()?;
                let args;
                _ = syn::parenthesized!(args in input);
                this.r#for = Punctuated::parse_terminated(&args)?;
            } else if input.peek(token::As) {
                _ = input.parse::<token::As>()?;
                _ = input.parse::<token::Eq>()?;
                this.r#as = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "unexpected attribute argument",
                ));
            }

            if input.peek(token::Comma) {
                _ = input.parse::<token::Comma>()?;
            } else {
                break;
            }
        }

        Ok(this)
    }
}

/// Definitions of `#[delegate]` macro expansion on traits.
#[derive(Debug)]
pub(super) struct Definition {
    /// [`Visibility`] of the trait.
    vis: syn::Visibility,

    /// Indicator whether the trait is unsafe.
    unsafety: Option<token::Unsafe>,

    /// [`Ident`] of the trait.
    ///
    /// [`Ident`]: struct@syn::Ident
    ident: syn::Ident,

    /// [`Generics`] of the trait.
    generics: syn::Generics,

    /// Methods with `self` receiver.
    methods_owned: Vec<syn::TraitItemFn>,

    /// Methods with `&self` receiver.
    methods_ref: Vec<syn::TraitItemFn>,

    /// Methods with `&mut self` receiver.
    methods_ref_mut: Vec<syn::TraitItemFn>,

    /// Types for deriving trait on.
    delegate_for: Vec<ForTy>,

    /// [`Ident`] for generated trait, that contains only methods with `self`
    /// receiver.
    ///
    /// [`Ident`]: struct@syn::Ident
    owned_trait_ident: syn::Ident,

    /// [`Ident`] for generated trait, that contains only methods with `&self`
    /// receiver.
    ///
    /// [`Ident`]: struct@syn::Ident
    ref_trait_ident: syn::Ident,

    /// [`Ident`] for generated trait, that contains only methods with
    /// `&mut self` receiver.
    ///
    /// [`Ident`]: struct@syn::Ident
    ref_mut_trait_ident: syn::Ident,

    /// [`Ident`] for generated macro, that implements trait for provided type.
    ///
    /// [`Ident`]: struct@syn::Ident
    impl_macro_ident: syn::Ident,

    /// [`Ident`] for a scope of the trait types.
    ///
    /// [`Ident`]: struct@syn::Ident
    scope_ident: syn::Ident,

    /// [`Ident`] for bindings of the trait types.
    ///
    /// [`Ident`]: struct@syn::Ident
    bind_ident: syn::Ident,

    /// Wrapper type to implement blanket impl of the trait on.
    wrapper_ty: syn::Path,

    /// [`Item`] of this [`Definition`].
    item: Item,

    /// Path to the macro definitions.
    macro_path: MacroPath,
}

impl ToTokens for Definition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.define_item().to_tokens(tokens);

        self.generate_scope().to_tokens(tokens);
        self.blanket_impl_for_scope().to_tokens(tokens);

        self.generate_binds().to_tokens(tokens);
        self.assign_types_to_binds().to_tokens(tokens);

        self.generate_owned_trait().to_tokens(tokens);
        self.impl_owned_trait_for_either().to_tokens(tokens);
        self.impl_owned_trait_for_void().to_tokens(tokens);

        self.generate_ref_trait(false).to_tokens(tokens);
        self.impl_ref_trait_for_either(false).to_tokens(tokens);
        self.impl_ref_trait_for_void(false).to_tokens(tokens);

        self.generate_ref_trait(true).to_tokens(tokens);
        self.impl_ref_trait_for_either(true).to_tokens(tokens);
        self.impl_ref_trait_for_void(true).to_tokens(tokens);

        self.blanket_impl_for_wrapper_type().to_tokens(tokens);
        self.impl_macro_for_delegated_trait().to_tokens(tokens);

        self.impl_trait_for().to_tokens(tokens);

        self.generate_self_bound_assertions().to_tokens(tokens);
    }
}

impl Definition {
    /// Parses [`Definition`] from the provided [`syn::ItemTrait`].
    #[expect(clippy::too_many_lines, reason = "TODO: Refactor")]
    pub(super) fn parse(
        mut item: syn::ItemTrait,
        args: TokenStream,
    ) -> syn::Result<Self> {
        let args = syn::parse2::<Args>(args)?;
        let macro_path = MacroPath::default();

        let def_item = args.r#as.as_ref().map_or_else(
            || Item::Definition(item.clone()),
            |path| Item::External(path.clone()),
        );
        let item_hash = {
            let mut hasher = DefaultHasher::new();
            item.to_token_stream().to_string().hash(&mut hasher);
            hasher.finish()
        };

        let mut methods_owned = Vec::new();
        let mut methods_ref = Vec::new();
        let mut methods_ref_mut = Vec::new();

        for i in &item.items {
            match i {
                syn::TraitItem::Fn(m) => match m.sig.receiver() {
                    Some(syn::Receiver {
                        reference: Some(_),
                        mutability: Some(_),
                        colon_token: None,
                        ..
                    }) => methods_ref_mut.push(m.clone()),
                    Some(syn::Receiver {
                        reference: Some(_),
                        mutability: None,
                        colon_token: None,
                        ..
                    }) => methods_ref.push(m.clone()),
                    Some(syn::Receiver {
                        reference: None,
                        colon_token: None,
                        ..
                    }) => methods_owned.push(m.clone()),
                    Some(syn::Receiver { colon_token: Some(_), .. }) | None => {
                        return Err(syn::Error::new(
                            m.span(),
                            "all trait method must have an untyped receiver",
                        ));
                    }
                },
                syn::TraitItem::Type(_)
                | syn::TraitItem::Const(_)
                | syn::TraitItem::Macro(_)
                | syn::TraitItem::Verbatim(_) => {
                    return Err(syn::Error::new(
                        i.span(),
                        "only trait methods with untyped receiver are allowed",
                    ));
                }
                i => {
                    return Err(syn::Error::new(
                        i.span(),
                        format!("{i:#?} not covered"),
                    ));
                }
            }
        }

        for m in methods_owned
            .iter()
            .chain(methods_ref.iter())
            .chain(methods_ref_mut.iter())
        {
            let to_be_early_bounded = m.sig.to_be_early_bounded_lifetimes();
            if !to_be_early_bounded.is_empty() {
                return Err(syn::Error::new(
                    m.span(),
                    format!(
                        "lifetime {} are limited to be early-bounded. \
                         Consider adding `{}` bound or replace them with `'_'. \
                         See `rust-lang/rust#87803` for details.",
                        to_be_early_bounded.iter().format_with(", ", |l, f| {
                            f(&format_args!("`{l}`"))
                        }),
                        to_be_early_bounded
                            .iter()
                            .format_with(" + ", |l, f| {
                                f(&format_args!("{l}: {l}"))
                            }),
                    ),
                ));
            }
        }

        item.generics.make_where_clause().predicates.extend(
            item.supertraits.iter().map(|t| -> syn::WherePredicate {
                parse_quote! { Self: #t }
            }),
        );

        let owned_trait_ident =
            format_ident!("__delegate_{}__DelegateOwned", item.ident);
        let ref_trait_ident =
            format_ident!("__delegate_{}__DelegateRef", item.ident);
        let ref_mut_trait_ident =
            format_ident!("__delegate_{}__DelegateRefMut", item.ident);
        let impl_macro_ident = format_ident!(
            "__delegate_{}{item_hash}{}{}",
            item.ident,
            item.ident.span().start().line,
            item.ident.span().start().column,
        );
        let wrapper_ty = args
            .r#as
            .map(|_| {
                syn::parse_str(&format!("__delegate_{}__Wrapper", item.ident))
            })
            .transpose()?
            .unwrap_or_else(|| parse_quote! { #macro_path ::Wrapper });
        let scope_ident = format_ident!("__delegate_{}__Scope", item.ident);
        let bind_ident = format_ident!("__delegate_{}__Bind", item.ident);

        Ok(Self {
            vis: item.vis,
            unsafety: item.unsafety,
            ident: item.ident,
            generics: item.generics,
            methods_owned,
            methods_ref,
            methods_ref_mut,
            delegate_for: args.r#for.into_iter().collect(),
            owned_trait_ident,
            ref_trait_ident,
            ref_mut_trait_ident,
            impl_macro_ident,
            wrapper_ty,
            scope_ident,
            bind_ident,
            item: def_item,
            macro_path,
        })
    }

    /// Defines a trait [`Item`].
    ///
    /// [`Item`] differs relying on the `#[delegate(as = ..)]` attribute:
    /// - For crate-local traits it's just a trait definition.
    /// - For external traits it's a newtype wrapper to implement the trait for.
    fn define_item(&self) -> TokenStream {
        match &self.item {
            Item::Definition(def) => {
                let scope_ident = &self.scope_ident;

                let mut def = def.clone();
                def.supertraits.push(parse_quote! { #scope_ident });
                def.to_token_stream()
            }
            Item::External(_) => {
                let vis = &self.vis;
                let wrapper_ty = &self.wrapper_ty;

                quote! {
                    #[automatically_derived]
                    #[derive(Clone, Copy, Debug)]
                    #[doc(hidden)]
                    #[repr(transparent)]
                    #vis struct #wrapper_ty <T>(T)
                    where
                        T: ?::core::marker::Sized;
                }
            }
        }
    }

    /// Generates a trait representing a scope of types for the delegated trait.
    ///
    /// Required for:
    /// - Bypassing the orphan rules.
    /// - Resolving the types of trait scope on enum-side.
    fn generate_scope(&self) -> TokenStream {
        let vis = &self.vis;

        let assoc_ty = self.methods_types().enumerate().map(
            |(seq_num, (method_gens, _))| {
                let bind_ident = format_ident!("{}{seq_num}", &self.bind_ident);

                let gens = {
                    let mut gens = self.generics.clone();
                    gens.append(&method_gens);
                    gens.remove_self_ty_bounds();
                    gens
                };
                let (impl_gens, _, where_clause) = gens.split_for_impl();

                quote! {
                    #[automatically_derived]
                    #[allow(non_camel_case_types, reason = "macro expansion")]
                    #[doc(hidden)]
                    type #bind_ident #impl_gens #where_clause;
                }
            },
        );

        match &self.item {
            Item::Definition(_) => {
                let scope_ident = &self.scope_ident;

                quote! {
                    #[automatically_derived]
                    #[allow(non_camel_case_types, reason = "macro expansion")]
                    #[doc(hidden)]
                    #vis trait #scope_ident {
                        #( #assoc_ty )*
                    }
                }
            }
            Item::External(as_trait) => {
                let ident = &self.ident;
                let doc = format!(
                    "Local definition of `{}` trait.\
                     \n\n\
                     > __NOTE__: Do not use directly. Should be used only in \
                     `#[delegate(derive(..)]` attribute.",
                    as_trait.to_token_stream(),
                );

                quote! {
                    #[automatically_derived]
                    #[doc = #doc]
                    #vis trait #ident <__Delegate: ?::core::marker::Sized> {
                        #[doc(hidden)]
                        type Wrapper: ?::core::marker::Sized;

                        #( #assoc_ty )*
                    }
                }
            }
        }
    }

    /// Generates a blanket implementation for the supertrait generated by the
    /// [`Self::generate_scope()`] method.
    fn blanket_impl_for_scope(&self) -> TokenStream {
        let assoc_ty = self.methods_types().enumerate().map(
            |(seq_num, (method_gens, _))| {
                let bind_ident = format_ident!("{}{seq_num}", &self.bind_ident);

                let gens = {
                    let mut gens = self.generics.clone();
                    gens.append(&method_gens);
                    gens.remove_self_ty_bounds();
                    gens
                };
                let (impl_gens, _, where_clause) = gens.split_for_impl();

                let gens_ty = {
                    let mut gens = gens.clone();
                    gens.params.push(parse_quote! { __Delegate });
                    gens
                };
                let (_, ty_gens, _) = gens_ty.split_for_impl();

                quote! {
                    #[automatically_derived]
                    #[allow(non_camel_case_types, reason = "macro expansion")]
                    #[doc(hidden)]
                    type #bind_ident #impl_gens = #bind_ident #ty_gens
                    #where_clause;
                }
            },
        );

        match &self.item {
            Item::Definition(_) => {
                let scope_ident = &self.scope_ident;

                let gens = {
                    let mut gens = syn::Generics::default();
                    gens.params.push(
                        parse_quote! { __Delegate: ?::core::marker::Sized },
                    );
                    gens
                };
                let (impl_gens, _, _) = gens.split_for_impl();

                quote! {
                    #[automatically_derived]
                    impl #impl_gens #scope_ident for __Delegate {
                        #( #assoc_ty )*
                    }
                }
            }
            Item::External(_) => {
                let macro_path = &self.macro_path;
                let ident = &self.ident;
                let wrapper_ty = &self.wrapper_ty;

                quote! {
                    #[automatically_derived]
                    impl<__Delegate: ?::core::marker::Sized> #ident <__Delegate>
                        for #macro_path ::External
                    {
                        #[doc(hidden)]
                        type Wrapper = #wrapper_ty <__Delegate>;

                        #( #assoc_ty )*
                    }
                }
            }
        }
    }

    /// Generates bind types containing all the [`Generics`] required for
    /// resolution of the types from trait scope.
    fn generate_binds(&self) -> TokenStream {
        let vis = &self.vis;

        let bind_ty = self.methods_types().enumerate().map(
            |(seq_num, (method_gens, _))| {
                let bind_ident = format_ident!("{}{seq_num}", &self.bind_ident);

                let gens = {
                    let mut gens = self.generics.clone();
                    gens.append(&method_gens);
                    gens.params.push(
                        parse_quote! { __Delegate: ?::core::marker::Sized },
                    );
                    gens.remove_self_ty_bounds();
                    gens
                };
                let (impl_gens, _, where_clause) = gens.split_for_impl();

                let phantom_data = gens.phantom_data();

                quote! {
                    #[automatically_derived]
                    #[allow(non_camel_case_types, reason = "macro expansion")]
                    #[doc(hidden)]
                    #vis struct #bind_ident #impl_gens (#phantom_data)
                    #where_clause;
                }
            },
        );

        quote! { #( #bind_ty )* }
    }

    /// Assigns real types from the trait scope to associated bind types.
    ///
    /// Such assignments allow to resolve the types from the trait scope when
    /// expanding nested `macro_rules!` macro expansion.
    fn assign_types_to_binds(&self) -> TokenStream {
        let macro_path = &self.macro_path;

        let impls = self.methods_types().enumerate().map(
            |(seq_num, (method_gens, ty))| {
                let bind_ident = format_ident!("{}{seq_num}", &self.bind_ident);

                let bind_gens = {
                    let mut gens = self.generics.clone();
                    gens.append(&method_gens);
                    gens.params.push(
                        parse_quote! { __Delegate: ?::core::marker::Sized },
                    );
                    gens.replace_self_ty(&parse_quote! { __Delegate });
                    gens
                };
                let (_, ty_gens, _) = bind_gens.split_for_impl();

                let impl_gens = {
                    let mut gens = bind_gens.clone();
                    gens.bound_type_to_lifetimes(&ty);
                    gens
                };
                let (impl_gens, _, where_clause) = impl_gens.split_for_impl();

                quote! {
                    #[automatically_derived]
                    impl #impl_gens #macro_path::TypeOf for #bind_ident #ty_gens
                    #where_clause
                    {
                        #[doc(hidden)]
                        type T = #ty;
                    }
                }
            },
        );

        quote! { #( #impls )* }
    }

    /// Generates a trait containing only methods with `self` receiver.
    fn generate_owned_trait(&self) -> TokenStream {
        let owned_trait = &self.owned_trait_ident;
        let generics = &self.generics;
        let where_clause = &self.generics.where_clause;
        let owned_methods = &self.methods_owned;

        quote! {
            #[automatically_derived]
            #[allow(non_camel_case_types, reason = "macro expansion")]
            trait #owned_trait #generics #where_clause {
                #( #owned_methods )*
            }
        }
    }

    /// Implements a trait generated by the [`Self::generate_owned_trait()`]
    /// method for an `Either`.
    fn impl_owned_trait_for_either(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let orig_trait = self.item.path();
        let owned_trait = &self.owned_trait_ident;

        let (_, ty_gens, _) = self.generics.split_for_impl();

        let generics = {
            let mut gens = self.generics.clone();

            let params: [syn::GenericParam; 2] =
                [parse_quote! { __Left }, parse_quote! { __Right }];
            gens.params.extend(params);

            let predicates: [syn::WherePredicate; 2] = [
                parse_quote! { __Left: #orig_trait #ty_gens },
                parse_quote! { __Right: #owned_trait #ty_gens },
            ];
            gens.make_where_clause().predicates.extend(predicates);

            gens
        };
        let (impl_gens, _, where_clause) = generics.split_for_impl();

        let methods = self.methods_owned.iter().map(|m| {
            let (signature, method_name, method_inputs) =
                m.sig.split_for_impl();
            let method_inputs = method_inputs.collect::<Vec<_>>();

            quote! {
                #signature {
                    match self {
                        Self::Left(__delegate) => {
                            <__Left as #orig_trait #ty_gens>::#method_name(
                                __delegate, #( #method_inputs ),*
                            )
                        }
                        Self::Right(__delegate) => {
                            <__Right as #owned_trait #ty_gens>::#method_name(
                                __delegate, #( #method_inputs ),*
                            )
                        }
                    }
                }
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_gens #owned_trait #ty_gens
             for #macro_path::Either<__Left, __Right> #where_clause
            {
                #( #methods )*
            }
        }
    }

    /// Implements a trait generated by the [`Self::generate_owned_trait()`]
    /// method for a `Void`.
    fn impl_owned_trait_for_void(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let owned_trait = &self.owned_trait_ident;

        let (impl_gens, ty_gens, where_clause) = self.generics.split_for_impl();

        let methods = self.methods_owned.iter().map(|m| {
            let signature = &m.sig;

            quote! {
                #signature {
                    match self {}
                }
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_gens #owned_trait #ty_gens
             for #macro_path::Void #where_clause
            {
                #( #methods )*
            }
        }
    }

    /// Generates a trait containing only methods with `&self` (or `&mut self`)
    /// receiver.
    fn generate_ref_trait(&self, mutable: bool) -> TokenStream {
        let ref_trait = if mutable {
            &self.ref_mut_trait_ident
        } else {
            &self.ref_trait_ident
        };

        let generics = self.ref_trait_generics();
        let where_clause = &generics.where_clause;

        let methods = self.ref_trait_signatures(mutable);

        quote! {
            #[automatically_derived]
            #[allow(non_camel_case_types, reason = "macro expansion")]
            trait #ref_trait #generics #where_clause {
                #( #methods; )*
            }
        }
    }

    /// Implements a trait generated by the [`Self::generate_ref_trait()`]
    /// method for an `Either`.
    fn impl_ref_trait_for_either(&self, mutable: bool) -> TokenStream {
        let mut_ = mutable.then(|| quote! { mut });
        let macro_path = &self.macro_path;
        let orig_trait = self.item.path();
        let ref_trait = if mutable {
            &self.ref_mut_trait_ident
        } else {
            &self.ref_trait_ident
        };

        let (_, trait_ty_gens, _) = self.generics.split_for_impl();

        let ref_trait_generics = self.ref_trait_generics();
        let (_, ref_trait_ty_gens, _) = ref_trait_generics.split_for_impl();

        let impl_generics = {
            let mut gens = ref_trait_generics.clone();

            let params: [syn::GenericParam; 2] =
                [parse_quote! { __Left }, parse_quote! { __Right }];
            gens.params.extend(params);

            let predicates: [syn::WherePredicate; 2] = [
                parse_quote! { __Left: #orig_trait #trait_ty_gens },
                parse_quote! { __Right: #ref_trait #ref_trait_ty_gens },
            ];
            gens.make_where_clause().predicates.extend(predicates);

            gens
        };
        let (impl_gens, _, where_clause) = impl_generics.split_for_impl();

        let methods = self.ref_trait_signatures(mutable).map(|signature| {
            let (signature, method_name, method_inputs) =
                signature.split_for_impl();
            let method_inputs = method_inputs.collect::<Vec<_>>();

            quote! {
                #signature {
                    match self {
                        Self::Left(__delegate) => {
                            <__Left as #orig_trait #trait_ty_gens>
                            ::#method_name(
                                __delegate, #( #method_inputs ),*
                            )
                        }
                        Self::Right(__delegate) => {
                            <__Right as #ref_trait #ref_trait_ty_gens>
                            ::#method_name(
                                __delegate, #( #method_inputs ),*
                            )
                        }
                    }
                }
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_gens #ref_trait #ref_trait_ty_gens
             for #macro_path::Either<&'__delegate #mut_ __Left, __Right>
                 #where_clause
            {
                #( #methods )*
            }
        }
    }

    /// Implements a trait generated by the [`Self::generate_ref_trait()`]
    /// method for a `Void`.
    fn impl_ref_trait_for_void(&self, mutable: bool) -> TokenStream {
        let macro_path = &self.macro_path;
        let ref_trait = if mutable {
            &self.ref_mut_trait_ident
        } else {
            &self.ref_trait_ident
        };

        let generics = self.ref_trait_generics();
        let (impl_gens, ty_gens, where_clause) = generics.split_for_impl();

        let methods = self.ref_trait_signatures(mutable).map(|signature| {
            quote! {
                #signature {
                    match self {}
                }
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_gens #ref_trait #ty_gens
             for #macro_path::Void #where_clause
            {
                #( #methods )*
            }
        }
    }

    /// Generates a blanket impl of the delegated trait for type, wrapping the
    /// inner type that implements the `Convert` trait, where its associated
    /// types satisfy the corresponding generated traits.
    fn blanket_impl_for_wrapper_type(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let unsafety = self.unsafety;
        let trait_path = self.item.path();

        let owned_ident = &self.owned_trait_ident;
        let ref_ident = &self.ref_trait_ident;
        let ref_mut_ident = &self.ref_mut_trait_ident;

        let for_ty = quote! { __Delegate };
        let wrapper_ty = &self.wrapper_ty;

        let (_, trait_ty_gens, _) = self.generics.split_for_impl();

        let ref_trait_generics = self.ref_trait_generics();
        let (_, ref_trait_ty_gens, _) = ref_trait_generics.split_for_impl();

        let ref_trait_anon_generics = {
            let mut gens = self.generics.clone();
            gens.params.push(parse_quote! { '_ });
            gens
        };
        let (_, ref_trait_anon_ty_gens, _) =
            ref_trait_anon_generics.split_for_impl();

        let impl_generics = {
            let mut gens = self.generics.clone();

            gens.params.push(parse_quote! { #for_ty });

            let predicates: [syn::WherePredicate; 4] = [
                parse_quote! { #for_ty: #macro_path::Convert },
                parse_quote! {
                    <#for_ty as #macro_path::Convert>::Owned:
                        #owned_ident #trait_ty_gens
                },
                parse_quote! {
                    for<'__delegate>
                    <#for_ty as #macro_path::Convert>::Ref<'__delegate>:
                        #ref_ident #ref_trait_ty_gens
                },
                parse_quote! {
                    for<'__delegate>
                    <#for_ty as #macro_path::Convert>::RefMut<'__delegate>:
                        #ref_mut_ident #ref_trait_ty_gens
                },
            ];
            gens.make_where_clause().predicates.extend(predicates);

            gens
        };
        let (impl_gens, _, where_clause) = impl_generics.split_for_impl();

        let owned_methods = self.methods_owned.iter().map(|m| {
            let (signature, method_name, method_inputs) =
                m.sig.split_for_impl();

            quote! {
                #signature {
                    <<#for_ty as #macro_path::Convert>::Owned
                     as #owned_ident>
                    ::#method_name(
                        <#for_ty as #macro_path::Convert>::convert_owned(
                            self.0
                        ),
                        #( #method_inputs ),*
                    )
                }
            }
        });
        let ref_methods = self.methods_ref.iter().map(|m| {
            let (signature, method_name, method_inputs) =
                m.sig.split_for_impl();

            quote! {
                #signature {
                    <<#for_ty as #macro_path::Convert>::Ref<'_>
                     as #ref_ident #ref_trait_anon_ty_gens>
                    ::#method_name(
                        <#for_ty as #macro_path::Convert>::convert_ref(&self.0),
                        #( #method_inputs ),*
                    )
                }
            }
        });
        let ref_mut_methods = self.methods_ref_mut.iter().map(|m| {
            let (signature, method_name, method_inputs) =
                m.sig.split_for_impl();

            quote! {
                #signature {
                    <<#for_ty as #macro_path::Convert>::RefMut<'_>
                     as #ref_mut_ident #ref_trait_anon_ty_gens>
                    ::#method_name(
                        <#for_ty as #macro_path::Convert>::convert_ref_mut(
                            &mut self.0
                        ),
                        #( #method_inputs ),*
                    )
                }
            }
        });

        quote! {
            #[automatically_derived]
            #unsafety impl #impl_gens #trait_path #trait_ty_gens
                for #wrapper_ty < #for_ty >
            #where_clause
            {
                #( #owned_methods )*
                #( #ref_methods )*
                #( #ref_mut_methods )*
            }
        }
    }

    /// Generates a declarative macro used to implement the trait for a type,
    /// provided to it.
    ///
    /// Generated macro, for now, just passes all its inputs to `impl_for!`,
    /// which fills the temporary unknown types with types, provided by
    /// `macro_rules!` arguments.
    fn impl_macro_for_delegated_trait(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let vis = &self.vis;
        let unsafety = &self.unsafety;
        let ident = &self.ident;
        let impl_macro_ident = &self.impl_macro_ident;
        let wrapper_ty = &self.wrapper_ty;

        let (impl_gens, ty_gens, where_clause) = self.generics.split_for_impl();

        let (trait_path, self_wrapped) = match &self.item {
            Item::Definition(_) => {
                (parse_quote! { #ident }, quote! { #wrapper_ty<Self> })
            }
            Item::External(as_trait) => (
                as_trait.clone(),
                quote! {
                    <#macro_path ::External as #ident<Self>>::Wrapper #ty_gens
                },
            ),
        };

        let mut seq_num: usize = 0;
        let methods = self
            .methods_owned
            .iter()
            .chain(&self.methods_ref)
            .chain(&self.methods_ref_mut)
            .map(|m| {
                let (_, method_name, method_inputs) = m.sig.split_for_impl();

                let signature = self.bind_signature_types(&m.sig, &mut seq_num);

                // TODO: Use `RefCast` here instead of `mem::transmute`.
                let mut receiver = quote! { ::core::mem::transmute(self) };
                if m.sig.unsafety.is_none() {
                    receiver = quote! {
                        // SAFETY: Wrapper is `#[repr(transparent)]`.
                        #[allow( // macro expansion
                            clippy::missing_transmute_annotations,
                            clippy::transmute_ptr_to_ptr,
                            unsafe_code,
                            reason = "macro expansion",
                        )]
                        unsafe { #receiver }
                    };
                }
                let body = quote! {
                    <#self_wrapped as #trait_path #ty_gens>:: #method_name(
                        #receiver, #( #method_inputs ),*
                    )
                };
                if m.sig.unsafety.is_some() {
                    quote! {
                        #signature {
                            // SAFETY: Wrapper is `#[repr(transparent)]`.
                            #[allow( // macro expansion
                                clippy::missing_transmute_annotations,
                                clippy::transmute_ptr_to_ptr,
                                reason = "macro expansion",
                            )]
                            unsafe { #body }
                        }
                    }
                } else {
                    quote! {
                        #signature { #body }
                    }
                }
            });

        let impl_block = quote! {
            #[automatically_derived]
            #unsafety impl #impl_gens #trait_path #ty_gens for T #where_clause
            {
                #( #methods )*
            }
        };

        // TODO: `macro_rules!` has the following limitations and requires to
        //       use the inner procedural macro:
        //       - generics can't be normally parsed in `macro_rules!` macro
        //       - type containing generic parameters can't be passed without
        //         them.
        quote! {
            #[automatically_derived]
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #impl_macro_ident {
                ($($tok:tt)*) => {
                    #macro_path::impl_for! {
                        #impl_block
                        $($tok)*
                    }
                };
            }

            // Always `pub` because of `#[macro_export]` limitation.
            #[automatically_derived]
            #[doc(hidden)]
            #[allow( // macro expansion
                non_snake_case,
                unused_imports,
                reason = "macro expansion",
            )]
            #vis use #impl_macro_ident as #ident;
        }
    }

    /// Implements the delegated trait for provided `for` types.
    fn impl_trait_for(&self) -> TokenStream {
        let macro_path = &self.macro_path;
        let ident = &self.ident;
        let trait_path = self.item.path();

        let (_, ty_gens, _) = self.generics.split_for_impl();

        let wrapper = match &self.item {
            Item::Definition(_) => quote! { #macro_path ::Wrapper },
            Item::External(_) => quote! { #ident },
        };

        self.delegate_for
            .iter()
            .map(|for_ty| {
                let ty = &for_ty.ty;
                let gens = self
                    .generics
                    .merge(for_ty.generics.as_ref())
                    .merge_where_clause(for_ty.where_clause.as_ref());
                let (impl_gens, _, where_clause) = gens.split_for_impl();

                quote! {
                    #ident!(
                        impl #impl_gens #trait_path #ty_gens as #wrapper
                        for #ty
                        #where_clause
                    );
                }
            })
            .collect()
    }

    // TODO: Add proper support for `Self:` bounds.
    /// Generates assertion of `Self:` bounds containing only marker traits like
    /// [`Sized`], [`Send`] or [`Sync`].
    fn generate_self_bound_assertions(&self) -> TokenStream {
        /// Returns an [`Iterator`] of [`syn::TraitBound`] for `Self:`.
        fn self_trait_bounds(
            pred: &syn::WherePredicate,
        ) -> impl Iterator<Item = &syn::TraitBound> {
            let self_: syn::Type = parse_quote! { Self };

            match pred {
                syn::WherePredicate::Type(syn::PredicateType {
                    bounded_ty,
                    bounds,
                    ..
                }) => (bounded_ty == &self_).then(|| {
                    bounds.iter().filter_map(|b| match b {
                        syn::TypeParamBound::Trait(tr) => Some(tr),
                        syn::TypeParamBound::Lifetime(_)
                        | syn::TypeParamBound::Verbatim(_)
                        | syn::TypeParamBound::PreciseCapture(_) => None,
                        bound => panic!("{bound:#?} not covered"),
                    })
                }),
                syn::WherePredicate::Lifetime(_) => None,
                pred => panic!("unknown `syn::WherePredicate`: {pred:?}"),
            }
            .into_iter()
            .flatten()
        }

        let self_bounds = self
            .generics
            .where_clause
            .iter()
            .flat_map(|cl| cl.predicates.iter())
            .flat_map(self_trait_bounds);

        quote! {
            #[automatically_derived]
            const _: fn() = || {
                struct OnlyMarkerSelfBoundsSupportedForNow;

                fn assert_impl_all<T: Sized #(+ #self_bounds )*>() {}
                assert_impl_all::<OnlyMarkerSelfBoundsSupportedForNow>();
            };
        }
    }

    /// Returns an [`Iterator`] over methods with `&self` or `&mut self`
    /// receivers with [lifted] lifetimes.
    ///
    /// [lifted]: util::SignatureExt::lift_receiver_lifetime()
    fn ref_trait_signatures(
        &self,
        mutable: bool,
    ) -> impl Iterator<Item = syn::Signature> {
        if mutable {
            self.methods_ref_mut.iter()
        } else {
            self.methods_ref.iter()
        }
        .cloned()
        .map(|mut method| {
            method.sig.lift_receiver_lifetime(parse_quote! { '__delegate });
            method.sig
        })
    }

    /// Returns [`Generics`] for traits generated by the
    /// [`Self::generate_ref_trait()`] method.
    fn ref_trait_generics(&self) -> syn::Generics {
        let mut gens = self.generics.clone();
        gens.params.push(parse_quote! { '__delegate });
        gens.make_where_clause()
            .predicates
            .push(parse_quote! { Self: Sized + '__delegate });
        gens
    }

    /// Returns [`Type`]s specified in method [`Signature`]s and their
    /// [`Generics`].
    fn methods_types(
        &self,
    ) -> impl Iterator<Item = (syn::Generics, syn::Type)> {
        self.methods_owned
            .iter()
            .chain(&self.methods_ref)
            .chain(&self.methods_ref_mut)
            .flat_map(|m| {
                let mut sig = m.sig.clone();

                let mut lt_count: usize = 0;
                sig.expand_lifetimes(parse_quote! { '__delegate }, || {
                    lt_count += 1;
                    syn::Lifetime::new(
                        &format!("'__delegate{lt_count}"),
                        Span::call_site(),
                    )
                });

                let output = (sig.generics.clone(), sig.return_type());

                sig.inputs
                    .iter()
                    .filter_map(move |i| match i {
                        syn::FnArg::Receiver(_) => None,
                        syn::FnArg::Typed(t) => {
                            Some((sig.generics.clone(), (*t.ty).clone()))
                        }
                    })
                    .chain(iter::once(output))
                    .collect::<Vec<_>>()
            })
    }

    /// Replaces all [`Type`]s in the provided [`Signature`] with bound
    /// [`Type`]s.
    fn bind_signature_types(
        &self,
        orig: &syn::Signature,
        seq_num: &mut usize,
    ) -> syn::Signature {
        let macro_path = &self.macro_path;
        let ident = &self.ident;

        let gens = {
            let mut sig = orig.clone();

            let lt: syn::Lifetime = parse_quote! { '_ };
            sig.expand_lifetimes(lt.clone(), || lt.clone());

            let mut gens = self.generics.clone();
            gens.append(&sig.generics);
            gens
        };
        let (_, ty_gens, _) = gens.split_for_impl();

        let mut bind_ty = || {
            let bind_ident = format_ident!("{}{seq_num}", &self.bind_ident);
            *seq_num += 1;

            match &self.item {
                Item::Definition(_) => {
                    parse_quote! {
                        <Self::#bind_ident #ty_gens
                             as #macro_path ::TypeOf>::T
                    }
                }
                Item::External(_) => {
                    parse_quote! {
                        <<#macro_path ::External
                          as #ident<Self>>::#bind_ident #ty_gens
                             as #macro_path ::TypeOf>::T
                    }
                }
            }
        };

        let mut binded = orig.clone();
        binded.inputs.iter_mut().for_each(|i| {
            if let syn::FnArg::Typed(ty) = i {
                ty.ty = bind_ty();
            }
        });
        binded.output =
            syn::ReturnType::Type(token::RArrow::default(), bind_ty());
        binded
    }
}

/// Type to delegate the trait for.
#[derive(Clone, Debug)]
struct ForTy {
    /// Name of the type to delegate the trait for.
    ty: syn::Path,

    /// [`Generics`] to be used in `impl` block.
    generics: Option<syn::Generics>,

    /// [`WhereClause`] to be used in `impl` block.
    where_clause: Option<syn::WhereClause>,
}

impl Parse for ForTy {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let generics = input
            .peek(token::For)
            .then(|| {
                _ = input.parse::<token::For>()?;
                input.parse()
            })
            .transpose()?;
        let ty = input.parse()?;
        let where_clause = syn::WhereClause::parse_thrifty_opt(input)?;

        Ok(Self { ty, generics, where_clause })
    }
}

/// Possible items passed to a [`Definition`].
#[derive(Clone, Debug)]
enum Item {
    /// Definition of the crate-local trait.
    Definition(syn::ItemTrait),

    /// Definition of the external trait.
    External(syn::Path),
}

impl Item {
    /// Returns [`Path`] of this [`Item`].
    fn path(&self) -> syn::Path {
        match self {
            Self::Definition(item) => item.ident.clone().into(),
            Self::External(path) => path.clone(),
        }
    }
}
