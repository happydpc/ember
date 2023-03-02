use bevy_ecs::{
    prelude::{
        Query, 
        Res,
    }
};
use vulkano::{memory::{allocator::StandardMemoryAllocator}};

use std::sync::Arc;
use crate::core::rendering::geometries::geometry_primitives::{
    Vertex,
};
use crate::core::plugins::components::geometry_component::{GeometryComponent, GeometryType};
use crate::core::managers::render_manager::VulkanAllocators;
pub struct GeometryInitHelper;

impl GeometryInitHelper{
    fn create_geometry(mut geom: &mut GeometryComponent, memory_allocator: Arc<StandardMemoryAllocator>){
        match geom.geometry_type{
            GeometryType::Box => GeometryInitHelper::init_cube(&mut geom),
            GeometryType::Triangle => GeometryInitHelper::init_triangle(&mut geom),
            GeometryType::Plane => GeometryInitHelper::init_plane(&mut geom),
        };
        geom.initialize(memory_allocator.clone());
    }

    fn init_cube(mut geom: &mut GeometryComponent){
        // dx here is just delta, not associated with x axis
        let dx = 0.5;

        // bottom plane
        let tl0 = Vertex::new(0.0 - dx, 0.0 - dx, 0.0 + dx);
        let tr0 = Vertex::new(0.0 + dx, 0.0 - dx, 0.0 + dx);
        let bl0 = Vertex::new(0.0 - dx, 0.0 - dx, 0.0 - dx);
        let br0 = Vertex::new(0.0 + dx, 0.0 - dx, 0.0 - dx);

        // top plane
        let tl1 = Vertex::new(0.0 - dx, 0.0 + dx, 0.0 + dx);
        let tr1 = Vertex::new(0.0 + dx, 0.0 + dx, 0.0 + dx);
        let bl1 = Vertex::new(0.0 - dx, 0.0 + dx, 0.0 - dx);
        let br1 = Vertex::new(0.0 + dx, 0.0 + dx, 0.0 - dx);

        // store verts.       0    1    2    3    4    5    6    7
        let vertices = vec![tl0, tr0, bl0, br0, tl1, tr1, bl1, br1];

        // visualize as looking top down (looking down -y axis) -x is left, +z is "up" (forwards)
        // top, front, right, back, left, bottom
        let indices = vec![
            5, 6, 4, 5, 7, 6, // top
            3, 2, 7, 2, 6, 7, // front
            7, 5, 1, 3, 7, 1, // right
            5, 4, 0, 1, 5, 0, // back
            4, 6, 2, 0, 4, 2, // left
            0, 2, 1, 2, 3, 1, // bottom
        ];

        geom.vertices = vertices;
        geom.indices = indices;
        // geom.initialized = true;
    }

    fn init_plane(mut geom: &mut GeometryComponent){
        let corner_offset = 0.5;

        // top left, top right, bottom left, bottom right
        let tl = Vertex{position: [-corner_offset, corner_offset, 0.0]};
        let tr = Vertex{position: [corner_offset, corner_offset, 0.0]};
        let bl = Vertex{position: [-corner_offset, -corner_offset, 0.0]};
        let br = Vertex{position: [corner_offset, -corner_offset, 0.0]};

        geom.vertices = vec![tl, tr, bl, br];
        geom.indices = vec![0, 1, 3, 2, 0, 3];
        // geom.initialized = true;
    }

    fn init_triangle(mut geom: &mut GeometryComponent){
        let corner_offset = 0.5;
        let vertices = vec![
            Vertex{position: [-corner_offset, -corner_offset, 0.0]},
            Vertex{position: [0.0, corner_offset, 0.0]},
            Vertex{position: [corner_offset, -corner_offset, 0.0]}
        ];
        geom.vertices = vertices;
        geom.indices = vec![0, 1, 2, 0];
        // geom.initialized = true;
    }
}


pub fn GeometryInitializerSystem(
    mut query: Query<&mut GeometryComponent>,
    allocators: Res<VulkanAllocators>,
)
{
    let memory_allocator = allocators.memory_allocator();
    log::debug!("Running geometry init system...");
    for mut geometry in query.iter_mut() {
        GeometryInitHelper::create_geometry(&mut geometry, memory_allocator.clone());
        geometry.initialize(memory_allocator.clone());
    }
}