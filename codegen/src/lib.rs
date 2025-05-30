#![cfg_attr(any(doc, test), doc = include_str!("../README.md"))]
#![cfg_attr(not(any(doc, test)), doc = env!("CARGO_PKG_NAME"))]
#![deny(nonstandard_style, rustdoc::all, trivial_casts, trivial_numeric_casts)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    clippy::absolute_paths,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::as_conversions,
    clippy::as_pointer_underscore,
    clippy::as_ptr_cast_mut,
    clippy::assertions_on_result_states,
    clippy::branches_sharing_code,
    clippy::cfg_not_test,
    clippy::clear_with_drain,
    clippy::clone_on_ref_ptr,
    clippy::collection_is_never_read,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::decimal_literal_representation,
    clippy::default_union_representation,
    clippy::derive_partial_eq_without_eq,
    clippy::doc_include_without_cfg,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::equatable_if_let,
    clippy::empty_enum_variants_with_brackets,
    clippy::exit,
    clippy::expect_used,
    clippy::fallible_impl_from,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::imprecise_flops,
    clippy::infinite_loop,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_over_hash_type,
    clippy::iter_with_drain,
    clippy::large_include_file,
    clippy::large_stack_frames,
    clippy::let_underscore_untyped,
    clippy::literal_string_with_formatting_args,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::map_with_unused_argument_over_ranges,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::missing_const_for_fn,
    clippy::missing_docs_in_private_items,
    clippy::module_name_repetitions,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::needless_collect,
    clippy::needless_pass_by_ref_mut,
    clippy::needless_raw_strings,
    clippy::non_zero_suggestions,
    clippy::nonstandard_macro_braces,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pathbuf_init_then_push,
    clippy::pedantic,
    clippy::precedence_bits,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::pub_without_shorthand,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::read_zero_byte_vec,
    clippy::redundant_clone,
    clippy::redundant_type_annotations,
    clippy::renamed_function_params,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_and_then,
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::set_contains_or_insert,
    clippy::shadow_unrelated,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::single_option_map,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_lit_chars_any,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::todo,
    clippy::too_long_first_doc_paragraph,
    clippy::trailing_empty_array,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::uninhabited_references,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unnecessary_struct_initialization,
    clippy::unused_peekable,
    clippy::unused_result_ok,
    clippy::unused_trait_names,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_file_reads,
    clippy::while_float,
    clippy::wildcard_enum_match_arm,
    ambiguous_negative_literals,
    closure_returning_async_block,
    future_incompatible,
    impl_trait_redundant_captures,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    redundant_lifetimes,
    rust_2018_idioms,
    single_use_lifetimes,
    unit_bindings,
    unnameable_types,
    unreachable_pub,
    unstable_features,
    unused,
    variant_size_differences
)]

mod derive;
mod impl_for;
mod impl_trait;
mod macro_path;
pub(crate) mod util;

#[cfg(test)]
#[doc(hidden)]
mod used_only_in_integrations_tests {
    use delegation as _;
    use rustversion as _;
    use trybuild as _;
}

use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::spanned::Spanned as _;

use self::macro_path::MacroPath;

