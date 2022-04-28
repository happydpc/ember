// use super::tensor::Tensor;
// use cgmath::Vector3;
// use serde::{Serialize, Deserialize, Serializer};

// impl<T> Serialize for Vector3<T>
// where
//     T: Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut seq = serializer.serialize_seq(3)?;
//         for e in self {
//             seq.serialize_element(e)?;
//         }
//         // seq.serialize_element()
//         seq.end()
//     }
// }

// // Struct definitions

// #[derive(Default, Copy, Clone, Debug)]
// pub struct Vector2{
//     pub position: [f32; 2],
// }

// #[derive(Default, Copy, Clone, Debug)]
// pub struct Vector3{
//     pub position: [f32; 3],
// }

// #[derive(Default, Copy, Clone, Debug)]
// pub struct Vector4{
//     pub position: [f32; 4],
// }

// // struct implementations
// impl Vector2{
//     pub fn get_x(&self) -> f32 {
//         self.position[0].clone()
//     }
//     pub fn get_y(&self) -> f32 {
//         self.position[1].clone()
//     }
// }

// //  Trait implementations
// impl Tensor for Vector2{
//     fn add(&self, other: &Self) -> Self{
//         let x = self.position[0] + other.position[0];
//         let y = self.position[1] + other.position[1];
//         Vector2{
//             position: [x, y],
//         }
//     }
//     fn subtract(&self, other: &Self) -> Self{
//         let x = self.position[0] - other.position[0];
//         let y = self.position[0] - other.position[0];
//         Vector2{
//             position: [x, y],
//         }
//     }
//     fn scale(&self, factor: f32) -> Self{
//         let x = self.position[0] * factor;
//         let y = self.position[1] * factor;
//         Vector2{
//             position: [x, y],
//         }
//     }
// }

// impl Tensor for Vector3{
//     fn add(&self, other: &Self) -> Self{
//         let x = self.position[0] + other.position[0];
//         let y = self.position[1] + other.position[1];
//         let z = self.position[2] + other.position[2];

//         Vector3{
//             position: [x, y, z],
//         }
//     }
//     fn subtract(&self, other: &Self) -> Self{
//         let x = self.position[0] - other.position[0];
//         let y = self.position[1] - other.position[1];
//         let z = self.position[2] - other.position[2];
//         Vector3{
//             position: [x, y, z],
//         }
//     }
//     fn scale(&self, factor: f32) -> Self{
//         let x = self.position[0] * factor;
//         let y = self.position[1] * factor;
//         let z = self.position[2] * factor;

//         Vector3{
//             position: [x, y, z],
//         }
//     }
// }

// impl Tensor for Vector4{
//     fn add(&self, other: &Self) -> Self{
//         let x = self.position[0] + other.position[0];
//         let y = self.position[1] + other.position[1];
//         let z = self.position[2] + other.position[2];
//         let w = self.position[3] + other.position[3];

//         Vector4{
//             position: [x, y, z, w],
//         }
//     }
//     fn subtract(&self, other: &Self) -> Self{
//         let x = self.position[0] - other.position[0];
//         let y = self.position[1] - other.position[1];
//         let z = self.position[2] - other.position[2];
//         let w = self.position[3] - other.position[3];

//         Vector4{
//             position: [x, y, z, w],
//         }
//     }
//     fn scale(&self, factor: f32) -> Self{
//         let x = self.position[0] * factor;
//         let y = self.position[1] * factor;
//         let z = self.position[2] * factor;
//         let w = self.position[3] * factor;
//         Vector4{
//             position: [x, y, z, w],
//         }
//     }
// }
