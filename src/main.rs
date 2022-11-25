use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod camera;

use crate::camera::CameraPlugin;

struct KeyboardDirectionalInput(Vec2);

#[derive(Debug, Default, Component)]
struct Player;

fn main() {
    App::new()
        .add_event::<KeyboardDirectionalInput>()
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_world)
        .add_startup_system(setup_player)
        .add_system(keyboard_input)
        .add_system(move_player)
        .run();
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(32.0, 32.0))
        .insert(SpriteBundle {
            texture: asset_server.load("sprites/WaterTile.png"),
            transform: Transform::from_xyz(0.0, -100.0, 0.0).with_scale(Vec3::new(8.0, 1.0, 1.0)),
            ..default()
        });
}
fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    /* Create the player ball. */
    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Velocity::default())
        .insert(Collider::ball(32.0))
        .insert(Restitution::coefficient(0.7))
        .insert(SpriteBundle {
            texture: asset_server.load("sprites/WaterPlayer.png"),
            transform: Transform::from_translation(Vec3::default()),
            ..default()
        })
        .insert(Player);
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<KeyboardDirectionalInput>,
) {
    if keys.just_pressed(KeyCode::W) {
        event_writer.send(KeyboardDirectionalInput(Vec2::new(0.0, 1.0)))
    }
    if keys.just_pressed(KeyCode::A) {
        event_writer.send(KeyboardDirectionalInput(Vec2::new(-1.0, 0.0)))
    }
    if keys.just_pressed(KeyCode::S) {
        event_writer.send(KeyboardDirectionalInput(Vec2::new(0.0, -1.0)))
    }
    if keys.just_pressed(KeyCode::D) {
        event_writer.send(KeyboardDirectionalInput(Vec2::new(1.0, 0.0)))
    }
}
fn move_player(
    mut event_reader: EventReader<KeyboardDirectionalInput>,
    mut player_q: Query<&mut Velocity, With<Player>>,
) {
    for event in event_reader.iter() {
        let mut player_velocity = player_q.single_mut();
        player_velocity.linvel = Vec2::new(event.0.x * 100.0, event.0.y * 100.0);
    }
}
