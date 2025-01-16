use delegation_codegen::delegate;

#[delegate(derive(for<> AsRef<str>))]
pub enum Name {
    First(String),
}

fn main() {
    unreachable!()
}
