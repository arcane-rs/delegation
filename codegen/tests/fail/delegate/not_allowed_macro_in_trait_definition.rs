use delegation::delegate;

#[delegate]
trait Channel {
    unreachable!();
}

fn main() {}
