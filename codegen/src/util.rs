//! Utilities for code generation.

#[cfg(doc)]
use syn::{parse::Parse, Generics, WhereClause, WherePredicate};
use syn::{
    parse::{discouraged::Speculative as _, ParseStream},
    punctuated::Punctuated,
    token,
};

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

/// Extension of [`WhereClause`] for code generation.
pub(crate) trait WhereClauseExt: Sized {
    /// Parses an [`Option`]al [`WhereClause`] from the provided [`ParseStream`]
    /// in a thrifty (non-greedy) manner, meaning that if the last
    /// [`WherePredicate`] fails to parse, then stops parsing instead of erring.
    ///
    /// This function allows to parse multiple [`WhereClause`]s separated by
    /// comma, which its [`Parse`] implementation is not capable of because of
    /// acting greedy.
    fn parse_thrifty_opt(input: ParseStream<'_>) -> syn::Result<Option<Self>>;
}

impl WhereClauseExt for syn::WhereClause {
    fn parse_thrifty_opt(input: ParseStream<'_>) -> syn::Result<Option<Self>> {
        let Some(where_token) = input.parse()? else {
            return Ok(None);
        };

        let mut predicates = Punctuated::new();
        predicates.push_value(input.parse()?);
        loop {
            if input.is_empty()
                || input.peek(token::Brace)
                || input.peek(token::Semi)
                || input.peek(token::Colon) && !input.peek(token::PathSep)
                || input.peek(token::Eq)
            {
                break;
            }

            let ahead = input.fork();
            let Ok(comma) = ahead.parse::<token::Comma>() else {
                break;
            };
            let Ok(predicate) = ahead.parse::<syn::WherePredicate>() else {
                break;
            };
            predicates.push_punct(comma);
            predicates.push_value(predicate);
            input.advance_to(&ahead);
        }

        Ok(Some(Self { where_token, predicates }))
    }
}
