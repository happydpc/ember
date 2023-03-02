use crate::core::plugins::components::ui::main_menu_component::{ComponentLibraryComponent, EditorUiState, EntityInspectorComponent, UiPanelComponent, PanelType};
use crate::core::plugins::components::{
    CameraComponent,
    TransformComponent,
    TransformUiComponent,
    FileSubMenuComponent,
    SceneGraphComponent,
};
use crate::core::events::project_events::{
    SaveEvent,
    CreateProjectEvent,
    CloseProjectEvent,
    OpenProjectEvent,
};


use bevy_ecs::system::Commands;
use bevy_hierarchy::Parent;
use ember_math::Vector3f;

use egui_vulkano::Painter;
use egui::Context;
use egui::Ui;
use egui::Layout;
use egui::Align;

use bevy_ecs::prelude::{
    Res,
    Query,
    World,
};

use bevy_ecs::prelude::EventWriter;
use bevy_ecs::prelude::Resource;
use bevy_ecs::entity::Entity;
// use puffin_egui;


use std::arch;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use log;

#[derive(Resource)]
pub struct EguiState{
    pub ctx: Context,
    pub painter: Painter,
}

pub fn PanelInitSystem(
    mut query: Query<(&mut UiPanelComponent, Entity)>,
    egui_state: Res<EguiState>,
){
    log::debug!("Init UiPanelComponent panel...");
    for (mut comp, entity) in query.iter_mut(){
        let ctx = egui_state.ctx.clone();
        let panel = match comp.panel_type {
            PanelType::left => {
                egui::SidePanel::left("LeftPanel")
                    .show(&ctx, |ui| {
                        ui.allocate_space(ui.available_size());
                        ui.child_ui(ui.max_rect(), Layout::top_down(Align::LEFT))
                    })
            },
            PanelType::right => {
                egui::SidePanel::right("RightPanel")
                    .show(&ctx, |ui| {
                        ui.allocate_space(ui.available_size());
                        ui.child_ui(ui.max_rect(), Layout::top_down(Align::LEFT))
                    })
            },
            PanelType::top => {
                egui::TopBottomPanel::top("TopPanel")
                    .resizable(true)
                    .show(&ctx, |ui| {
                        ui.allocate_space(ui.available_size());
                        ui.child_ui(ui.max_rect(), Layout::top_down(Align::LEFT))
                    })
            },
            PanelType::bottom => {
                egui::TopBottomPanel::bottom("BottomPanel")
                    .resizable(true)
                    .show(&ctx, |ui| {
                        ui.allocate_space(ui.available_size());
                        ui.child_ui(ui.max_rect(), Layout::top_down(Align::LEFT))
                    })
            },
        };
        let ui = panel.inner;
        comp.ui = Some(Arc::new(Mutex::new(ui)));
    }
}

pub fn FileSubMenuSystem(
    query: Query<(&FileSubMenuComponent, Entity)>,
    world: &World,
    mut commands: Commands
){
    log::debug!("File Sub Menu System...");
    let (comp, entity) = query.get_single().unwrap();
    let mut send_save = false;
    let mut send_close = false;
    let parent_entity = world.get_entity(entity).unwrap().get::<Parent>().unwrap().get();
    let ui_arc = world.get_entity(parent_entity).unwrap().get::<UiPanelComponent>().unwrap().ui.clone().unwrap();
    let mut ui = ui_arc.lock().unwrap();
    let _file_ui = ui.menu_button("File", |ui|{
        if ui.button("New").clicked() {
            log::info!("New project...");
            {
                *comp.new_project_window.lock().unwrap() = true;
                *comp.open_project_window.lock().unwrap() = false;
            }
            ui.close_menu();
        }
        if ui.button("Open").clicked() {
            {
                *comp.new_project_window.lock().unwrap() = false;
                *comp.open_project_window.lock().unwrap() = true;
            }
            ui.close_menu();
        }
        if ui.button("Save").clicked() {
            send_save = true;
            ui.close_menu();
        }
        if ui.button("Close").clicked() {
            send_close = true;
            ui.close_menu();
        }
    });

    if send_save {
        commands.add(|world: &mut World| {
            world.send_event(SaveEvent);
        })
    }

    if send_close {
        commands.add(|world: &mut World|{
            world.send_event(CloseProjectEvent);
        });
    }
}

