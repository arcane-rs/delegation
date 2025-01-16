# Contributing

Improvements of any kind are welcome!

If you have any questions, please check issues first to see there is no similar question. If there is no duplicates, feel free to open a new issue.




## Formatting

Use [rustfmt][1] tool to format the code before submitting a pull request.

Some features of formatting supported only in [nightly][3] channel.
To format the code type the following in a directory of the project:

```bash
cargo +nightly fmt
```




## Lints

Use [clippy][2] tool to lint the code before submitting a pull request.
To lint the code type the following in a directory of the project:

```bash
cargo clippy
```



## Testing

Codegen tests of the macro located in `tests` directory of `delegation-codegen` crate.
To run all tests, type the following in a directory of the project:

```bash
cargo test --all-features --workspace
```

Before submitting a pull request, please make sure all tests are passed.

When adding new features, please add tests for covering them.




## Documentation

To build the documentation, type the following in a directory of the project:

```bash
cargo doc --all-features --document-private-items --workspace
```

Documentation will be generated in `target/doc` directory of the project.

When changing the code, please check its documentation is updated
and do not forget to update the `README.md` file if needed.




## Benchmarks

Benchmarks located in `benches` directory of the project.

To run benchmarks, type the following in a directory of the project:

```bash
cargo bench --workspace
```

Before submitting a pull request, please check benchmarks are not regressed.




## Changelog

All user visible changes should be documented in `CHANGELOG.md` file following [Semantic Versioning 2.0.0][4] rules.




[1]: https://github.com/rust-lang/rustfmt
[2]: https://github.com/rust-lang/rust-clippy
[3]: https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust
[4]: https://semver.org
