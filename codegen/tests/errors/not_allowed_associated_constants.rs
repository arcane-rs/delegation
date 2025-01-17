use delegation::delegate;

#[delegate]
trait Channel {
    const ID: usize;
}

fn main() {
    unreachable!()
}
