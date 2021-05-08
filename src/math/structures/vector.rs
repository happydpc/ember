use super::tensor::Tensor;
use glium;

// Struct definitions

#[derive(Default, Copy, Clone)]
pub struct Vector2{
    pub position: [f64; 2],
}

#[derive(Default, Copy, Clone)]
pub struct Vector3{
    pub position: [f64; 3],
}

#[derive(Default, Copy, Clone)]
pub struct Vector4{
    pub position: [f64; 4],
}

// implement vertex for these
glium::implement_vertex!(Vector2, position);
glium::implement_vertex!(Vector3, position);

// struct implementations
impl Vector2{
    pub fn get_x(&self) -> f64 {
        self.position[0].clone()
    }
    pub fn get_y(&self) -> f64 {
        self.position[1].clone()
    }
}

//  Trait implementations
impl Tensor for Vector2{
    fn add(&self, other: &Self) -> Self{
        let x = self.position[0] + other.position[0];
        let y = self.position[1] + other.position[1];
        Vector2{
            position: [x, y],
        }
    }
    fn subtract(&self, other: &Self) -> Self{
        let x = self.position[0] - other.position[0];
        let y = self.position[0] - other.position[0];
        Vector2{
            position: [x, y],
        }
    }
    fn scale(&self, factor: f64) -> Self{
        let x = self.position[0] * factor;
        let y = self.position[1] * factor;
        Vector2{
            position: [x, y],
        }
    }
}

impl Tensor for Vector3{
    fn add(&self, other: &Self) -> Self{
        let x = self.position[0] + other.position[0];
        let y = self.position[1] + other.position[1];
        let z = self.position[2] + other.position[2];

        Vector3{
            position: [x, y, z],
        }
    }
    fn subtract(&self, other: &Self) -> Self{
        let x = self.position[0] - other.position[0];
        let y = self.position[1] - other.position[1];
        let z = self.position[2] - other.position[2];
        Vector3{
            position: [x, y, z],
        }
    }
    fn scale(&self, factor: f64) -> Self{
        let x = self.position[0] * factor;
        let y = self.position[1] * factor;
        let z = self.position[2] * factor;

        Vector3{
            position: [x, y, z],
        }
    }
}

impl Tensor for Vector4{
    fn add(&self, other: &Self) -> Self{
        let x = self.position[0] + other.position[0];
        let y = self.position[1] + other.position[1];
        let z = self.position[2] + other.position[2];
        let w = self.position[3] + other.position[3];

        Vector4{
            position: [x, y, z, w],
        }
    }
    fn subtract(&self, other: &Self) -> Self{
        let x = self.position[0] - other.position[0];
        let y = self.position[1] - other.position[1];
        let z = self.position[2] - other.position[2];
        let w = self.position[3] - other.position[3];

        Vector4{
            position: [x, y, z, w],
        }
    }
    fn scale(&self, factor: f64) -> Self{
        let x = self.position[0] * factor;
        let y = self.position[1] * factor;
        let z = self.position[2] * factor;
        let w = self.position[3] * factor;
        Vector4{
            position: [x, y, z, w],
        }
    }
}
