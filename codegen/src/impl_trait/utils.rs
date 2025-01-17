//! Utils for `#[delegate]` macro expansion on traits.

use std::{
    collections::{HashMap, HashSet},
    iter, mem,
};

use quote::quote;
use syn::{
    parse_quote, punctuated,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
};

/// Extension of a [`syn::Generics`].
pub(super) trait GenericsExt {
    /// Returns [`PhantomData`] type for holding this [`Generics`].
    ///
    /// [`Generics`]: syn::Generics
    /// [`PhantomData`]: core::marker::PhantomData
    fn phantom_data(&self) -> syn::Type;

    /// Appends provided [`Generics`] to this one.
    ///
    /// [`Generics`]: syn::Generics
    fn append(&mut self, other: &syn::Generics);

    /// Generate bounds guaranteeing that provided type is live as long as the
    /// lifetimes specified in it.
    fn bound_type_to_lifetimes(&mut self, ty: &syn::Type);

    /// Replaces `Self` ty with provided type.
    fn replace_self_ty(&mut self, ty: &syn::Type);

    /// Removes `Self:` bounds from this [`Generics`].
    ///
    /// [`Generics`]: syn::Generics
    fn remove_self_ty_bounds(&mut self);
}

impl GenericsExt for syn::Generics {
    fn phantom_data(&self) -> syn::Type {
        let ty = self.params.iter().filter_map(|p| match p {
            syn::GenericParam::Type(ty) => {
                let ident = &ty.ident;
                Some(quote! { *const #ident })
            }
            syn::GenericParam::Lifetime(def) => {
                let lt = &def.lifetime;
                Some(quote! { &#lt () })
            }
            syn::GenericParam::Const(_) => None,
        });

        parse_quote! { ::core::marker::PhantomData< ( #(#ty),* ) > }
    }

    fn append(&mut self, other: &syn::Generics) {
        self.params.extend(other.params.iter().cloned());
        self.where_clause
            .get_or_insert_with(|| parse_quote! { where })
            .predicates
            .extend(
                other
                    .where_clause
                    .iter()
                    .flat_map(|wc| wc.predicates.iter().cloned()),
            );
    }

    fn bound_type_to_lifetimes(&mut self, ty: &syn::Type) {
        /// Visitor for collecting types containing [`syn::Lifetime`]s.
        pub(super) struct CollectTypesWithLifetimesVisitor {
            /// [`syn::Lifetime`]s to search for.
            lifetimes: HashSet<syn::Ident>,

            /// Collected [`syn::Type`]s.
            types: HashMap<syn::Type, Vec<syn::Lifetime>>,
        }

        impl CollectTypesWithLifetimesVisitor {
            /// Creates a new [`CollectTypesWithLifetimesVisitor`].
            #[expect(
                single_use_lifetimes,
                reason = "use anonymous lifetimes instead once supported"
            )]
            fn new<'a>(
                lifetimes: impl IntoIterator<Item = &'a syn::Lifetime>,
            ) -> Self {
                Self {
                    lifetimes: lifetimes
                        .into_iter()
                        .map(|lt| lt.ident.clone())
                        .collect::<HashSet<_>>(),
                    types: HashMap::new(),
                }
            }
        }

        impl<'ast> Visit<'ast> for CollectTypesWithLifetimesVisitor {
            fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {
                if self.lifetimes.contains(&i.ident) {
                    #[expect(
                        clippy::iter_over_hash_type,
                        reason = "order doesn't matter here"
                    )]
                    for lifetimes in self.types.values_mut() {
                        lifetimes.push(i.clone());
                    }
                }

                visit::visit_lifetime(self, i);
            }

            fn visit_type(&mut self, i: &'ast syn::Type) {
                drop(self.types.insert(i.clone(), Vec::new()));

                visit::visit_type(self, i);
            }
        }

        let mut visitor = CollectTypesWithLifetimesVisitor::new(
            self.lifetimes().map(|d| &d.lifetime),
        );
        visitor.visit_type(ty);

        #[expect(
            clippy::iter_over_hash_type,
            reason = "order doesn't matter here"
        )]
        for (t, lt) in &visitor.types {
            if lt.is_empty() {
                continue;
            }

            self.where_clause
                .get_or_insert_with(|| parse_quote! { where })
                .predicates
                .push(parse_quote! { #t: #( #lt )+* });
        }
    }

    fn replace_self_ty(&mut self, ty: &syn::Type) {
        /// Visitor for replacing `Self` with provided type.
        struct ReplaceSelfTy<'a> {
            /// Type to replace `Self` with.
            ty: &'a syn::Type,
        }

        impl VisitMut for ReplaceSelfTy<'_> {
            fn visit_type_mut(&mut self, i: &mut syn::Type) {
                if let syn::Type::Path(path) = i {
                    if path.path.is_ident("Self") {
                        *i = self.ty.clone();
                    }
                }

                visit_mut::visit_type_mut(self, i);
            }
        }

