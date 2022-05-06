use bevy_ecs::component::Component;

use cgmath::{
    Matrix4,
    Vector3,
    Rad,
    Point3,
    Deg
};
use cgmath;
use serde::{
    Serialize,
    Deserialize,
};


#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CameraComponent{
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub look_at: Vector3<f32>,
    pub eye: Vector3<f32>,
    pub up: Vector3<f32>,
    pub perspective: Matrix4<f32>,
    pub view: Matrix4<f32>,
    correction_matrix: Matrix4<f32>,
}

impl CameraComponent {
    pub fn create_default() -> Self {
        let fov = 3.1415 / 1.75;
        let near = 0.001;
        let far = 1e6;
        let aspect = 0.5;
        let look_at = Vector3::new(0.0, 0.0, 0.0);
        let eye = Vector3::new(3.0, 0.0, -3.0);
        let up = Vector3::new(0.0, 0.0, 1.0);
        let mut cam = CameraComponent{
            fov: fov,
            near: near,
            far: far,
            aspect: aspect,
            look_at: look_at,
            eye: eye,
            up: up,
            perspective: Matrix4::from_scale(1.0),
            view: Matrix4::from_scale(1.0),
            correction_matrix: Matrix4::from_angle_y(Deg(180.0)),
        };
        cam.calculate_perspective();
        cam.calculate_view();
        cam
    }

    pub fn calculate_perspective(&mut self) {
        self.perspective = cgmath::perspective(Rad(self.fov), self.aspect, self.near, self.far);
    }

    pub fn calculate_view(&mut self) {
        self.view = Matrix4::look_at_rh(
            Point3::new(self.eye.x, self.eye.y, self.eye.z),
            Point3::new(self.look_at.x, self.look_at.y, self.look_at.z),
            self.up
        ) * self.correction_matrix;
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        self.view.clone()
    }

    pub fn get_perspective(&self) -> Matrix4<f32> {
        self.perspective.clone()
    }
}
