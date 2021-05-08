pub trait Tensor{
    fn add(&self, other: &Self) -> Self;
    fn subtract(&self, other: &Self) -> Self;
    fn scale(&self, factor: f64) -> Self;
    // fn magnitude(&self) -> f64;
    // fn magnitude_squared(&self) -> f64;
}
