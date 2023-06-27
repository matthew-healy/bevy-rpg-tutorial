use bevy::prelude::*;

use crate::TILE_SIZE;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load.in_base_set(StartupSet::PreStartup));
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
                ..Default::default()
            },
            ..Default::default()
        })
        .id()
}
