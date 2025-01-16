use criterion::{criterion_group, criterion_main, Criterion};
use delegation::delegate;

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

macro_rules! def_enum {
    ($($variant:ident),+) => {
        #[delegate(derive(AsStr))]
        enum Name {
            $($variant(String)),+
        }
    };
}

macro_rules! def_bench {
    ($($variant:ident),*) => {
        fn bench(bencher: &mut ::criterion::Bencher) {
            $(
                bencher.iter(|| {
                    let name = ::criterion::black_box(
                        Name::$variant(String::from("Test"))
                    );

                    assert_eq!(name.as_str(), "Test");
                });
            )*
        }
    };
}

macro_rules! define {
    ($($variant:ident),*) => {
        def_enum!($($variant),*);
        def_bench!($($variant),*);
    };
}

define!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y,
    Z
);

fn delegate_benchmark(c: &mut Criterion) {
    c.bench_function("delegate", bench);
}

criterion_group!(benches, delegate_benchmark);
criterion_main!(benches);
