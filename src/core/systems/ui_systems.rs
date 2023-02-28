use crate::core::plugins::components::{
    CameraComponent,
    TransformComponent,
    TransformUiComponent,
    MainMenuComponent,
    FileSubMenuComponent,
    SceneGraphComponent,
};
use crate::core::events::project_events::{
    SaveEvent,
    CreateProjectEvent,
    CloseProjectEvent,
    OpenProjectEvent,
};


use ember_math::Vector3f;

use egui_vulkano::Painter;
use egui::Context;
use egui::Ui;


use bevy_ecs::prelude::{
    Res,
    Query,
    World,
};

use bevy_ecs::prelude::EventWriter;
use bevy_ecs::entity::Entity;
// use puffin_egui;


use std::path::Path;
use log;

pub struct EguiState{
    pub ctx: Context,
    pub painter: Painter,
}


pub fn MainMenuInitSystem(
    mut query: Query<(&mut MainMenuComponent, Entity)>,
    egui_state: Res<EguiState>,
){
    log::debug!("Init main menu...");
    for (mut comp, entity) in query.iter_mut(){
        let ctx = egui_state.ctx.clone();
        let panel = egui::TopBottomPanel::top("Debug")
            .show(&ctx, |ui| {
                Ui::new(
                    ctx.clone(),
                    ui.layer_id(),
                    egui::Id::new(("MainMenuComponent{}", entity.id())),
                    ui.max_rect(),
                    ui.clip_rect()
                )
            });
        let ui = panel.inner;
        comp.ui = Some(ui);
    }
}

pub fn FileSubMenuSystem(
    mut query: Query<(&mut FileSubMenuComponent, Entity)>,
    mut main_menu_query: Query<&mut MainMenuComponent>,
    egui_state: Res<EguiState>,
    mut save_events: EventWriter<SaveEvent>,
    mut close_events: EventWriter<CloseProjectEvent>,
){
    log::debug!("File Sub Menu System...");
    for (mut comp, entity) in query.iter_mut(){
        let _ctx = egui_state.ctx.clone();
        // let mut parent = query.get_mut(entity.id()).resolve_from_id(comp.parent_id.unwrap()).unwrap();//&mut comp.parent.as_mut().unwrap();
        let mut target_comp = main_menu_query.get_mut(entity).expect("target not found");
        let menu_ui = target_comp.ui.as_mut().expect("No ui on target comp");
        let _file_ui = menu_ui.menu_button("File", |ui|{
            if ui.button("New").clicked() {
                log::info!("New project...");
                comp.new_project_window = true;
                comp.open_project_window = false;
                // comp.current_nav_path = std::env::current_dir().unwrap();
                ui.close_menu();
            }
            if ui.button("Open").clicked() {
                let _root = std::env::current_dir().unwrap();
                comp.open_project_window = true;
                comp.new_project_window = false;
                ui.close_menu();
                // comp.current_nav_path = std::env::current_dir().unwrap();
            }
            if ui.button("Save").clicked() {
                save_events.send(SaveEvent);
                ui.close_menu();
            }
            if ui.button("Close").clicked() {
                close_events.send(CloseProjectEvent);
                ui.close_menu();
            }
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

        egui::Window::new("New Project")
            .open(&mut comp.new_project_window)
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

        egui::Window::new("Open Project")
            .open(&mut comp.open_project_window)
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
    query: Query<&SceneGraphComponent>,
    egui_state: Res<EguiState>,
    world: &World,
){
    for comp in query.iter(){
        let ctx = egui_state.ctx.clone();
        egui::SidePanel::left("LeftPanel").show(&ctx, |ui|{
            ui.colored_label(egui::Color32::WHITE, "Entities");
            // for entity in world.iter_entities() {
                // ui.colored_label(egui::Color32::WHITE, format!("Entity {}", entity));
            // }
            ui.allocate_space(ui.available_size());
        });
    }
}