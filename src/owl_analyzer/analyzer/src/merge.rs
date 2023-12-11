pub trait Merge<Rhs = Self> {
    fn merge(&mut self, other: &Rhs);

    fn merge_own(&mut self, other: Rhs) {
        self.merge(&other)
    }
}
