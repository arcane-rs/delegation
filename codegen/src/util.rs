//! Utilities for code generation.

#[cfg(doc)]
use syn::{Generics, WhereClause};

/// Extension of [`Generics`] for code generation.
pub(crate) trait GenericsExt: Sized {
    /// Merges two sets of [`Generics`] and returns the resulting [`Generics`].
    fn merge(&self, other: Option<&Self>) -> Self;

    /// Extends these [`Generics`] with the provided [`WhereClause`] and returns
    /// the resulting [`Generics`].
    fn merge_where_clause(&self, c: Option<&syn::WhereClause>) -> Self;
}

impl GenericsExt for syn::Generics {
    fn merge(&self, other: Option<&Self>) -> Self {
        let mut gens = self.clone();
        if let Some(g) = other {
            gens.params.extend(g.params.clone());
            gens = gens.merge_where_clause(g.where_clause.as_ref());
        }
        gens
    }

    fn merge_where_clause(&self, c: Option<&syn::WhereClause>) -> Self {
        let mut gens = self.clone();
        if let Some(c) = c {
            gens.make_where_clause().predicates.extend(c.predicates.clone());
        }
        gens
    }
}
