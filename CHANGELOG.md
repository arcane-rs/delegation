`delegation` changelog
======================

All user visible changes to this project will be documented in this file. This project uses [Semantic Versioning 2.0.0].




## [0.1.0] - 2025-??-?? ~TBD
[0.1.0]: /../../tree/v0.1.0

### Added

- Codegen:
    - `delegate` macro (#1).
- Machinery:
    - `Either` and `Void` enums (#1);
    - `External` and `Wrapper` structs (#1);
    - `Convert` and `TypeOf` traits (#1).

### Added

- `#[delegate]` macro: ([#1])
    - Single-fielded structs support.
    - Enums with single-fielded variants support.
    - Limited generics support.
    - External types support.
    - External traits support.

[#1]: https://github.com/arcane-rs/delegation/pull/1




[Semantic Versioning 2.0.0]: https://semver.org
