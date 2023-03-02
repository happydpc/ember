use bevy_ecs::{world::World, system::Commands};
use bevy_hierarchy::AddChild;

use crate::core::plugins::components::{
    AppInterfaceFlag,
    ui::main_menu_component::{
        EditorUiState, EntityInspectorComponent, UiPanelComponent
    },
    FileSubMenuComponent,
    SceneGraphComponent
};

pub fn initalize_editor_interface(
    mut commands: Commands
){
    log::info!("Setting up editor interface");
    commands.insert_resource(EditorUiState::default());

    let app_interface_entity = commands.spawn_empty().insert(AppInterfaceFlag::default()).id();
    let main_menu_entity = commands.spawn_empty().insert(UiPanelComponent::top()).id();
    let left_panel_component = commands.spawn_empty().insert(UiPanelComponent::left()).id();
    let right_panel_component = commands.spawn_empty().insert(UiPanelComponent::right()).id();
    let bottom_panel_component = commands.spawn_empty().insert(UiPanelComponent::bottom()).id();
    let file_sub_menu_entity = commands.spawn_empty().insert(FileSubMenuComponent::default()).id();
    let scene_graph_entity = commands.spawn_empty().insert(SceneGraphComponent::default()).id();
    let entity_inspector_entity = commands.spawn_empty().insert(EntityInspectorComponent::default()).id();

    // set parent relationships

    commands.add(AddChild{
        parent: app_interface_entity,
        child: main_menu_entity
    });

    commands.add(AddChild{
        parent: main_menu_entity,
        child: file_sub_menu_entity
    });

    commands.add(AddChild{
        parent: app_interface_entity,
        child: left_panel_component
    });

    commands.add(AddChild{
        parent: app_interface_entity,
        child: right_panel_component
    });

    commands.add(AddChild{
        parent: app_interface_entity,
        child: bottom_panel_component
    });

    commands.add(AddChild{
        parent: left_panel_component,
        child: scene_graph_entity
    });

    commands.add(AddChild{
        parent: right_panel_component,
        child: entity_inspector_entity
    });

}