use bevy::prelude::*;

pub struct CameraPlugin;

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
	commands
	.spawn(Camera2dBundle::default())
	.insert(MainCamera);
}

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system_to_stage(StartupStage::PreStartup, setup);
	}

}
