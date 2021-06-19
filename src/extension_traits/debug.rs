pub trait Dbg: std::fmt::Debug {
    fn dbg(self) -> Self;
}

impl<T: std::fmt::Debug> Dbg for T {
    fn dbg(self) -> Self {
        println!("{:?}", &self);
        self
    }
}
