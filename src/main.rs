use bevy::{prelude::*, render::camera::ScalingMode, window::WindowResolution};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16. / 9.;

fn main() {
    let height = 900.;
    App::new()
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
        // a resource is something that only exists once for the whole game
        // which makes it easy to access the single instance from elsewhere.
        .insert_resource(ClearColor(CLEAR))
        .add_startup_system(load_ascii.in_base_set(StartupSet::PreStartup))
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .run();
}

fn spawn_player(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let mut sprite = TextureAtlasSprite::new(1);
    sprite.color = Color::rgb(0.3, 0.3, 0.9);
    // TODO: in the tutoral this is set to `Vec2::splat(1.)`, but if I do that
    // here then we end up with a sprite so small it's essentially invisible.
    // Setting the size to something else seems to work for now, but it'd be
    // good to know why this doesn't work.
    sprite.custom_size = Some(Vec2::splat(450.));

    let player = commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(0., 0., 900.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Player"))
        .id();

    let mut background_sprite = TextureAtlasSprite::new(0);
    background_sprite.color = Color::rgb(0.5, 0.5, 0.5);
    background_sprite.custom_size = Some(Vec2::splat(450.));

    let bg = commands
        .spawn(SpriteSheetBundle {
            sprite: background_sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(0., 0., -1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Background"))
        .id();

    commands.entity(player).push_children(&[bg]);
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

#[derive(Resource)]
struct AsciiSheet(Handle<TextureAtlas>);

fn load_ascii(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("ascii.png");
    let atlas =
        TextureAtlas::from_grid(image, Vec2::splat(9.), 16, 16, Some(Vec2::splat(2.)), None);

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(AsciiSheet(atlas_handle))
}
