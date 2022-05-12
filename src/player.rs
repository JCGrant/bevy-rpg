use bevy::{prelude::*, render::camera::Camera2d, sprite::collide_aabb::collide};
use bevy_inspector_egui::Inspectable;

use crate::{
    ascii::AsciiSheet,
    combat::CombatStats,
    fadeout::create_fadeout,
    graphics::{CharacterSheet, FacingDirection, FrameAnimation, PlayerGraphics},
    tilemap::{EncounterSpawner, TileCollider},
    GameState, TILE_SIZE,
};

pub struct PlayerPlugin;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EncounterTracker {
    timer: Timer,
}

#[derive(Component, Inspectable)]
pub struct Player {
    pub active: bool,
    speed: f32,
    just_moved: bool,
    pub exp: usize,
}

impl Player {
    pub fn give_exp(&mut self, exp: usize, stats: &mut CombatStats) -> bool {
        self.exp += exp;
        if self.exp >= 50 {
            stats.health += 2;
            stats.max_health += 2;
            stats.attack += 1;
            stats.defence += 1;
            self.exp -= 50;
            return true;
        }
        false
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_resume(GameState::Overworld).with_system(show_player))
            .add_system_set(SystemSet::on_pause(GameState::Overworld).with_system(hide_player))
            .add_system_set(
                SystemSet::on_update(GameState::Overworld)
                    .with_system(player_encounter_checking.after(player_movement))
                    .with_system(camera_follow.after(player_movement))
                    .with_system(player_movement),
            )
            .add_system_set(SystemSet::on_enter(GameState::Overworld).with_system(spawn_player));
    }
}

fn hide_player(
    mut player_query: Query<&mut Visibility, With<Player>>,
    children_query: Query<&Children, With<Player>>,
    mut child_visibility_query: Query<&mut Visibility, Without<Player>>,
) {
    let mut player_vis = player_query.single_mut();
    player_vis.is_visible = false;
    if let Ok(children) = children_query.get_single() {
        for child in children.iter() {
            if let Ok(mut child_vis) = child_visibility_query.get_mut(*child) {
                child_vis.is_visible = false;
            }
        }
    }
}

fn show_player(
    mut player_query: Query<(&mut Player, &mut Visibility)>,
    children_query: Query<&Children, With<Player>>,
    mut child_visibility_query: Query<&mut Visibility, Without<Player>>,
) {
    let (mut player, mut player_vis) = player_query.single_mut();
    player.active = true;
    player_vis.is_visible = true;
    if let Ok(children) = children_query.get_single() {
        for child in children.iter() {
            if let Ok(mut child_vis) = child_visibility_query.get_mut(*child) {
                child_vis.is_visible = true;
            }
        }
    }
}

fn player_encounter_checking(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &mut EncounterTracker, &Transform)>,
    encounter_query: Query<&Transform, With<EncounterSpawner>>,
    ascii: Res<AsciiSheet>,
    time: Res<Time>,
) {
    let (mut player, mut encounter_tracker, transform) = player_query.single_mut();
    let player_pos = transform.translation;
    if player.just_moved
        && encounter_query
            .iter()
            .any(|&transform| tile_collision_check(player_pos, transform.translation))
    {
        encounter_tracker.timer.tick(time.delta());
        if encounter_tracker.timer.just_finished() {
            player.active = false;
            create_fadeout(&mut commands, Some(GameState::Combat), &ascii);
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera2d>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation = player_transform.translation;
}

fn player_movement(
    mut player_query: Query<(
        &mut Player,
        &mut Transform,
        &mut PlayerGraphics,
        &mut FrameAnimation,
    )>,
    wall_query: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform, mut graphics, mut animation) = player_query.single_mut();
    player.just_moved = false;
    if !player.active {
        return;
    }
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
    if velocity == Vec2::ZERO {
        animation.active = false;
        return;
    }
    animation.active = true;
    player.just_moved = true;
    let target = transform.translation + Vec3::new(0.0, velocity.y, 0.0);
    if !wall_query
        .iter()
        .any(|&transform| tile_collision_check(target, transform.translation))
    {
        transform.translation = target;
        if velocity.y != 0.0 {
            if velocity.y > 0.0 {
                graphics.facing = FacingDirection::Up;
            } else {
                graphics.facing = FacingDirection::Down;
            }
        }
    }
    let target = transform.translation + Vec3::new(velocity.x, 0.0, 0.0);
    if !wall_query
        .iter()
        .any(|&transform| tile_collision_check(target, transform.translation))
    {
        transform.translation = target;
        if velocity.x != 0.0 {
            if velocity.x > 0.0 {
                graphics.facing = FacingDirection::Right;
            } else {
                graphics.facing = FacingDirection::Left;
            }
        }
    }
}

fn tile_collision_check(target_player_pos: Vec3, tile_pos: Vec3) -> bool {
    collide(
        target_player_pos,
        Vec2::splat(TILE_SIZE * 0.9),
        tile_pos,
        Vec2::splat(TILE_SIZE),
    )
    .is_some()
}

fn spawn_player(mut commands: Commands, characters: Res<CharacterSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: characters.player_down[0],
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(2.0 * TILE_SIZE, -2.0 * TILE_SIZE, 900.0),
            texture_atlas: characters.handle.clone(),
            ..default()
        })
        .insert(FrameAnimation {
            active: false,
            timer: Timer::from_seconds(0.2, true),
            frames: characters.player_down.to_vec(),
            current_frame: 0,
        })
        .insert(PlayerGraphics {
            facing: FacingDirection::Down,
        })
        .insert(Name::new("Player"))
        .insert(Player {
            speed: 3.0,
            active: true,
            just_moved: false,
            exp: 0,
        })
        .insert(EncounterTracker {
            timer: Timer::from_seconds(1.0, true),
        })
        .insert(CombatStats {
            health: 10,
            max_health: 10,
            attack: 2,
            defence: 1,
        });
}
