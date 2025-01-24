use delegation_codegen::delegate;

#[delegate(derive(AsRef<str>))]
pub enum Name {
    First(String),
}

fn main() {}