        let mut visitor = ReplaceSelfTy { ty };
        visitor.visit_generics_mut(self);
    }

    fn remove_self_ty_bounds(&mut self) {
        let Some(where_clause) = &mut self.where_clause else {
            return;
        };

        for pred in mem::take(&mut where_clause.predicates) {
            if let syn::WherePredicate::Type(ty) = &pred {
                if let syn::Type::Path(path) = &ty.bounded_ty {
                    if path.path.is_ident("Self") {
                        continue;
                    }
                }
            }

            where_clause.predicates.push(pred);
        }
    }
}

/// Extension of a [`syn::Signature`].
pub(super) trait SignatureExt {
    /// Helper for implementing method on a `Either`.
    ///
    /// Returns method's [`Signature`] itself, [`Ident`] and [`Iterator`] over
    /// inputs, excluding the [`Receiver`].
    ///
    /// [`Ident`]: struct@syn::Ident
    /// [`Receiver`]: syn::Receiver
    /// [`Signature`]: syn::Signature
    fn split_for_impl(
        &self,
    ) -> (&syn::Signature, &syn::Ident, InputsWithoutReceiverIter<'_>);

    /// Lifts [`Lifetime`] from a [`Receiver`] leaving `self` in its place and
    /// replaces/inserts all the needed occurrences with the provided
    /// `replace_with` [`Lifetime`].
    ///
    /// This method is useful, when you need lift [`Receiver`]'s [`Lifetime`] to
    /// trait/struct generic level.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    /// [`Receiver`]: syn::Receiver
    fn lift_receiver_lifetime(&mut self, replace_with: syn::Lifetime);

    /// Expands all elided lifetimes in the [`Signature`].
    ///
    /// [`Signature`]: syn::Signature
    fn expand_lifetimes<F>(&mut self, receiver_lt: syn::Lifetime, expand_fn: F)
    where
        F: FnMut() -> syn::Lifetime;

    /// Returns [`ReturnType`] of the [`Signature`] including default return
    /// type.
    ///
    /// [`ReturnType`]: syn::ReturnType
    /// [`Signature`]: syn::Signature
    fn return_type(&self) -> syn::Type;

    /// Returns [`Lifetime`]s presented in this [`Signature`] that
    /// are limited to be early bounded.
    ///
    /// See [`rust-lang/rust#87803`] for more details.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    /// [`Signature`]: syn::Signature
    /// [`rust-lang/rust#87803`]: https://github.com/rust-lang/rust/issues/87803
    fn to_be_early_bounded_lifetimes(&self) -> HashSet<syn::Lifetime>;
}

impl SignatureExt for syn::Signature {
    fn split_for_impl(
        &self,
    ) -> (&syn::Signature, &syn::Ident, InputsWithoutReceiverIter<'_>) {
        (
            self,
            &self.ident,
            self.inputs.iter().filter_map(|i| match i {
                syn::FnArg::Typed(t) => Some(&t.pat),
                syn::FnArg::Receiver(_) => None,
            }),
        )
    }

