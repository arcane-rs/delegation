use delegation::delegate;

#[delegate(derive(AsStr))]
trait AsStr {
    fn as_str(&self) -> &str;
}

#[delegate]
trait AsString {
    fn as_str(&self) -> &str;
}

#[delegate(for(AsString))]
struct FirstName(String);

#[delegate(derive(AsString))]
struct MiddleName(#[delegate(derive(AsString))] String);

#[delegate(derive(AsString))]
struct NickName(#[delegate(for(AsString))] String);

#[delegate(derive(AsString))]
enum OlegName {
    #[delegate(derive(AsString))]
    First(String),
}

#[delegate(derive(AsString))]
enum BorisFullName {
    #[delegate(for(AsString))]
    FirstAndLast(String),
}

#[delegate(derive(AsString))]
enum Name {
    First(#[delegate(derive(AsString))] String),
}

#[delegate(derive(AsString))]
enum FullName {
    FirstAndLast(#[delegate(for(AsString))] String),
}

#[delegate]
enum AliasName {
    Alias(#[delegate(as = SomeType)] String),
}

#[delegate(as = SomeType)]
enum UserName {
    Name(String),
}

#[delegate(as = SomeType)]
struct MemberName(String);

fn main() {
    unreachable!()
}
