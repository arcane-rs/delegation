use delegation::delegate;

#[delegate]
trait Channel {
    fn id(this: &Self) -> usize;
}

fn main() {}
