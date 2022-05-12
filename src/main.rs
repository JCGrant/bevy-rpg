use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PresentMode;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const TILE_SIZE: f32 = 0.1;

mod ascii;
mod audio;
mod combat;
mod debug;
mod fadeout;
mod graphics;
mod npc;
mod player;
mod startmenu;
mod tilemap;

use ascii::AsciiPlugin;
use audio::GameAudioPlugin;
use combat::CombatPlugin;
use debug::DebugPlugin;
use fadeout::FadeoutPlugin;
use graphics::GraphicsPlugin;
use npc::NpcPlugin;
use player::PlayerPlugin;
use startmenu::MainMenuPlugin;
use tilemap::TileMapPlugin;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
    StartMenu,
    Overworld,
    Combat,
}

fn main() {
    let height = 900.0;
    App::new()
        .add_state(GameState::StartMenu)
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: height * RESOLUTION,
            height,
            title: "Bevy RPG".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_camera)
        .add_plugin(PlayerPlugin)
        .add_plugin(FadeoutPlugin)
        .add_plugin(CombatPlugin)
        .add_plugin(AsciiPlugin)
        .add_plugin(NpcPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(GameAudioPlugin)
        .add_plugin(GraphicsPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(TileMapPlugin)
        .run()
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;
    camera.orthographic_projection.right = 1.0 * RESOLUTION;
    camera.orthographic_projection.left = -1.0 * RESOLUTION;
    camera.orthographic_projection.scaling_mode = ScalingMode::None;
    commands.spawn_bundle(camera);
}
