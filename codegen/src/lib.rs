//! Code generation of `#[delegate]` macro.

#![deny(
    macro_use_extern_crate,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    clippy::absolute_paths,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::as_conversions,
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
    clippy::else_if_without_else,
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
    clippy::format_push_string,
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
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::set_contains_or_insert,
    clippy::shadow_unrelated,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
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
    clippy::unneeded_field_pattern,
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
    explicit_outlives_requirements,
    future_incompatible,
    let_underscore_drop,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    redundant_lifetimes,
    semicolon_in_expressions_from_macros,
    single_use_lifetimes,
    unit_bindings,
    unnameable_types,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

mod derive;
mod impl_for;
mod impl_trait;
mod macro_path;

use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::spanned::Spanned as _;
#[cfg(test)]
use {delegation as _, trybuild as _}; // Used in integration tests.

pub(crate) use macro_path::MacroPath;

/// Derives trait on a new-type struct or enum, invoking it on its inner type.
///
/// # Examples
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
/// #
/// # fn main() {
/// let mut name = Name::First(FirstName("John".into()));
/// assert_eq!(name.as_str(), "John");
///
/// name.as_mut_str().push_str("ny");
/// assert_eq!(name.as_str(), "Johnny");
/// assert_eq!(name.into_string(), "Johnny");
/// # }
/// ```
///
/// # Generics
///
/// In some cases, trait or a type requires additional generic parameters to
/// implement delegation. For this case, macro provides `for<..>` and `where`
/// syntax for `#[delegate(derive(..))]` and `#[delegate(for(..))]` attribute
/// arguments. Specified generics will replace existing, provided by
/// a trait/type definition. To remove generics when all types are known use
/// `for<>`.
///
/// ## Example
///
/// ```rust
/// # use delegation::delegate;
/// #
/// #[delegate]
/// trait AsInner<T: ?Sized> {
///     fn as_inner(&self) -> &T;
/// }
///
/// impl AsInner<str> for String {
///     fn as_inner(&self) -> &str {
///         self
///     }
/// }
///
/// #[delegate(derive(for<> AsInner<str>))]
/// struct FirstName(String);
///
/// #[delegate(derive(
///     for<I> AsInner<str>
///     where
///         I: AsInner<str> + 'static;
/// ))]
/// struct NickName<I>(I);
/// #
/// # fn main() {
/// let first = FirstName("John".into());
/// assert_eq!(first.as_inner(), "John");
/// let last = NickName::<FirstName>(first);
/// assert_eq!(last.as_inner(), "John");
/// # }
/// ```
///
/// # External types
///
/// Because of both sides of the delegation should be marked with `#[delegate]`,
/// it's impossible to make external type delegatable. For handle this,
/// the macro provides `#[delegate(as = my::Def)]` attribute argument for
/// struct fields and enum variants. It uses provided type as known declaration
/// of some external type. Provided type should be crate-local, marked with
/// `#[delegate]` and provides infallible conversion from external type
/// (including reference-to-reference).
///
/// ## Example
///
/// ```rust
/// # use delegation::{__macros::Either, delegate};
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
/// #
/// # fn main() {
/// let left = EitherString(Either::Left("left".to_string()));
/// let right = EitherString(Either::Right("right".to_string()));
/// assert_eq!(left.as_str(), "left");
/// assert_eq!(right.as_str(), "right");
/// # }
/// ```
///
/// # External traits
///
/// Because of both sides of the delegation should be marked with `#[delegate]`,
/// it's impossible to make external trait delegatable. For handle this,
/// the macro provides `#[delegate(as = my::Def)]` attribute argument for
/// traits. It uses provided trait as known declaration of some external trait.
/// With this argument, macro will generate wrapper type that implements
/// external trait on it, with the name of expanded "declaration" trait. By
/// using this wrapper type in `#[delegate(derive(ext::Trait as my::TraitDef))]`
/// argument, you can delegate external trait to your type.
///
/// ## Example
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
///     for<> AsRef<str> as AsRefDef;
///     AsStr as AsStrDef;
/// ))]
/// enum Name {
///     First(String),
/// }
/// #
/// # fn main() {
/// let name = Name::First("John".to_string());
/// assert_eq!(name.as_ref(), "John");
/// assert_eq!(name.as_str(), "John");
/// # }
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
///   (See [rust-lang/rust#87803]).
/// - `Self` type is limited to be used in methods return types.
///
/// [rust-lang/rust#87803]: https://github.com/rust-lang/rust/issues/87803
#[proc_macro_attribute]
pub fn delegate(
    attr_args: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(attr_args.into(), &body.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Implements delegated trait for provided type.
///
/// Actually, this macro is called by `macro_rules!` expanded by
/// [`macro@delegate`] macro and only filling implementation template generated
/// by it.
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
fn expand(args: TokenStream, input: &TokenStream) -> syn::Result<TokenStream> {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "too much variants to handle"
    )]
    let tokens = match syn::parse2::<syn::Item>(input.clone())? {
        syn::Item::Enum(item) => {
            derive::Definition::parse_enum(item, args)?.into_token_stream()
        }
        syn::Item::Struct(item) => {
            derive::Definition::parse_struct(item, args)?.into_token_stream()
        }
        syn::Item::Trait(item) => {
            impl_trait::Definition::parse(item, args)?.into_token_stream()
        }
        item => {
            return Err(syn::Error::new(
                item.span(),
                "allowed only on enums, structs and traits",
            ))
        }
    };

    Ok(tokens.into_token_stream())
}