pub fn ShowNewProjectWindow(
    mut query: Query<&mut FileSubMenuComponent>,
    egui_state: Res<EguiState>,
    mut create_project_events: EventWriter<CreateProjectEvent>,
){
    for mut comp in query.iter_mut(){
        let ctx = egui_state.ctx.clone();
        let mut current_path = comp.current_nav_path.clone();
        let paths = std::fs::read_dir(&current_path).unwrap();
        let mut entry_buf = comp.text_entry.clone();
        let new_project_window_arc = comp.new_project_window.clone();
        let mut new_project_window = new_project_window_arc.lock().unwrap();
        egui::Window::new("New Project")
            .open(&mut *new_project_window)
            .show(&ctx.clone(), |ui|{
                ui.label("Select a location for a new project.");
                ui.horizontal(|ui|{

                    ui.label("Current Path : ");
                    if ui.button("+").clicked(){
                        current_path = match current_path.clone().as_path().parent(){
                            Some(p) => p.to_path_buf(),
                            None => current_path.clone(),
                        };
                    }
                    ui.label(format!("{}", current_path.to_str().unwrap()))
                });

                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .max_width(f32::INFINITY)
                    .show(ui, |ui|{
                        for path in paths {
                            let p = path.unwrap().path();
                            if !p.is_dir(){
                                continue;
                            }
                            let label = ui.selectable_label(
                                false,
                                p.clone().to_str().unwrap(),
                            );
                            if label.clicked() {
                                if p.clone().is_dir(){
                                    current_path = p.clone();
                                }
                            }
                        }
                });
                ui.separator();
                ui.horizontal(|ui|{
                    ui.label("Project Name : ");
                    ui.text_edit_singleline(&mut entry_buf);
                    if ui.button("Create Project").clicked(){
                        let target = Path::new(&entry_buf);
                        let mut p = current_path.clone();
                        p.push(target);
                        if p.exists() {
                            log::warn!("Path already exists!");
                        }else{
                            create_project_events.send(
                                CreateProjectEvent{
                                    project_path: String::from(
                                        p.to_str()
                                            .unwrap()
                                        ),
                                    scene_name: String::from("default.ron")
                                    }
                                );
                        }
                    }
                });

        });
        comp.text_entry = entry_buf;
        comp.current_nav_path = current_path;
    }
}

pub fn ShowOpenProjectWindow(
    mut query: Query<&mut FileSubMenuComponent>,
    egui_state: Res<EguiState>,
    mut open_project_events: EventWriter<OpenProjectEvent>,
){
    for mut comp in query.iter_mut(){
        let ctx = egui_state.ctx.clone();
        let mut current_path = comp.current_nav_path.clone();
        let paths = std::fs::read_dir(&current_path).unwrap();
        let entry_buf = comp.text_entry.clone();
        let open_project_window_arc = comp.open_project_window.clone();
        let mut open_project_window = open_project_window_arc.lock().unwrap();
        egui::Window::new("Open Project")
            .open(&mut *open_project_window)
            .show(&ctx.clone(), |ui|{
                ui.label("Select a location for a new project.");
                ui.horizontal(|ui|{

                    ui.label("Current Path : ");
                    if ui.button("+").clicked(){
                        current_path = match current_path.clone().as_path().parent(){
                            Some(p) => p.to_path_buf(),
                            None => current_path.clone(),
                        };
                    }
                    ui.label(format!("{}", current_path.to_str().unwrap()))
                });

                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .max_width(f32::INFINITY)
                    .show(ui, |ui|{
                        for path in paths {
                            let p = path.unwrap().path();
                            if !p.is_dir(){
                                continue;
                            }
                            let label = ui.selectable_label(
                                false,
                                p.clone().to_str().unwrap(),
                            );
                            if label.clicked() {
                                if p.clone().is_dir(){
                                    current_path = p.clone();
                                }
                            }
                        }
                });
                ui.separator();
                ui.horizontal(|ui|{
                    ui.label("Selected Project : ");
                    ui.label(current_path.clone().to_str().unwrap());
                    if ui.button("Open Project").clicked(){
                        let target = Path::new(&entry_buf);
                        let mut p = current_path.clone();
                        p.push(target);
                        if !p.exists() {
                            log::warn!("Project doesn't exist at {:?} ", p.to_str());
                        }else{ // project folder exists
                            let mut proj_file = p.clone();
                            proj_file.push("ember.project");
                            if !proj_file.exists(){
                                log::warn!("This is not a valid project. {} Does not exist", proj_file.display());
                            } else {
                                log::info!("Opening project : {:?}", p.to_str().unwrap());
                                open_project_events.send(OpenProjectEvent{project_path: String::from(p.to_str().unwrap())});
                            }
                        }
                    }
                });

        });
        comp.current_nav_path = current_path;
    }

}

