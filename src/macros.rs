#[doc(hidden)]
pub use codegen::impl_for;

/// Enum for holding either `L` or `R` type.
#[derive(Clone, Copy, Debug)]
pub enum Either<L, R> {
    /// Left type.
    Left(L),

    /// Right type.
    Right(R),
}

/// Type of unreachable [`Either`] variant.
#[derive(Clone, Copy, Debug)]
pub enum Void {}

/// Wrapper around `T` to implement traits for delegation.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Wrapper<T: ?Sized>(pub T);

/// Type for interacting with external traits.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct External;

/// Trait for converting a type into its delegate.
pub trait Convert {
    /// Type of an owned any enum variant.
    type Owned;

    /// Type of a referenced any enum variant.
    type Ref<'a>
    where
        Self: 'a;

    /// Type of a mutable referenced any enum variant.
    type RefMut<'a>
    where
        Self: 'a;

    /// Converts this enum into an owned variant.
    fn convert_owned(self) -> Self::Owned;

    /// Converts reference to this enum into a variant reference.
    fn convert_ref(&self) -> Self::Ref<'_>;

    /// Converts mutable reference to this enum into a mutable variant
    /// reference.
    fn convert_ref_mut(&mut self) -> Self::RefMut<'_>;
}

/// Trait for retrieving an actual type from a bind type.
pub trait TypeOf {
    /// Actual type associated with the bind.
    type T;
}
