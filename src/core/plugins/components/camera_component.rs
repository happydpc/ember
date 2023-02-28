use core::f32::consts::PI;
use bevy_ecs::component::Component;

use ember_math::{
    Matrix4f,
    Vector3f
};
use serde::{
    Serialize,
    Deserialize,
};
use bevy_ecs::reflect::ReflectComponent;
use bevy_reflect::{Reflect, FromReflect};

// bevy_reflect::impl_reflect_value!(CameraComponent);
// bevy_reflect::impl_from_reflect_value!(CameraComponent);

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, FromReflect)]
#[reflect(Component)]
pub struct CameraComponent{
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub orbit_speed: f32, // units are radians
    #[reflect(ignore)]
    pub look_at: Vector3f,
    #[reflect(ignore)]
    pub eye: Vector3f,
    #[reflect(ignore)]
    pub up: Vector3f,
    #[reflect(ignore)]
    pub perspective: Matrix4f,
    #[reflect(ignore)]
    pub view: Matrix4f,
    #[reflect(ignore)]
    pub azimuth: f32,
    #[reflect(ignore)]
    pub declination: f32,
    #[reflect(ignore)]
    pub radius: f32,
}

impl CameraComponent {

    pub fn update_cartesian(&mut self){
        let eye_x = self.radius * self.declination.sin() * self.azimuth.cos();
        let eye_y = self.radius * self.declination.cos();
        let eye_z = self.radius * self.declination.sin() * self.azimuth.sin();
        self.eye.x = eye_x;
        self.eye.y = eye_y;
        self.eye.z = eye_z;
        self.eye += self.look_at;
        self.calculate_perspective();
        self.calculate_view();
    }

    pub fn update(&mut self){
        self.calculate_perspective();
        self.calculate_view();
    }

    pub fn calculate_perspective(&mut self) {
        self.perspective = ember_math::Matrix4f::perspective(self.fov, self.aspect, self.near, self.far);
    }

    pub fn calculate_view(&mut self) {
        self.view = Matrix4f::look_at_rh(
            Vector3f::new(self.eye.x, self.eye.y, self.eye.z),
            Vector3f::new(self.look_at.x, self.look_at.y, self.look_at.z),
            self.up
        )
    }

    pub fn get_view(&self) -> Matrix4f {
        self.view.clone()
    }

    pub fn get_perspective(&self) -> Matrix4f {
        self.perspective.clone()
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        let fov = 3.14 / 2.0;
        let near = 0.01;
        let far = 1e6;
        let aspect = 1.0;
        let radius = 10.0;
        let azimuth = PI / 4.0;
        let declination = PI / 4.0;
        let look_at = Vector3f::new(0.0, 0.0, 0.0);
        let eye = Vector3f::new(5.0, 5.0, 5.0);
        let up = Vector3f::new(0.0, 1.0, 0.0);
        let mut cam = CameraComponent{
            fov: fov,
            near: near,
            far: far,
            aspect: aspect,
            look_at: look_at,
            eye: eye,
            up: up,
            perspective: Matrix4f::from_scale(1.0),
            view: Matrix4f::from_scale(1.0),
            azimuth: azimuth,
            declination: declination,
            radius: radius,
            orbit_speed: PI / 330.0 // magic number. just seemed reasonable
        };
        cam.update_cartesian();
        cam.update();
        cam
    }
}