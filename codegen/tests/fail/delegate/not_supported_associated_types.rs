// TODO: Remove once associated types are supported.

use delegation::delegate;

#[delegate]
trait Parser {
    type Buffer;
}

fn main() {}
