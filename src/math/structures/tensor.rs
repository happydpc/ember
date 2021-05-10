pub trait Tensor{
    fn add(&self, other: &Self) -> Self;
    fn subtract(&self, other: &Self) -> Self;
    fn scale(&self, factor: f32) -> Self;
    // fn magnitude(&self) -> f32;
    // fn magnitude_squared(&self) -> f32;
}
