use crate::core::plugins::components::{
    DebugUiComponent,
    CameraComponent,
    TransformComponent,
    TransformUiComponent,
    MainMenuComponent,
    FileSubMenuComponent,
    FileMenuSaveComponent,
};
use crate::core::events::serialization_events::SaveEvent;
use crate::core::events::menu_messages::MenuMessage;


use egui_vulkano::Painter;
use egui::Context;
use egui::Ui;
use egui::Id;

use bevy_ecs::prelude::{
    Res,
    ResMut,
    Query,
    With,
};
use bevy_ecs::event::Events;
use bevy_ecs::entity::Entity;
// use puffin_egui;

use log;


pub struct EguiState{
    pub ctx: Context,
    pub painter: Painter,
}


pub fn MainMenuInitSystem(
    mut query: Query<(&mut MainMenuComponent, Entity)>,
    mut egui_state: ResMut<EguiState>,
){
    log::debug!("Init main menu.........................");
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
pub fn MainMenuSystem(
    mut query: Query<&mut MainMenuComponent>,
    mut egui_state: ResMut<EguiState>,
    mut menu_items: ResMut<Events<MenuMessage<MainMenuComponent>>>,
){
    // let ctx = &mut egui_state.ctx;
    // let mut reader = menu_items.get_reader();

    // egui::TopBottomPanel::top("Test")
    //     .show(&ctx, |ui|{
    //         for item in reader.iter(&menu_items){
    //             (item.ui)(ui);
    //         }
    //     });
    // menu_items.clear();
}

pub fn FileSubMenuSystem(
    mut query: Query<&mut FileSubMenuComponent>,
    mut main_menu_query: Query<&mut MainMenuComponent>,
    mut save_events: ResMut<Events<SaveEvent>>,
){
    log::debug!("File Sub Menu System...");
    for mut comp in query.iter_mut(){
        let mut parent = &mut comp.parent.as_mut().unwrap();
        // let mut ui = &mut parent.get::<MainMenuComponent>().ui;
        let mut target_comp = main_menu_query.get_mut(parent.clone()).expect("Couldn't get main menu component");
        let mut ui = target_comp.ui.as_mut().expect("No ui on target comp");
        ui.menu_button("File", |ui|{
            if ui.button("New").clicked() {
                log::info!("New project...");
            }
            if ui.button("Open").clicked() {
                log::info!("Opening a file...");
            }
            if ui.button("Save").clicked() {
                log::info!("Sending a save message");
                save_events.send(SaveEvent);
            }
            if ui.button("Close").clicked() {
                log::info!("Close scene...");
            }
        });
    }
    // let mut reader = submenu_items.get_reader();
    // let message = MenuMessage::<MainMenuComponent>::new(|ui|{
    //     ui.menu_button("File", |ui| {
    //         for item in reader.iter(&submenu_items) {
    //             (item.ui)(ui);
    //         }
    //     });
    // });
    // target_menu_channel.send(message);
    // submenu_items.clear();
}

pub fn FileMenuSaveSystem(
    mut query: Query<&FileMenuSaveComponent>,
    mut target_menu_channel: ResMut<Events<MenuMessage<FileSubMenuComponent>>>,
    mut save_events: ResMut<Events<SaveEvent>>,
){
    // let message = MenuMessage::<FileSubMenuComponent>::new(|ui|{
    //     // if ui.button("Save").clicked() {
    //     //     log::info!("Sending a save message");
    //     //     save_events.send(SaveEvent);
    //     // }
    // });
}

pub fn DebugUiSystem(
    mut query: Query<&mut DebugUiComponent>,
    mut egui_state: ResMut<EguiState>,
    mut should_save: ResMut<bool>,
    mut save_events: ResMut<Events<SaveEvent>>,
){
    log::debug!("Debug ui...");
    let ctx = egui_state.ctx.clone();
    for mut comp in query.iter_mut() {

        // draw ui
        let panel = egui::TopBottomPanel::top("Debug")
            .show(&ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            log::info!("New project...");
                        }
                        if ui.button("Open").clicked() {
                            log::info!("Opening a file...");
                        }
                        if ui.button("Save").clicked() {
                            log::info!("Sending a save message");
                            save_events.send(SaveEvent);
                        }
                        if ui.button("Close").clicked() {
                            log::info!("Close scene...");
                        }
                        if SubMenu(ui){
                            save_events.send(SaveEvent);
                        }
                    });

                    ui.menu_button("Debug Options", |ui| {
                        if ui.button("Toggle Profiling").clicked() {
                            log::info!("I still don't know why this breaks.");
                            comp.show_profiler = !comp.show_profiler;
                        }
                        if ui.button("Toggle wireframe").clicked() {
                            log::info!("Toggling wireframe");
                            comp.terrain_wireframe = !comp.terrain_wireframe;
                        }
                    });
                });
            Ui::new(
                ctx.clone(),
                ui.layer_id(),
                egui::Id::new(&*comp),
                ui.max_rect(),
                ui.clip_rect()
            )
            }); // end of panel
        let ui = panel.inner;
    }
}

pub fn SubMenu(ui: &mut Ui) -> bool{
    if ui.button("Test").clicked(){
        log::info!("Test");
        return true;
    }
    return false;
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
                ui.add(egui::Slider::new(&mut fov, 0.1..=3.0))
            });
        if cam.fov != fov {
            cam.fov = fov;
            cam.calculate_perspective();
        }
    }
}


pub fn TransformUiSystem(
    mut query: Query<&mut TransformComponent, With<TransformUiComponent>>,
    egui_state: Res<EguiState>,
){
    log::debug!("Transform ui....");
    let ctx = egui_state.ctx.clone();
    for _transform in query.iter_mut(){
        // let mut pos = transform.global_position();
        // let mut posx = pos[0];
        // let mut posy = pos[1];
        // let mut posz = pos[2];
        egui::Window::new("Transform")
            .show(&ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position");
                    // ui.add(egui::DragValue::new(&mut posx).speed(0.1).clamp_range(-100.0..=100.0));
                    // ui.add(egui::DragValue::new(&mut posy).speed(0.1).clamp_range(-100.0..=100.0));
                    // ui.add(egui::DragValue::new(&mut posz).speed(0.1).clamp_range(-100.0..=100.0));
                });
            });
    }
}