pub fn CameraUiSystem(
    mut query: Query<&mut CameraComponent>,
    egui_state: Res<EguiState>,
)
{
    log::debug!("Camera ui...");
    let ctx = egui_state.ctx.clone();
    for mut cam in query.iter_mut(){
        let mut fov = cam.fov;
        egui::Window::new("Camera Settings")
            .show(&ctx, |ui| {
                ui.label("FOV");
                ui.add(egui::Slider::new(&mut fov, 0.1..=5.0));
                ui.label(format!("Radius {}", cam.radius));
                ui.label(format!("Azimuth {}", cam.azimuth));
                ui.label(format!("Declination {}", cam.declination));
                ui.horizontal(|ui| {
                    ui.label("eye: : ");
                    ui.add(egui::DragValue::new(&mut cam.eye.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut cam.eye.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut cam.eye.z).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("look_at: : ");
                    ui.add(egui::DragValue::new(&mut cam.look_at.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut cam.look_at.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut cam.look_at.z).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("up: : ");
                    ui.add(egui::DragValue::new(&mut cam.up.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut cam.up.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut cam.up.z).speed(0.1));
                });
                if ui.button("Reset").clicked() {
                    cam.look_at = Vector3f::zero();
                    cam.eye = Vector3f::new(10.0, 10.0, 10.0);
                    cam.up = Vector3f::new(0.0, 1.0, 0.0);
                }
            });
        if cam.fov != fov {
            cam.fov = fov;
            cam.calculate_perspective();
        }
    }
}


pub fn TransformUiSystem(
    mut query: Query<(&mut TransformComponent, &TransformUiComponent, Entity)>,
    egui_state: Res<EguiState>,
){
    log::debug!("Transform ui....");
    let ctx = egui_state.ctx.clone();
    for (mut transform, _ui_comp, entity) in query.iter_mut(){
        egui::Window::new(format!("Transform {:?}", entity))
            .show(&ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position: ");
                    ui.add(egui::DragValue::new(&mut transform.global_position.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut transform.global_position.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut transform.global_position.z).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Scale: ");
                    ui.add(egui::DragValue::new(&mut transform.scale).speed(0.01));
                })
            });
    }
}

pub fn SceneGraphUiSystem(
    mut query: Query<(&SceneGraphComponent, Entity)>,
    egui_state: Res<EguiState>,
    world: &World,
    mut commands: Commands
){

    for (comp, entity) in query.iter(){
        let parent_entity = world.get_entity(entity).unwrap().get::<Parent>().unwrap().get();
        let mut left_panel = world.get_entity(parent_entity).unwrap().get::<UiPanelComponent>().expect("target not found");
        let ui_arc = left_panel.ui.clone().unwrap().clone();
        let mut left_panel_ui = ui_arc.lock().unwrap();
        
        let ctx = egui_state.ctx.clone();
        let archetypes = world.archetypes();
        let components = world.components();
        
        egui::CollapsingHeader::new("Entities").show(&mut left_panel_ui, |ui|{
            for entity in world.iter_entities() {
                let response = ui.selectable_label(false, format!("Entity {}", entity.to_bits()));
                if response.clicked(){
                    let send_entity = entity.clone();
                    commands.add(move |world: &mut World|{
                        let mut ui_state = world.get_resource_or_insert_with(EditorUiState::default);
                        ui_state.selected_entity = Some(send_entity);
                    });
                }
            }
        });
    }
}

pub fn ComponentLibraryUiSystem(
    query: Query<&ComponentLibraryComponent>,
    egui_state: Res<EguiState>,
    world: &World,
){

}

pub fn EntityInspectionUiSystem(
    query: Query<(&EntityInspectorComponent, Entity)>,
    ui_state: Res<EditorUiState>,
    world: &World,
){
    if let Some(selected_entity) = ui_state.selected_entity{

        // for (comp, entity) in query.iter(){
        let (_, entity) = query.get_single().unwrap();
        let parent_entity = world.get_entity(entity).unwrap().get::<Parent>().unwrap().get();
        let mut right_panel = world.get_entity(parent_entity).unwrap().get::<UiPanelComponent>().unwrap();
        let ui_arc = right_panel.ui.clone().unwrap().clone();
        let mut ui = ui_arc.lock().unwrap();

        ui.heading(format!("Selected Entity {}", selected_entity.to_bits()));
        ui.separator();

        let archetypes = world.archetypes();
        let components = world.components();
        for archetype in archetypes.iter() {
            if archetype.entities().iter().any(|e| e.entity() == selected_entity) {
                let entity_components = archetype.components();
                for comp in entity_components{
                    if let Some(comp_info) = components.get_info(comp) {
                        ui.selectable_label(false, comp_info.name().split("::").last().unwrap());
                    }
                }
            }
        }
    }
}