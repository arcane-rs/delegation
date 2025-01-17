`delegation` crate
==================

[![crates.io](https://img.shields.io/crates/v/delegation.svg "crates.io")](https://crates.io/crates/delegation)
[![Unsafe Forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden")](https://github.com/rust-secure-code/safety-dance)  
[![CI](https://github.com/arcane-rs/delegation/workflows/CI/badge.svg?branch=main "CI")](https://github.com/arcane-rs/delegation/actions?query=workflow%3ACI+branch%3Amain)
[![Rust docs](https://docs.rs/delegation/badge.svg "Rust docs")](https://docs.rs/delegation)

[API Docs](https://docs.rs/delegation) |
[Changelog](https://github.com/arcane-rs/delegation/blob/main/CHANGELOG.md)

Macro-based delegation for enums and structs.

> **NOTE**: This crate is a hard fork and THE successor of the [unreleased `enum_delegate` 0.3 crate rewrite](https://gitlab.com/dawn_app/enum_delegate/-/compare/0.2.0...41957162), which fell to be [unmaintained](https://gitlab.com/dawn_app/enum_delegate/-/issues/14#note_2247359443).




## How to use it

```rust
use delegation::delegate;

#[delegate(for(LastName))]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr))]
struct FirstName(String);

#[delegate]
struct LastName {
    name: String,
}

#[delegate(derive(AsStr))]
enum Name {
    First(FirstName),
    Last(LastName),
}

let name = Name::First(FirstName("John".to_string()));
assert_eq!(name.as_str(), "John");

let name = Name::Last(LastName {
    name: "Doe".to_string(),
});
assert_eq!(name.as_str(), "Doe");
```




## Generics

In some cases, a trait or a type requires additional generic parameters to implement delegation. For this case, macro provides `for<..>` and `where` syntax for `#[delegate(derive(..))]` and `#[delegate(for(..))]` attribute arguments. Specified generics will replace existing, provided by the trait/type definition. To remove generics when all types are known use `for<>`.

```rust
use delegation::delegate;

#[delegate]
trait AsInner<T: ?Sized> {
    fn as_inner(&self) -> &T;
}

impl AsInner<str> for String {
    fn as_inner(&self) -> &str {
        self
    }
}

#[delegate(derive(for<> AsInner<str>))]
struct FirstName(String);

#[delegate(derive(
    for<I> AsInner<str>
    where
        I: AsInner<str> + 'static;
))]
struct NickName<I>(I);

let first = FirstName("John".into());
assert_eq!(first.as_inner(), "John");

let last = NickName::<FirstName>(first);
assert_eq!(last.as_inner(), "John");
```




## External types

Because the both sides of the delegation should be marked with the `#[delegate]` attribute, it's impossible to make external type delegatable. To handle this, the macro provides the `#[delegate(as = "my::Def")]` attribute argument for struct fields and enum variants. It uses the provided type as known declaration of some external type. Provided type should be crate-local, and marked with the `#[delegate]` macro, and to provide an infallible conversion from external type (including reference-to-reference one).

```rust
use delegation::{
    private::Either, // non-public, but OK for showcase.
    delegate
};

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr))]
enum EitherDef {
    Left(String),
    Right(String),
}

impl<'a> From<&'a mut Either<String, String>> for &'a mut EitherDef {
    fn from(t: &'a mut Either<String, String>) -> Self {
        #[expect(unsafe_code, reason = "macro expansion")]
        unsafe {
            &mut *(t as *mut Either<String, String> as *mut EitherDef)
        }
    }
}

impl<'a> From<&'a Either<String, String>> for &'a EitherDef {
    fn from(t: &'a Either<String, String>) -> Self {
        #[expect(unsafe_code, reason = "macro expansion")]
        unsafe {
            &*(t as *const Either<String, String> as *const EitherDef)
        }
    }
}

impl From<Either<String, String>> for EitherDef {
    fn from(t: Either<String, String>) -> Self {
        match t {
            Either::Left(t) => EitherDef::Left(t),
            Either::Right(t) => EitherDef::Right(t),
        }
    }
}

#[delegate(derive(AsStr))]
struct EitherString(#[delegate(as = "EitherDef")] Either<String, String>);

let left = EitherString(Either::Left("left".to_string()));
let right = EitherString(Either::Right("right".to_string()));
assert_eq!(left.as_str(), "left");
assert_eq!(right.as_str(), "right");
```




## External traits

Because the both sides of the delegation should be marked with the `#[delegate]` attribute, it's impossible to make an external trait delegatable. To handle this, the macro provides the `#[delegate(as = "my::Def")]` attribute argument for traits. It uses the provided trait as known declaration of some external trait. With this argument, the macro will generate a wrapper type implementing the external trait on it, with the name of the expanded "declaration" trait. By using this wrapper type in `#[delegate(derive(ext::Trait as my::TraitDef))]` argument, you can delegate external trait to your type.

```rust
use delegation::delegate;

#[delegate(as = "AsRef")]
trait AsRefDef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(as = "AsStr")]
trait AsStrDef {
    fn as_str(&self) -> &str;
}

#[delegate(derive(
    for<> AsRef<str> as AsRefDef;
    AsStr as AsStrDef;
))]
enum Name {
    First(String),
}

let name = Name::First("John".to_string());
assert_eq!(name.as_ref(), "John");
assert_eq!(name.as_str(), "John");
```




## How it works

Crate provides several definitions:
- `delegate` macro - derives trait on a new-type struct or enum, invoking it on its inner type.
- `Convert` trait - converts enum or struct to a type representing "any of its variants".
- "wrapper" type - some type used as a proxy to avoid blanket impls and [orphan rules] problems.


### `#[delegate]` expansion on type

Implements the `Convert` trait for an enum/struct, which allows to convert it to "any of its variants" type.

```rust
use delegation::delegate;

#[delegate]
enum Name {
    First(FirstName),
    Last {
        name: LastName,
    },
}
```
generates
```rust,ignore
// NOTE: Simplified for readability.

impl Convert for Name {
    type Output = Either<FirstName, LastName>;

    fn convert(self) -> Self::Output {
        match self {
            Name::First(first_name) => Either::Left(first_name),
            Name::Last { name } => Either::Right(name),
        }
    }
}
```

### `#[delegate]` expansion on trait

Implements the trait for a "wrapper" type, with inner type implementing the `Convert` trait, which "any variant" implements the target trait. I.e. each method in the generated `impl` converts `self` to the "wrapper" and then to "any of its variants" and invokes the target trait method on it.

Also, it generates a declarative macro used to implement delegation of this trait for a type.

```rust
use delegation::delegate;

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}
```
generates
```rust,ignore
// NOTE: Simplified for readability.

// Implementation for "any variant of enum or struct" type.
impl<L: AsStr, R: AsStr> AsStr for Either<L, R> {
    fn as_str(&self) -> &str {
        match self {
            Either::Left(left) => left.as_str(),
            Either::Right(right) => right.as_str(),
        }
    }
}

// Implementation for a wrapper type, which inner type implements the `Convert` trait.
// Required to make external types work. In that case the `Wrapper` will be crate-local type.
impl<T> AsStr for Wrapper<T>
where
    T: Convert,
    T::Output: AsStr,
{
    fn as_str(&self) -> &str {
        let this = self.convert(); // convert type to "any of its variant".
        this.as_str() // call the method.
    }
}

// Definition of macro which implements trait for the provided type.
// It invokes when `for`/`derive` arguments are provided to the `#[delegate]` macro. 
macro_rules! AsStr {
    ($trait_path:path, $ty:ty, $wrapper:ty) => {
        impl $trait_path for $ty {
            fn as_str(&self) -> &str {
                Wrapper(self).convert().as_str()
            }
        }
    };
}
```




## Limitations

- Both struct/enum and trait should be marked with the `#[delegate]` macro attribute.
- Struct or enum variant should contain only a single field.
- Trait methods must have an untyped receiver.
- Supertraits or `Self` trait/method bounds except marker traits like `Sized`, `Send` or `Sync` are not supported yet.
- Associated types/constants are not supported yet.
- Lifetimes in methods are limited to be early-bounded in some cases (see [rust-lang/rust#87803](https://github.com/rust-lang/rust/issues/87803)).
- `Self` type is limited to be used in methods return types.




## Alternatives and similar crates


### [`enum_dispatch`]

[`delegation`] was highly inspired by the [`enum_dispatch`] crate. It provides similar functionality, but has more limitations:
- Supports only enums.
- Using [`enum_dispatch`] between crates is impossible due to limitations of its design.
- Order-dependent macro expansion (in some cases your code fails if items marked with the macro has different order than macro expects).


### [`enum_derive`]

Derives a method to return a borrowed pointer to the inner value, cast to a trait object, using the [`enum_derive::EnumInnerAsTrait`].
Slower though, more similar to [dynamic dispatch][1]. Also, less ergonomic due do usage of function-like macros.


### [`enum_delegate`]

[`delegation`] is a hard fork and THE successor of the [unreleased `enum_delegate` 0.3 crate rewrite](https://gitlab.com/dawn_app/enum_delegate/-/compare/0.2.0...41957162), which fell to be [unmaintained](https://gitlab.com/dawn_app/enum_delegate/-/issues/14#note_2247359443).




Provides trait delegation functionality for enums and structs.

Forked from unmaintained [enum_dispatch][4] crate.

```toml
[dependencies]
delegation = "0.3"
```

## Example

```rust
use delegation::delegate;

#[delegate(for(LastName))]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr))]
struct FirstName(String);

#[delegate]
struct LastName {
    name: String,
}

#[delegate(derive(AsStr))]
enum Name {
    First(FirstName),
    Last(LastName),
}

fn main() {
    let name = Name::First(FirstName("John".to_string()));
    assert_eq!(name.as_str(), "John");

    let name = Name::Last(LastName {
        name: "Doe".to_string(),
    });
    assert_eq!(name.as_str(), "Doe");
}
```


# Generics

In some cases, trait or a type requires additional generic parameters to
implement delegation. For this case, macro provides `for<..>` and `where`
syntax for `#[delegate(derive(..))]` and `#[delegate(for(..))]` attribute
arguments. Specified generics will replace existing, provided by
a trait/type definition. To remove generics when all types are known use
`for<>`.

## Example

```rust
use delegation::delegate;

#[delegate]
trait AsInner<T: ?Sized> {
    fn as_inner(&self) -> &T;
}

impl AsInner<str> for String {
    fn as_inner(&self) -> &str {
        self
    }
}

#[delegate(derive(for<> AsInner<str>))]
struct FirstName(String);

#[delegate(derive(
    for<I> AsInner<str>
    where
        I: AsInner<str> + 'static;
))]
struct NickName<I>(I);

fn main() {
    let first = FirstName("John".into());
    assert_eq!(first.as_inner(), "John");
    let last = NickName::<FirstName>(first);
    assert_eq!(last.as_inner(), "John");
}
```

# External types

Because of both sides of the delegation should be marked with `#[delegate]`,
it's impossible to make external type delegatable. For handle this,
the macro provides `#[delegate(as = my::Def)]` attribute argument for
struct fields and enum variants. It uses provided type as known declaration
of some external type. Provided type should be crate-local, marked with
`#[delegate]` and provides infallible conversion from external type
(including reference-to-reference).

## Example

```rust
use delegation::{
    __macros::Either, // non-public, but OK for showcase.
    delegate
};

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr))]
enum EitherDef {
    Left(String),
    Right(String),
}

impl<'a> From<&'a mut Either<String, String>> for &'a mut EitherDef {
    fn from(t: &'a mut Either<String, String>) -> Self {
        #[expect(unsafe_code, reason = "macro expansion")]
        unsafe {
            &mut *(t as *mut Either<String, String> as *mut EitherDef)
        }
    }
}

impl<'a> From<&'a Either<String, String>> for &'a EitherDef {
    fn from(t: &'a Either<String, String>) -> Self {
        #[expect(unsafe_code, reason = "macro expansion")]
        unsafe {
            &*(t as *const Either<String, String> as *const EitherDef)
        }
    }
}

impl From<Either<String, String>> for EitherDef {
    fn from(t: Either<String, String>) -> Self {
        match t {
            Either::Left(t) => EitherDef::Left(t),
            Either::Right(t) => EitherDef::Right(t),
        }
    }
}

#[delegate(derive(AsStr))]
struct EitherString(#[delegate(as = EitherDef)] Either<String, String>);

fn main() {
    let left = EitherString(Either::Left("left".to_string()));
    let right = EitherString(Either::Right("right".to_string()));
    assert_eq!(left.as_str(), "left");
    assert_eq!(right.as_str(), "right");
}
```

# External traits

Because of both sides of the delegation should be marked with `#[delegate]`,
it's impossible to make external trait delegatable. For handle this,
the macro provides `#[delegate(as = my::Def)]` attribute argument for
traits. It uses provided trait as known declaration of some external trait.
With this argument, macro will generate wrapper type that implements
external trait on it, with the name of expanded "declaration" trait. By
using this wrapper type in `#[delegate(derive(ext::Trait as my::TraitDef))]`
argument, you can delegate external trait to your type.

## Example

```rust
use delegation::delegate;

#[delegate(as = AsRef)]
trait AsRefDef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(as = AsStr)]
trait AsStrDef {
    fn as_str(&self) -> &str;
}

#[delegate(derive(
    for<> AsRef<str> as AsRefDef;
    AsStr as AsStrDef;
))]
enum Name {
    First(String),
}

fn main() {
    let name = Name::First("John".to_string());
    assert_eq!(name.as_ref(), "John");
    assert_eq!(name.as_str(), "John");
}
```


## How it works

Crate provides several definitions:
- `delegate` macro - derives trait on a new-type struct or enum, invoking it on its inner type.
- `Convert` trait - converts enum or struct to type represents "any of its variant".
- "wrapper" type - some type used as proxy to avoid blanket impls and [orphan rules] problems.

### `#[delegate]` expansion on type

Implements `Convert` trait to enum/struct, which allows to convert it to "any of its variant" type.

#### Source

```rust,ignore
#[delegate]
enum Name {
    First(FirstName),
    Last {
        name: LastName,
    },
}
```

#### Generated

**Note**: Example is simplified for readability.

```rust,ignore
impl Convert for Name {
    type Output = Either<FirstName, LastName>;

    fn convert(self) -> Self::Output {
        match self {
            Name::First(first_name) => Either::Left(first_name),
            Name::Last { name } => Either::Right(name),
        }
    }
}
```

### `#[delegate]` expansion on trait

Implements the trait for a "wrapper" type, with inner type implements `Convert` trait, which "any variant" implements target trait.
I.e. each method in generated `impl` convert `self` to "wrapper" and then to "any of its variant" and invokes target trait method on it.

Also it generates a declarative macro used to implement delegation of this trait for a type.

#### Source

```rust,ignore
#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}
```

#### Generated

**Note**: Example is simplified for readability.

```rust,ignore
// Implementation for "any variant of enum or struct" type.
impl<L: AsStr, R: AsStr> AsStr for Either<L, R> {
    fn as_str(&self) -> &str {
        match self {
            Either::Left(left) => left.as_str(),
            Either::Right(right) => right.as_str(),
        }
    }
}

// Implementation for a wrapper type, which inner type implements `Convert` trait.
// Required to make external types work. In that case `Wrapper` will be crate-local type.
impl<T> AsStr for Wrapper<T>
where
    T: Convert,
    T::Output: AsStr,
{
    fn as_str(&self) -> &str {
        let this = self.convert(); // convert type to "any of its variant".
        this.as_str() // call the method.
    }
}

// Definition of macro which implements trait for provided type.
// It invokes when `for`/`derive` arguments are provided to `delegate` macro. 
macro_rules! AsStr {
    ($trait_path:path, $ty:ty, $wrapper:ty) => {
        impl $trait_path for $ty {
            fn as_str(&self) -> &str {
                Wrapper(self).convert().as_str()
            }
        }
    };
}
```


## Limitations

- Both struct/enum and trait should be marked with `#[delegate]` macro attribute.
- Struct or enum variant should contain only single field.
- Trait methods must have an untyped receiver.
- Supertraits or `Self` trait/method bounds except marker traits like `Sized`, `Send` or `Sync` are not supported yet.
- Associated types/constants are not supported yet.
- Lifetimes in methods are limited to be early-bounded in some cases (See [rust-lang/rust#87803](https://github.com/rust-lang/rust/issues/87803)).
- `Self` type is limited to be used in methods return types.

## Alternatives

### [Dynamic dispatch][1]

Rust mechanism for dynamic dispatch using trait objects, which adds runtime overhead.

#### Example

```rust
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

struct FirstName(String);

impl AsStr for FirstName {
    fn as_str(&self) -> &str {
        &self.0
    }
}

fn do_something_with_string(s: &dyn AsStr) {
    println!("{}", s.as_str());
}

fn main() {
    let name = "John".to_string();
    do_something_with_string(&name);

    let name = FirstName(name);
    do_something_with_string(&name);
}
```

### [enum_dispatch][2]

`delegation` was highly inspired by [enum_dispatch][2] crate. It provides similar functionality, but has more limitations:
- Supports only enums.
- Using `enum_dispatch` between crates is impossible due to limitations of its design.
- Order-dependent macro expansion (in some cases your code fails if items marked with the macro has different order than macro expects).

### [enum_derive][3]

Derive a method to return a borrowed pointer to the inner value, cast to a trait object, using `enum_derive::EnumInnerAsTrait`.
Slower though, more similar to [Dynamic dispatch][1].




[1]: https://doc.rust-lang.org/book/ch17-02-trait-objects.html
[2]: https://docs.rs/enum_dispatch/latest/enum_dispatch
[3]: https://docs.rs/enum_derive/latest/enum_derive
[4]: https://gitlab.com/dawn_app/enum_delegate

[orphan rules]: https://rust-lang.github.io/chalk/book/clauses/coherence.html#the-orphan-rules



## License

This crate is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE] or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT] or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the [Apache-2.0 license][APACHE], shall be dual licensed as above, without any additional terms or conditions.




[`delegation`]: https://docs.rs/delegation
[`enum_delegate`]: https://docs.rs/enum_delegate
[`enum_derive`]: https://docs.rs/enum_derive
[`enum_derive::EnumInnerAsTrait`]: https://docs.rs/enum_derive/latest/enum_derive/macro.EnumInnerAsTrait.html
[`enum_dispatch`]: https://docs.rs/enum_dispatch
[1]: https://doc.rust-lang.org/book/ch17-02-trait-objects.html
[APACHE]: https://github.com/arcane-rs/delegation/blob/main/LICENSE-APACHE
[MIT]: https://github.com/arcane-rs/delegation/blob/main/LICENSE-MIT
[orphan rules]: https://rust-lang.github.io/chalk/book/clauses/coherence.html#the-orphan-rules