    fn lift_receiver_lifetime(&mut self, replace_with: syn::Lifetime) {
        // 1. Remove lifetime from the receiver or return, if there is no
        //    reference.
        let self_lifetime = match self.inputs.first_mut() {
            Some(syn::FnArg::Receiver(rec)) => {
                if rec.reference.is_none() {
                    return;
                }

                if let syn::Type::Reference(r) = rec.ty.as_ref() {
                    rec.ty = r.elem.clone();
                }

                rec.mutability = None;
                rec.reference.take().and_then(|(_, l)| l)
            }
            Some(syn::FnArg::Typed(_)) | None => return,
        };

        // 2. Replace receiver's lifetime and `'_` with `replace_with` in
        //    signature's output.
        let mut replacer = ReplaceLifetimes {
            replaced: [self_lifetime, Some(parse_quote! { '_ })]
                .into_iter()
                .flatten()
                .collect(),
            replace_with: &replace_with,
            matched: 0,
        };
        replacer.visit_return_type_mut(&mut self.output);

        // 3. Replace receiver's lifetime with `replace_with` in entire
        //    signature.
        _ = replacer.replaced.remove(&parse_quote! { '_ });
        if !replacer.replaced.is_empty() {
            replacer.visit_signature_mut(self);
        }

        // 4. Remove `replace_with` lifetime from generic parameters.
        let generic_params_len_before = self.generics.params.len();
        self.generics.params = mem::take(&mut self.generics.params)
            .into_iter()
            .filter(|par| match par {
                syn::GenericParam::Lifetime(syn::LifetimeParam {
                    lifetime,
                    ..
                }) => lifetime != replacer.replace_with,
                syn::GenericParam::Type(_) | syn::GenericParam::Const(_) => {
                    true
                }
            })
            .collect();

        // 5. In case `self_lifetime` came from trait generics, we add bound
        //    `#replace_with: #self_lifetime` and
        //    `#self_lifetime: #replace_with` to indicate, that they are
        //    identical.
        if let Some(self_lt) = replacer.replaced.iter().next() {
            if self.generics.params.len() == generic_params_len_before {
                let replace_lt = &replacer.replace_with;
                let predicates: [syn::WherePredicate; 2] = [
                    parse_quote! { #self_lt: #replace_lt },
                    parse_quote! { #replace_lt: #self_lt },
                ];
                self.generics
                    .make_where_clause()
                    .predicates
                    .extend(predicates);
            }
        }

        // 6. Insert `replace_with` after every reference without a lifetime in
        //    signature's output.
        InsertLifetime {
            inserted: replacer.replace_with,
        }
        .visit_return_type_mut(&mut self.output);
    }

    #[expect(clippy::renamed_function_params, reason = "more readable")]
    fn expand_lifetimes<F>(&mut self, return_lt: syn::Lifetime, expand_fn: F)
    where
        F: FnMut() -> syn::Lifetime,
    {
        // 1. Get or expand receiver lifetime.
        let receiver_lt = match self.inputs.first_mut() {
            Some(syn::FnArg::Receiver(rec)) => {
                if let Some((_, lt)) = &mut rec.reference {
                    Some(&*lt.get_or_insert_with(|| {
                        self.generics.params.push(parse_quote! { #return_lt });
                        return_lt.clone()
                    }))
                } else {
                    None
                }
            }
            Some(syn::FnArg::Typed(_)) | None => return,
        };

        // 2. Replace `'_` with `receiver_lt` or `return_lt` in signature's
        //    output.
        let mut replacer = ReplaceLifetimes {
            replaced: iter::once(parse_quote! { '_ }).collect(),
            replace_with: receiver_lt.unwrap_or(&return_lt),
            matched: 0,
        };
        replacer.visit_return_type_mut(&mut self.output);

        // 3. If no receiver's lifetime, create return type's lifetime.
        if replacer.matched > 0 && receiver_lt.is_none() {
            self.generics.params.push(parse_quote! { #return_lt });
        }

        // 4. Insert `replace_with` after every reference without a lifetime in
        //    signature's output.
        InsertLifetime {
            inserted: replacer.replace_with,
        }
        .visit_return_type_mut(&mut self.output);

        // 5. Expand every elided lifetime in whole signature.
        let mut expander = ExpandLifetime {
            expand_fn,
            expanded: vec![],
        };

        if return_lt.ident != "_" {
            expander.visit_return_type_mut(&mut self.output);
        }

        for arg in self.inputs.iter_mut().skip(1) {
            expander.visit_fn_arg_mut(arg);
        }

        // 6. Add expanded lifetimes to generic parameters.
        self.generics.params.extend(expander.expanded.iter().map(
            |lt| -> syn::GenericParam {
                parse_quote! { #lt }
            },
        ));
    }

    fn return_type(&self) -> syn::Type {
        match &self.output {
            syn::ReturnType::Default => parse_quote! { () },
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        }
    }

    fn to_be_early_bounded_lifetimes(&self) -> HashSet<syn::Lifetime> {
        /// Collector of the [`Lifetime`]s.
        ///
        /// [`Lifetime`]: struct@syn::Lifetime
        struct CollectLifetimes {
            /// Collected [`Lifetime`]s.
            ///
            /// [`Lifetime`]: struct@syn::Lifetime
            lifetimes: HashSet<syn::Lifetime>,
        }

        impl<'ast> Visit<'ast> for CollectLifetimes {
            fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {
                if i.ident != "_" {
                    _ = self.lifetimes.insert(i.clone());
                }
            }
        }

        /// Remove the [`Lifetime`]s that are early bounded. I.e. `'a: 'a`.
        ///
        /// [`Lifetime`]: struct@syn::Lifetime
        struct RemoveEarlyBoundedLifetimes<'a>(&'a mut HashSet<syn::Lifetime>);

        impl<'ast> Visit<'ast> for RemoveEarlyBoundedLifetimes<'_> {
            fn visit_lifetime_param(&mut self, i: &'ast syn::LifetimeParam) {
                if i.bounds.iter().any(|b| *b == i.lifetime) {
                    _ = self.0.remove(&i.lifetime);
                }
            }

            fn visit_predicate_lifetime(
                &mut self,
                i: &'ast syn::PredicateLifetime,
            ) {
                if i.bounds.iter().any(|b| *b == i.lifetime) {
                    _ = self.0.remove(&i.lifetime);
                }
            }
        }

        /// Remover of the [`Lifetime`]s from the provided set.
        ///
        /// [`Lifetime`]: struct@syn::Lifetime
        struct RemoveLifetimes<'a>(&'a mut HashSet<syn::Lifetime>);

        impl<'ast> Visit<'ast> for RemoveLifetimes<'_> {
            fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {
                _ = self.0.remove(i);
            }
        }

        let mut collector = CollectLifetimes {
            lifetimes: HashSet::new(),
        };

        // 1. Collect all lifetimes occurring in the arguments.
        for arg in &self.inputs {
            match arg {
                syn::FnArg::Receiver(_) => {}
                syn::FnArg::Typed(syn::PatType { ty, .. }) => {
                    collector.visit_type(ty);
                }
            }
        }

        // 2. Remove receiver's lifetime from the collected set.
        if let Some(syn::FnArg::Receiver(syn::Receiver {
            reference: Some((_, Some(lt))),
            ..
        })) = self.inputs.first()
        {
            if lt.ident != "_" {
                _ = collector.lifetimes.remove(lt);
            }
        };

        // 3. Remove lifetimes defined in trait's generics.
        let method_lifetimes =
            self.generics
                .params
                .iter()
                .filter_map(|p| match p {
                    syn::GenericParam::Lifetime(lt) => Some(&lt.lifetime),
                    syn::GenericParam::Const(_)
                    | syn::GenericParam::Type(_) => None,
                })
                .cloned()
                .collect::<HashSet<_>>();
        collector
            .lifetimes
            .retain(|lt| method_lifetimes.contains(lt));

        // 4. Remove lifetimes that are early bounded.
        RemoveEarlyBoundedLifetimes(&mut collector.lifetimes)
            .visit_generics(&self.generics);

        // 5. Remove lifetimes occurring in the return type.
        RemoveLifetimes(&mut collector.lifetimes)
            .visit_return_type(&self.output);

        collector.lifetimes
    }
}

/// [`Iterator`] over [`Signature`]'s inputs, excluding its [`Receiver`].
///
/// [`Receiver`]: syn::Receiver
/// [`Signature`]: syn::Signature
pub(super) type InputsWithoutReceiverIter<'a> = iter::FilterMap<
    punctuated::Iter<'a, syn::FnArg>,
    fn(&'a syn::FnArg) -> Option<&'a Box<syn::Pat>>,
>;

/// Replacer of the `replaced` [`Lifetime`] with the `replace_with` one.
///
/// [`Lifetime`]: struct@syn::Lifetime
struct ReplaceLifetimes<'r> {
    /// [`Lifetime`] to be replaced.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    replaced: HashSet<syn::Lifetime>,

    /// [`Lifetime`] to replace with.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    replace_with: &'r syn::Lifetime,

    /// Count of replaced [`Lifetime`]s.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    matched: usize,
}

impl VisitMut for ReplaceLifetimes<'_> {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        if self.replaced.contains(i) {
            *i = self.replace_with.clone();
            self.matched += 1;
        }

        visit_mut::visit_lifetime_mut(self, i);
    }
}

/// Inserter of a [`Lifetime`] after every empty reference.
///
/// [`Lifetime`]: struct@syn::Lifetime
struct InsertLifetime<'i> {
    /// [`Lifetime`] to be inserted.
    ///
    /// [`Lifetime`]: struct@syn::Lifetime
    inserted: &'i syn::Lifetime,
}

impl VisitMut for InsertLifetime<'_> {
    fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
        if i.lifetime.is_none() {
            i.lifetime = Some(self.inserted.clone());
        }

        visit_mut::visit_type_reference_mut(self, i);
    }
}

/// Expander of elided [`Lifetime`]s.
struct ExpandLifetime<F>
where
    F: FnMut() -> syn::Lifetime,
{
    /// Function to expand elided [`Lifetime`]s.
    expand_fn: F,

    /// Collection of [`Lifetime`]s being expanded.
    expanded: Vec<syn::Lifetime>,
}

impl<F> VisitMut for ExpandLifetime<F>
where
    F: FnMut() -> syn::Lifetime,
{
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        if i.ident == "_" {
            *i = (self.expand_fn)();
            self.expanded.push(i.clone());
        }

        visit_mut::visit_lifetime_mut(self, i);
    }

    fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
        if i.lifetime.is_none() {
            let lt = (self.expand_fn)();
            i.lifetime = Some(lt.clone());
            self.expanded.push(lt);
        }

        visit_mut::visit_type_reference_mut(self, i);
    }
}

#[cfg(test)]
mod lift_self_lifetime_spec {
    use quote::{quote, ToTokens as _};
    use syn::parse_quote;

    use super::SignatureExt;

    fn with() -> syn::Lifetime {
        parse_quote! { '__delegated }
    }

    #[test]
    fn without_receiver() {
        for (input, expected) in [(
            parse_quote! {
                fn test<'a, 'b>(arg1: &'a i32, arg2: &'b i32) -> &'a i32
            },
            quote! {
                fn test<'a, 'b>(arg1: &'a i32, arg2: &'b i32) -> &'a i32
            },
        )] {
            let mut input: syn::Signature = input;
            input.lift_receiver_lifetime(with());

            assert_eq!(
                input.to_token_stream().to_string(),
                expected.to_string(),
            );
        }
    }

    #[test]
    fn receiver_without_reference() {
        for (input, expected) in [(
            parse_quote! {
                fn test<'a>(self, arg2: &'a i32) -> &'a i32
            },
            quote! {
                fn test<'a>(self, arg2: &'a i32) -> &'a i32
            },
        )] {
            let mut input: syn::Signature = input;
            input.lift_receiver_lifetime(with());

            assert_eq!(
                input.to_token_stream().to_string(),
                expected.to_string(),
            );
        }
    }

    #[test]
    fn receiver_without_lifetime() {
        for (input, expected) in [
            (
                parse_quote! {
                    fn test<'a>(&self, arg2: &'a i32) -> &i32
                },
                quote! {
                    fn test<'a>(self, arg2: &'a i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a>(&self, arg2: &'a i32) -> &'_ i32
                },
                quote! {
                    fn test<'a>(self, arg2: &'a i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a>(&self, arg2: &'a i32) -> &'a i32
                },
                quote! {
                    fn test<'a>(self, arg2: &'a i32) -> &'a i32
                },
            ),
        ] {
            let mut input: syn::Signature = input;
            input.lift_receiver_lifetime(with());

            assert_eq!(
                input.to_token_stream().to_string(),
                expected.to_string(),
            );
        }
    }

    #[test]
    fn receiver_with_anonymous_lifetime() {
        for (input, expected) in [
            (
                parse_quote! {
                    fn test<'a>(&'_ self, arg2: &'a i32) -> &i32
                },
                quote! {
                    fn test<'a>(self, arg2: &'a i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a>(&'_ self, arg2: &'a i32) -> &'_ i32
                },
                quote! {
                    fn test<'a>(self, arg2: &'a i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a>(&'_ self, arg2: &'a i32) -> &'a i32
                },
                quote! {
                    fn test<'a>(self, arg2: &'a i32) -> &'a i32
                },
            ),
            (
                parse_quote! {
                    fn test(&'_ self, arg2: &'_ i32) -> &i32
                },
                quote! {
                    fn test(self, arg2: &'_ i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test(&'_ self, arg2: &'_ i32) -> &'_ i32
                },
                quote! {
                    fn test(self, arg2: &'_ i32) -> &'__delegated i32
                },
            ),
        ] {
            let mut input: syn::Signature = input;
            input.lift_receiver_lifetime(with());

            assert_eq!(
                input.to_token_stream().to_string(),
                expected.to_string(),
            );
        }
    }

    #[test]
    fn receiver_with_lifetime() {
        for (input, expected) in [
            (
                parse_quote! {
                    fn test<'a, 'b>(&'a self, arg2: &'b i32) -> &'a i32
                },
                quote! {
                    fn test<'b>(self, arg2: &'b i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a, 'b>(&'a self, arg2: &'b i32) -> &'b i32
                },
                quote! {
                    fn test<'b>(self, arg2: &'b i32) -> &'b i32
                },
            ),
            (
                parse_quote! {
                    fn test<'b>(&'a self, arg2: &'b i32) -> &'a i32
                },
                quote! {
                    fn test<'b>(self, arg2: &'b i32) -> &'__delegated i32
                    where
                        'a: '__delegated,
                        '__delegated: 'a
                },
            ),
            (
                parse_quote! {
                    fn test<'b>(&'a self, arg2: &'b i32) -> &'b i32
                },
                quote! {
                    fn test<'b>(self, arg2: &'b i32) -> &'b i32
                    where
                        'a: '__delegated,
                        '__delegated: 'a
                },
            ),
            (
                parse_quote! {
                    fn test<'a>(&'a self, arg2: &'b i32) -> &'b i32
                },
                quote! {
                    fn test(self, arg2: &'b i32) -> &'b i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a>(&'a self, arg2: &'b i32) -> &'a i32
                },
                quote! {
                    fn test(self, arg2: &'b i32) -> &'__delegated i32
                },
            ),
            (
                parse_quote! {
                    fn test<'a, 'b>(&'a self, arg2: &'b i32) -> &'a i32
                    where
                        'a: 'b
                },
                quote! {
                    fn test<'b>(self, arg2: &'b i32) -> &'__delegated i32
                    where
                        '__delegated: 'b
                },
            ),
            (
                parse_quote! {
                    fn test<'b>(&'a self, arg2: &'b i32) -> &'a i32
                    where
                        'a: 'b
                },
                quote! {
                    fn test<'b>(self, arg2: &'b i32) -> &'__delegated i32
                    where
                        '__delegated: 'b,
                        'a: '__delegated,
                        '__delegated: 'a
                },
            ),
        ] {
            let mut input: syn::Signature = input;
            input.lift_receiver_lifetime(with());

            assert_eq!(
                input.to_token_stream().to_string(),
                expected.to_string(),
            );
        }
    }
}
