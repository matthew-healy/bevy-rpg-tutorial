mod ascii;
mod debug;
mod player;
mod tilemap;

use bevy::{prelude::*, render::camera::ScalingMode, window::WindowResolution};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16. / 9.;
// TODO: in the tutoral this is set to `0.1`, but if I do that here then we
// end up with a sprite so small it's essentially invisible. Setting the size
// to something else seems to work for now, but it'd be good to know why this
// doesn't work.
pub const TILE_SIZE: f32 = 45.;

fn main() {
    let height = 900.;
    App::new()
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
        .add_startup_system(spawn_camera)
        .add_plugin(ascii::Plugin)
        .add_plugin(player::Plugin)
        .add_plugin(debug::Plugin)
        .add_plugin(tilemap::Plugin)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();

    camera.projection.area = Rect::new(-1. * RESOLUTION, -1., 1. * RESOLUTION, 1.);
    // TODO:  I *think* this is the same as ScalingMode::None from the tutorial.
    // That was replaced with ScalingMode::Fixed in Bevy 0.10, but that takes a
    // width & height, which is surely just going to be the screen size?
    camera.projection.scaling_mode = ScalingMode::WindowSize(1.0);

    commands.spawn(camera);
}
