mod ascii;
mod audio;
mod combat;
mod debug;
mod fadeout;
mod player;
mod tilemap;
mod util;

use bevy::{prelude::*, render::camera::ScalingMode, window::WindowResolution};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16. / 9.;
pub const TILE_SIZE: f32 = 0.1;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, States)]
enum GameState {
    #[default]
    Overworld,
    Combat,
}

fn main() {
    let height = 900.;
    App::new()
        .add_state::<GameState>()
        .insert_resource(ClearColor(CLEAR))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(height * RESOLUTION, height),
                        title: "Bevy Tutorial".to_string(),
                        present_mode: bevy::window::PresentMode::Fifo,
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_systems(Startup, spawn_camera)
        .add_plugins(audio::Plugin)
        .add_plugins(ascii::Plugin)
        .add_plugins(player::Plugin)
        .add_plugins(debug::Plugin)
        .add_plugins(tilemap::Plugin)
        .add_plugins(combat::Plugin)
        .add_plugins(fadeout::Plugin)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();

    camera.projection.scaling_mode = ScalingMode::Fixed {
        width: 2. * RESOLUTION,
        height: 2.,
    };

    commands.spawn(camera);
}