/// Derives trait on a new-type struct or enum, invoking it on its inner type.
///
/// # Example
///
/// ```rust
/// # use delegation::delegate;
/// #
/// #[delegate(derive(AsString))]
/// enum Name {
///     First(FirstName),
///     Last(LastName),
/// }
///
/// #[delegate(derive(AsString))]
/// struct FirstName(String);
///
/// #[delegate]
/// struct LastName(String);
///
/// #[delegate(for(LastName))]
/// trait AsString {
///     fn into_string(self) -> String;
///     fn as_str(&self) -> &str;
///     fn as_mut_str(&mut self) -> &mut String;
/// }
///
/// impl AsString for String {
///     fn into_string(self) -> Self {
///         self
///     }
///     fn as_str(&self) -> &str {
///         self.as_str()
///     }
///     fn as_mut_str(&mut self) -> &mut Self {
///         self
///     }
/// }
///
/// let mut name = Name::First(FirstName("John".into()));
/// assert_eq!(name.as_str(), "John");
///
/// name.as_mut_str().push_str("ny");
/// assert_eq!(name.as_str(), "Johnny");
/// assert_eq!(name.into_string(), "Johnny");
/// ```
///
/// # Generics
///
/// In some cases, a trait or a type requires additional generic parameters to
/// implement delegation. For this case, macro provides `for<..>` and `where`
/// syntax for `#[delegate(derive(..))]`/`#[delegate(for(..))]` attribute
/// arguments. Specified generics will be merged with the existing ones,
/// provided by the trait/type definition.
///
/// ```rust
/// # use delegation::delegate;
/// #
/// #[delegate(for(
///     for<U> Case2<U>
///     where
///         U: Named<N> + 'static,
/// ))]
/// trait Named<N> {
///     fn name(&self) -> N;
/// }
///
/// struct User(String);
/// impl Named<String> for User {
///     fn name(&self) -> String {
///         self.0.clone()
///     }
/// }
///
/// #[delegate(derive(
///     for<N> Named<N>
///     where
///         U: Named<N> + 'static,
/// ))]
/// enum Case1<U> {
///     User(U),
/// }
///
/// #[delegate]
/// struct Case2<U>(U);
///
/// #[delegate(derive(
///    Named<String>
///    where
///        U: Named<String> + 'static,
/// ))]
/// enum Case3<U> {
///     Case1(Case1<U>),
///     Case2(Case2<U>),
/// }
///
/// let user1 = Case1::User(User("Alice".to_string()));
/// assert_eq!(user1.name(), "Alice");
///
/// let user2 = Case2(User("Bob".to_string()));
/// assert_eq!(user2.name(), "Bob");
///
/// let user3 = Case3::Case1(Case1::User(User("Charlie".to_string())));
/// assert_eq!(user3.name(), "Charlie");
/// ```
///
/// # External types
///
/// Because the both sides of the delegation should be marked with the
/// `#[delegate]` attribute, it's impossible to make external type delegatable.
/// To handle this, the macro provides the `#[delegate(as = my::Def)]`
/// attribute argument for struct fields and enum variants. It uses the provided
/// type as known declaration of some external type. Provided type should be
/// crate-local, and marked with the `#[delegate]` macro, and to provide an
/// infallible conversion from external type (including reference-to-reference
/// one).
///
/// ```rust
/// # use delegation::{private::Either, delegate};
/// #
/// #[delegate]
/// trait AsStr {
///     fn as_str(&self) -> &str;
/// }
///
/// impl AsStr for String {
///     fn as_str(&self) -> &str {
///         self
///     }
/// }
///
/// #[delegate(derive(AsStr))]
/// enum EitherDef {
///     Left(String),
///     Right(String),
/// }
///
/// impl<'a> From<&'a mut Either<String, String>> for &'a mut EitherDef {
///     fn from(t: &'a mut Either<String, String>) -> Self {
///         #[expect(unsafe_code, reason = "macro expansion")]
///         unsafe {
///             &mut *(t as *mut Either<String, String> as *mut EitherDef)
///         }
///     }
/// }
///
/// impl<'a> From<&'a Either<String, String>> for &'a EitherDef {
///     fn from(t: &'a Either<String, String>) -> Self {
///         #[expect(unsafe_code, reason = "macro expansion")]
///         unsafe {
///             &*(t as *const Either<String, String> as *const EitherDef)
///         }
///     }
/// }
///
/// impl From<Either<String, String>> for EitherDef {
///     fn from(t: Either<String, String>) -> Self {
///         match t {
///             Either::Left(t) => EitherDef::Left(t),
///             Either::Right(t) => EitherDef::Right(t),
///         }
///     }
/// }
///
/// #[delegate(derive(AsStr))]
/// struct EitherString(#[delegate(as = EitherDef)] Either<String, String>);
///
/// let left = EitherString(Either::Left("left".to_string()));
/// let right = EitherString(Either::Right("right".to_string()));
/// assert_eq!(left.as_str(), "left");
/// assert_eq!(right.as_str(), "right");
/// ```
///
/// # External traits
///
/// Because the both sides of the delegation should be marked with the
/// `#[delegate]` attribute, it's impossible to make an external trait
/// delegatable. To handle this, the macro provides the
/// `#[delegate(as = my::Def)]` attribute argument for traits. It uses the
/// provided trait as known declaration of some external trait. With this
/// argument, the macro will generate a wrapper type implementing the external
/// trait on it, with the name of the expanded "declaration" trait. By using
/// this wrapper type in `#[delegate(derive(ext::Trait as my::TraitDef))]`
/// argument, you can delegate external trait to your type.
///
/// ```rust
/// # use delegation::delegate;
/// #
/// #[delegate(as = AsRef)]
/// trait AsRefDef<T: ?Sized> {
///     fn as_ref(&self) -> &T;
/// }
///
/// #[delegate]
/// trait AsStr {
///     fn as_str(&self) -> &str;
/// }
///
/// impl AsStr for String {
///     fn as_str(&self) -> &str {
///         self
///     }
/// }
///
/// #[delegate(as = AsStr)]
/// trait AsStrDef {
///     fn as_str(&self) -> &str;
/// }
///
/// #[delegate(derive(
///     AsRef<str> as AsRefDef,
///     AsStr as AsStrDef,
/// ))]
/// enum Name {
///     First(String),
/// }
///
/// let name = Name::First("John".to_string());
/// assert_eq!(name.as_ref(), "John");
/// assert_eq!(name.as_str(), "John");
/// ```
///
/// # Limitations
///
/// - Both struct/enum and trait should be marked with `#[delegate]` macro
///   attribute.
/// - Struct or enum variant should contain only single field.
/// - Trait methods must have an untyped receiver.
/// - Supertraits or `Self` trait/method bounds except marker traits like
///   [`Sized`], [`Send`] or [`Sync`] are not supported yet.
/// - Associated types/constants are not supported yet.
/// - Lifetimes in methods are limited to be early-bounded in some cases
///   (see [rust-lang/rust#87803]).
/// - `Self` type is limited to be used in methods return types.
///
/// [rust-lang/rust#87803]: https://github.com/rust-lang/rust/issues/87803
#[proc_macro_attribute]
pub fn delegate(
    attr_args: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(attr_args.into(), body.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Implements a delegated trait for the provided type.
///
/// Actually, this macro is called by `macro_rules!` in the expansion of the
/// [`delegate`] macro, and only fills an implementation template generated by
/// it.
///
/// [`delegate`]: macro@delegate
// TODO: Replace this with flat declarative macro, generated by `#[delegate]`,
//       once `macro_rules!` can handle generics easily.
#[proc_macro]
pub fn impl_for(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse::<impl_for::Definition>(input)
        .map_or_else(
            |e| e.to_compile_error(),
            quote::ToTokens::into_token_stream,
        )
        .into()
}

/// Expands `#[delegate]` macro on the provided `input`.
fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<syn::Item>(input)?;
    let tokens = match item {
        syn::Item::Enum(item) => {
            derive::Definition::parse_enum(item, args)?.into_token_stream()
        }
        syn::Item::Struct(item) => {
            derive::Definition::parse_struct(item, args)?.into_token_stream()
        }
        syn::Item::Trait(item) => {
            impl_trait::Definition::parse(item, args)?.into_token_stream()
        }
        syn::Item::Const(_)
        | syn::Item::ExternCrate(_)
        | syn::Item::Fn(_)
        | syn::Item::ForeignMod(_)
        | syn::Item::Impl(_)
        | syn::Item::Macro(_)
        | syn::Item::Mod(_)
        | syn::Item::Static(_)
        | syn::Item::TraitAlias(_)
        | syn::Item::Type(_)
        | syn::Item::Union(_)
        | syn::Item::Use(_)
        | syn::Item::Verbatim(_) => {
            return Err(syn::Error::new(
                item.span(),
                "allowed only on enums, structs and traits",
            ));
        }
        item => {
            return Err(syn::Error::new(
                item.span(),
                format!("unknown `syn::Item`: {item:?}"),
            ));
        }
    };

    Ok(tokens.into_token_stream())
}
