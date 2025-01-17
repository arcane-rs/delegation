//! [`MacroPath`] definitions.

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, ToTokens};

// TODO: Re-impl once https://github.com/rust-lang/rust/issues/54363 is
//       resolved.
/// Path to the macro definitions.
#[derive(Clone, Debug)]
pub(crate) struct MacroPath {
    /// Identifier of the crate the macro is defined in.
    crate_name: syn::Ident,

    /// Identifier of the module the macro definitions are located in.
    module_name: syn::Ident,
}

impl MacroPath {
    /// Name of the crate the macro is defined in.
    const CRATE_NAME: &'static str = "delegation";

    /// Name of the module the macro definitions are located in.
    const MODULE_NAME: &'static str = "__macros";
}

impl Default for MacroPath {
    fn default() -> Self {
        let crate_name = crate_name(Self::CRATE_NAME)
            .unwrap_or_else(|_err| unreachable!("can't find macro definition"));
        let crate_name = match &crate_name {
            FoundCrate::Name(name) => name.as_str(),
            FoundCrate::Itself => Self::CRATE_NAME,
        };

        Self {
            crate_name: syn::Ident::new(crate_name, Span::call_site()),
            module_name: syn::Ident::new(Self::MODULE_NAME, Span::call_site()),
        }
    }
}

impl ToTokens for MacroPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = &self.crate_name;
        let module_name = &self.module_name;
        quote! { ::#crate_name ::#module_name }.to_tokens(tokens);
    }
}
