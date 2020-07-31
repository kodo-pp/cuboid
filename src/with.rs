pub trait With<T> {
    type Output;

    fn with(self, item: T) -> Self::Output;
}
