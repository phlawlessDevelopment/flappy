use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use rand::Rng;
use std::time::Duration;

mod camera;

use crate::camera::CameraPlugin;

#[derive(Debug, Default, Component)]
struct Player {
    is_jumping: bool,
    jump_timer: Timer,
}
#[derive(Debug, Default, Resource)]
struct StoredVelocity(Vec2);

#[derive(Debug, Default, Resource)]
struct Score(u32);

#[derive(Debug, Default, Component)]
struct Pipe {
    used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppSate {
    Playing,
    Dead,
    Paused,
}
#[derive(Resource)]
struct PipeSpawnConfig {
    timer: Timer,
}

fn main() {
    App::new()
        .init_resource::<StoredVelocity>()
        .init_resource::<Score>()
        .insert_resource(PipeSpawnConfig {
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
        })
        .add_loopless_state(AppSate::Playing)
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_player)
        .add_startup_system(create_gui)
        .add_system(player_input_playing.run_in_state(AppSate::Playing))
        .add_system(spawn_pipe.run_in_state(AppSate::Playing))
        .add_system(apply_y_velocity.run_in_state(AppSate::Playing))
        .add_system(check_player_on_screen.run_in_state(AppSate::Playing))
        .add_system(handle_player_contacts.run_in_state(AppSate::Playing))
        .add_system(update_gui.run_in_state(AppSate::Playing))
        .add_system(check_pipes.run_in_state(AppSate::Playing))
        .add_system(player_input_paused.run_in_state(AppSate::Paused))
        .add_system(handle_player_dead.run_in_state(AppSate::Dead))
        .run();
}

fn create_gui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TextBundle::from_section(
        "0",
        TextStyle {
            font: asset_server.load("fonts/SourceCodePro-Bold.otf"),
            font_size: 30.0,
            color: Color::WHITE,
        },
    ));
}
fn update_gui(score: Res<Score>, mut text_q: Query<&mut Text>) {
    text_q.single_mut().sections[0].value = score.0.to_string();
}

