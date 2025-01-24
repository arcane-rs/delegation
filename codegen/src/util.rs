//! Utilities for code generation.

#[cfg(doc)]
use syn::{Generics, WhereClause};
use syn::parse_quote;

/// Extension trait for [`Generics`].
pub(crate) trait GenericsExt: Sized {
    /// Merges two sets of [`Generics`] and returns the resulting [`Generics`].
    fn merge(&self, other: &Option<Self>) -> Self;

    /// Extend this [`Generics`] with the provided [`WhereClause`] and returns
    /// the resulting [`Generics`].
    fn merge_where_clause(&self, c: &Option<syn::WhereClause>) -> Self;
}

impl GenericsExt for syn::Generics {
    fn merge(&self, other: &Option<Self>) -> Self {
        let mut gens = self.clone();
        if let Some(g) = other {
            gens.params.extend(g.params.iter().cloned());
            gens = gens.merge_where_clause(&g.where_clause);
        }
        gens
    }

    fn merge_where_clause(&self, c: &Option<syn::WhereClause>) -> Self {
        let mut gens = self.clone();
        if let Some(c) = c {
            let mut where_clause =
                gens.where_clause.unwrap_or_else(|| parse_quote! { where });
            where_clause.predicates.extend(c.predicates.iter().cloned());
            gens.where_clause = Some(where_clause);
        }
        gens
    }
}
