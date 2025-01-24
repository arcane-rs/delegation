`delegation` changelog
======================

All user visible changes to this project will be documented in this file. This project uses [Semantic Versioning 2.0.0].




## main

[Diff](https://github.com/arcane-rs/delegation/compare/v0.3.0...main) | [Milestone](https://github.com/arcane-rs/delegation/milestone/2)

## BC Breaks

- Made `for<..>` syntax in `#[delegate(derive(..))]`/`#[delegate(for(..))]` attribute arguments only for declaring additional generic parameters not present on type/trait already. ([#10])
- Made entries in `#[delegate(derive(..))]`/`#[delegate(for(..))]` attribute arguments separated by comma instead of semicolon. ([#11])

[#10]: https://github.com/arcane-rs/delegation/pull/10
[#11]: https://github.com/arcane-rs/delegation/pull/11




## [0.3.0] · 2025-01-17
[0.3.0]: https://github.com/arcane-rs/delegation/tree/v0.3.0

[Diff](https://github.com/arcane-rs/delegation/compare/d375a898...v0.3.0) | [Milestone](https://github.com/arcane-rs/delegation/milestone/1)

### Added

- `#[delegate]` macro: ([#1])
    - Single-fielded structs support.
    - Enums with single-fielded variants support.
    - Limited generics support.
    - External types support.
    - External traits support.

[#1]: https://github.com/arcane-rs/delegation/pull/1




[Semantic Versioning 2.0.0]: https://semver.org