fn spawn_pipe(
    mut commands: Commands,
    mut pipe_spawn: ResMut<PipeSpawnConfig>,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    if !pipe_spawn.timer.finished() {
        pipe_spawn.timer.tick(time.delta());
    } else {
        let window = windows.primary_mut();
        pipe_spawn.timer.reset();
        let mut rng = rand::thread_rng();
        let pipe_scale = 7.5;
        let offset = 256.0;
        let half_pipe_height = 32.0 * (pipe_scale / 2.0);
        let top_pipe_y: f32 = rng.gen_range(offset..window.height() / 2.0 + half_pipe_height);
        let bot_pipe_y = top_pipe_y - (window.height());
        // let bot_pipe_y = rng.gen_range(-window.height() / 2.0 - half_pipe_height..-offset);
        commands
            .spawn(RigidBody::KinematicVelocityBased)
            .insert(Collider::cuboid(32.0, 32.0))
            .insert(Velocity {
                linvel: Vec2 { x: -100.0, y: 0.0 },
                ..default()
            })
            .insert(Pipe::default())
            .insert(SpriteBundle {
                texture: asset_server.load("sprites/WaterTile.png"),
                transform: Transform::from_xyz(window.width() * 0.6, top_pipe_y, 0.0)
                    .with_scale(Vec3::new(1.0, 7.5, 1.0)),
                ..default()
            });
        commands
            .spawn(RigidBody::KinematicVelocityBased)
            .insert(Collider::cuboid(32.0, 32.0))
            .insert(Velocity {
                linvel: Vec2 { x: -100.0, y: 0.0 },
                ..default()
            })
            .insert(Pipe::default())
            .insert(SpriteBundle {
                texture: asset_server.load("sprites/WaterTile.png"),
                transform: Transform::from_xyz(window.width() * 0.6, bot_pipe_y, 0.0)
                    .with_scale(Vec3::new(1.0, 7.5, 1.0)),
                ..default()
            });
    }
}
fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    /* Create the player ball. */
    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Velocity { ..default() })
        .insert(Collider::ball(32.0))
        .insert(SpriteBundle {
            texture: asset_server.load("sprites/WaterPlayer.png"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Player {
            jump_timer: Timer::new(Duration::from_secs_f32(0.25), TimerMode::Once),
            ..default()
        });
}

fn check_pipes(
    mut pipes_q: Query<(Entity, &mut Pipe, &Transform)>,
    mut score: ResMut<Score>,
    windows: Res<Windows>,
    mut commands: Commands,
) {
    let window = windows.primary();
    for (entity, mut pipe, transform) in pipes_q.iter_mut() {
        if !(pipe.used) && (transform.translation.x) < 0.0 && (transform.translation.y) > 0.0 {
            pipe.used = true;
            score.0 += 1;
        }
        if transform.translation.x < -window.width() / 2.0 {
            commands.entity(entity).despawn_recursive()
        }
    }
}

fn apply_y_velocity(mut player_q: Query<(&mut Player, &mut Velocity)>, time: Res<Time>) {
    let (mut player, mut player_velocity) = player_q.single_mut();
    if player.is_jumping {
        if player.jump_timer.finished() {
            player.is_jumping = false;
        } else {
            player.jump_timer.tick(time.delta());
            player_velocity.linvel += Vec2::new(0.0, 10.0);
        }
    } else {
        player_velocity.linvel = Vec2::new(player_velocity.linvel.x, -25.0);
    }
}
fn check_player_on_screen(
    mut player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    windows: ResMut<Windows>,
) {
    let transform = player_q.single_mut();
    let min = windows.primary().height() / 2.0;
    if transform.translation.y < -min {
        commands.insert_resource(NextState(AppSate::Dead));
    }
}
fn player_input_playing(
    keys: Res<Input<KeyCode>>,
    mut player_q: Query<(&mut Player, &mut Velocity)>,
    mut stored_vel: ResMut<StoredVelocity>,
    mut commands: Commands,
) {
    let (mut player, mut player_velocity) = player_q.single_mut();
    if keys.just_pressed(KeyCode::Space) {
        player.is_jumping = true;
        player.jump_timer.reset();
        // player_velocity.linvel = Vec2::new(player_velocity.linvel.x, 0.0);
    }
    if keys.just_pressed(KeyCode::Escape) {
        stored_vel.0 = player_velocity.linvel;
        player_velocity.linvel = Vec2::ZERO;
        commands.insert_resource(NextState(AppSate::Paused));
    }
}
fn player_input_paused(
    keys: Res<Input<KeyCode>>,
    mut player_q: Query<(&mut Player, &mut Velocity)>,
    mut commands: Commands,
    stored_vel: ResMut<StoredVelocity>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let (_player, mut player_velocity) = player_q.single_mut();
        player_velocity.linvel = stored_vel.0;
        commands.insert_resource(NextState(AppSate::Playing));
    }
}

fn handle_player_contacts(
    rapier_context: Res<RapierContext>,
    mut player_q: Query<(Entity, &mut Velocity), (With<Player>, Without<Pipe>)>,
    mut pipe_q: Query<(Entity, &mut Velocity), (With<Pipe>, Without<Player>)>,
    mut commands: Commands,
) {
    let (player, mut velocity) = player_q.single_mut();
    for _contact in rapier_context.contacts_with(player) {
        for (_pipe, mut pipe_vel) in pipe_q.iter_mut() {
            pipe_vel.linvel = Vec2::ZERO;
        }
        velocity.linvel = Vec2::ZERO;
        commands.insert_resource(NextState(AppSate::Dead));
    }
}

fn handle_player_dead(mut player_q: Query<(&mut Player, &mut Velocity)>, _time: Res<Time>) {
    let (_player, mut player_velocity) = player_q.single_mut();
    player_velocity.linvel -= Vec2::new(0.0, 20.0);
}
