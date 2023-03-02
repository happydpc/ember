use bevy_ecs::{world::World, system::Commands};
use bevy_hierarchy::AddChild;

use crate::core::plugins::components::{
    AppInterfaceFlag,
    MainMenuComponent,
    ui::main_menu_component::{
        LeftPanelComponent,
        RightPanelComponent,
        BottomPanelComponent
    },
    FileSubMenuComponent,
    SceneGraphComponent
};

pub fn initalize_editor_interface(
    mut commands: Commands
){
    log::info!("Setting up editor interface");
    let app_interface_entity = commands.spawn_empty().insert(AppInterfaceFlag::default()).id();
    let main_menu_entity = commands.spawn_empty().insert(MainMenuComponent::default()).id();
    let left_panel_component = commands.spawn_empty().insert(LeftPanelComponent::default()).id();
    let right_panel_component = commands.spawn_empty().insert(RightPanelComponent::default()).id();
    let bottom_panel_component = commands.spawn_empty().insert(BottomPanelComponent::default()).id();
    let file_sub_menu_entity = commands.spawn_empty().insert(FileSubMenuComponent::default()).id();
    let scene_graph_entity = commands.spawn_empty().insert(SceneGraphComponent::default()).id();

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


}