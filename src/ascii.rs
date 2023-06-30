use bevy::prelude::*;

use crate::TILE_SIZE;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load.in_base_set(StartupSet::PreStartup))
            .insert_resource(NinesliceIndices {
                center: 2 * 16,
                upper_left: 13 * 16 + 10,
                upper_right: 11 * 16 + 15,
                lower_left: 12 * 16,
                lower_right: 13 * 16 + 9,
                horizontal: 12 * 16 + 4,
                vertical: 11 * 16 + 3,
            });
    }
}

#[derive(Resource)]
pub struct Sheet(pub Handle<TextureAtlas>);

fn load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("ascii.png");
    let atlas =
        TextureAtlas::from_grid(image, Vec2::splat(9.), 16, 16, Some(Vec2::splat(2.)), None);

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(Sheet(atlas_handle))
}

pub fn spawn_sprite(
    commands: &mut Commands,
    ascii: &Sheet,
    index: usize,
    color: Color,
    translation: Vec3,
    scale: Vec3,
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(index);
    sprite.color = color;
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation,
                scale,
                ..Default::default()
            },
            ..Default::default()
        })
        .id()
}

#[derive(Component)]
pub struct Text;

pub fn spawn_text(commands: &mut Commands, ascii: &Sheet, text: &str, left_center: Vec3) -> Entity {
    let color = Color::rgb(0.8, 0.8, 0.8);
    let mut character_sprites = Vec::new();
    for (i, char) in text.chars().enumerate() {
        assert!(char as usize <= 255);
        character_sprites.push(spawn_sprite(
            commands,
            ascii,
            char as usize,
            color,
            Vec3::new(i as f32 * TILE_SIZE, 0., 0.),
            Vec3::splat(1.),
        ))
    }

    commands
        .spawn_empty()
        .insert(Name::new(format!("Text - {}", text)))
        .insert(Text)
        .insert(SpatialBundle::default())
        .insert(Transform {
            translation: left_center,
            ..Default::default()
        })
        .push_children(&character_sprites)
        .id()
}

#[derive(Resource)]
pub struct NinesliceIndices {
    center: usize,
    upper_left: usize,
    upper_right: usize,
    lower_left: usize,
    lower_right: usize,
    horizontal: usize,
    vertical: usize,
}

#[derive(Component)]
pub struct Nineslice;

pub fn spawn_nineslice(
    commands: &mut Commands,
    ascii: &Sheet,
    indices: &NinesliceIndices,
    width: f32,
    height: f32,
) -> Entity {
    assert!(width >= 2.);
    assert!(height >= 2.);

    let color = Color::rgb(0.3, 0.3, 0.9);
    let mut sprites = Vec::new();

    let left = (-width / 2. + 0.5) * TILE_SIZE;
    let right = -left;
    let down = (-height / 2. + 0.5) * TILE_SIZE;
    let up = -down;

    let calls = [
        (
            indices.center,
            Vec3::splat(0.),
            Vec3::new(width - 2., height - 2., 0.),
        ),
        (indices.upper_left, Vec3::new(left, up, 0.), Vec3::splat(1.)),
        (
            indices.vertical,
            Vec3::new(left, 0., 0.),
            Vec3::new(1., height - 2., 1.),
        ),
        (
            indices.lower_left,
            Vec3::new(left, down, 0.),
            Vec3::splat(1.),
        ),
        (
            indices.horizontal,
            Vec3::new(0., down, 0.),
            Vec3::new(width - 2., 1., 1.),
        ),
        (
            indices.horizontal,
            Vec3::new(0., up, 0.),
            Vec3::new(width - 2., 1., 1.),
        ),
        (
            indices.upper_right,
            Vec3::new(right, up, 0.),
            Vec3::splat(1.),
        ),
        (
            indices.vertical,
            Vec3::new(right, 0., 0.),
            Vec3::new(1., height - 2., 1.),
        ),
        (
            indices.lower_right,
            Vec3::new(right, down, 0.),
            Vec3::splat(1.),
        ),
    ];

    for (index, translation, scale) in calls.iter() {
        sprites.push(spawn_sprite(
            commands,
            ascii,
            *index,
            color,
            *translation,
            *scale,
        ));
    }

    commands
        .spawn_empty()
        .insert(Nineslice)
        .insert(Name::new("NineSpriteBox"))
        .insert(SpatialBundle::default())
        .push_children(&sprites)
        .id()
}
