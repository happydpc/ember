use specs::{Component, HashMapStorage};
use cgmath::{
    Matrix4,
    Vector3,
    Rad,
    InnerSpace,
};
use cgmath;

#[derive(Component, Debug)]
#[storage(HashMapStorage)]
pub struct CameraComponent{
    pub fov: f64,
    pub near: f64,
    pub far: f64,
    pub aspect: f64,
    pub look_at: Vector3<f64>,
    pub eye: Vector3<f64>,
    pub up: Vector3<f64>,
    pub perspective: Matrix4<f64>,
    pub view: Matrix4<f64>,
}

impl CameraComponent {
    pub fn create_default() -> Self {
        let fov = 3.1415 / 2.0;
        let near = 0.001;
        let far = 1e6;
        let aspect = 0.5;
        let look_at = Vector3::new(0.0, 0.0, 0.0);
        let eye = Vector3::new(1.0, 0.0, 1.0);
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
        };
        cam.calculate_perspective();
        cam.calculate_view();
        cam
    }

    pub fn calculate_perspective(&mut self) {
        self.perspective = cgmath::perspective(Rad(self.fov), self.aspect, self.near, self.far);
    }

    pub fn calculate_view(&mut self) {
        let z_axis = (self.eye - self.look_at).normalize();
        let x_axis = self.up.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis);

        let orientation_matrix = Matrix4::new(
                x_axis.x, y_axis.x, z_axis.x, 0.0,
                x_axis.y, y_axis.y, z_axis.y, 0.0,
                x_axis.z, y_axis.z, z_axis.z, 0.0,
                0.0, 0.0, 0.0, 1.0,
        );

        let translation_matrix = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -self.eye.x, -self.eye.y, -self.eye.z, 1.0,
        );

        self.view = orientation_matrix * translation_matrix;
    }

    pub fn get_view(&self) -> Matrix4<f64> {
        self.view.clone()
    }

    pub fn get_perspective(&self) -> Matrix4<f64> {
        self.perspective.clone()
    }
}
