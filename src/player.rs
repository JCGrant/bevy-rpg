use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_inspector_egui::Inspectable;

use crate::{
    ascii::{spawn_ascii_sprite, AsciiSheet},
    tilemap::TileCollider,
    TILE_SIZE,
};

pub struct PlayerPlugin;

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_movement.label("movement"))
            .add_system(camera_follow.after("movement"));
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation = player_transform.translation;
}

fn player_movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    wall_query: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform) = player_query.single_mut();
    let speed = player.speed * TILE_SIZE * time.delta_seconds();
    let mut velocity = Vec2::new(0.0, 0.0);
    if keyboard.pressed(KeyCode::W) {
        velocity.y += speed;
    }
    if keyboard.pressed(KeyCode::S) {
        velocity.y -= speed;
    }
    if keyboard.pressed(KeyCode::A) {
        velocity.x -= speed;
    }
    if keyboard.pressed(KeyCode::D) {
        velocity.x += speed;
    }
    let target = transform.translation + Vec3::new(velocity.x, 0.0, 0.0);
    if wall_collision_check(target, &wall_query) {
        transform.translation = target;
    }
    let target = transform.translation + Vec3::new(0.0, velocity.y, 0.0);
    if wall_collision_check(target, &wall_query) {
        transform.translation = target;
    }
}

fn wall_collision_check(
    target_player_pos: Vec3,
    wall_query: &Query<&Transform, (With<TileCollider>, Without<Player>)>,
) -> bool {
    for wall_transform in wall_query.iter() {
        let collision = collide(
            target_player_pos,
            Vec2::splat(TILE_SIZE * 0.9),
            wall_transform.translation,
            Vec2::splat(TILE_SIZE),
        );
        if collision.is_some() {
            return false;
        }
    }
    return true;
}

fn spawn_player(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let player = spawn_ascii_sprite(
        &mut commands,
        &ascii,
        1,
        Color::rgb(0.3, 0.3, 0.9),
        Vec3::new(2.0 * TILE_SIZE, -2.0 * TILE_SIZE, 900.0),
    );
    commands
        .entity(player)
        .insert(Name::new("Player"))
        .insert(Player { speed: 3.0 });

    let background = spawn_ascii_sprite(
        &mut commands,
        &ascii,
        0,
        Color::rgb(0.5, 0.5, 0.5),
        Vec3::new(0.0, 0.0, -1.0),
    );
    commands.entity(background).insert(Name::new("Background"));

    commands.entity(player).push_children(&[background]);
}
