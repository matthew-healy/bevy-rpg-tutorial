use bevy::{prelude::*, utils::HashMap};

use crate::combat;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load)
            .add_systems(PostUpdate, animate)
            .add_systems(Update, update_player);
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Resource)]
pub struct CharacterSheet {
    pub handle: Handle<TextureAtlas>,
    pub player_frames: HashMap<Direction, [usize; 3]>,
    pub enemy_frames: HashMap<combat::EnemyType, [usize; 3]>,
}

impl CharacterSheet {
    pub fn get_player_frames(&self, direction: &Direction) -> &[usize; 3] {
        self.player_frames.get(direction).unwrap()
    }

    pub fn get_enemy_frames(&self, typ: &combat::EnemyType) -> &[usize; 3] {
        self.enemy_frames.get(typ).unwrap()
    }
}

#[derive(Component)]
pub struct PlayerDirection(pub Direction);

#[derive(Component)]
pub struct FrameAnimation {
    pub timer: Timer,
    pub frames: Vec<usize>,
    pub current_frame: usize,
}

impl FrameAnimation {
    fn frame(&self) -> usize {
        self.frames[self.current_frame]
    }

    fn tick(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames.len();
    }
}

pub fn spawn_enemy(
    commands: &mut Commands,
    typ: &combat::EnemyType,
    characters: &CharacterSheet,
    translation: Vec3,
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(characters.get_enemy_frames(typ)[0]);
    sprite.custom_size = Some(Vec2::splat(0.5));

    commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: characters.handle.clone(),
            transform: Transform {
                translation,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FrameAnimation {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            frames: characters.get_enemy_frames(typ).to_vec(),
            current_frame: 0,
        })
        .id()
}

fn load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("characters.png");
    let columns = 12;
    let atlas = TextureAtlas::from_grid(
        image,
        Vec2::splat(16.),
        columns,
        8,
        Some(Vec2::splat(2.)),
        None,
    );
    let handle = texture_atlases.add(atlas);

    let player_frames = {
        let mut f = HashMap::new();
        f.insert(
            Direction::Down,
            [columns * 0 + 6, columns * 0 + 7, columns * 0 + 8],
        );
        f.insert(
            Direction::Left,
            [columns * 1 + 6, columns * 1 + 7, columns * 1 + 8],
        );
        f.insert(
            Direction::Right,
            [columns * 2 + 6, columns * 2 + 7, columns * 2 + 8],
        );
        f.insert(
            Direction::Up,
            [columns * 3 + 6, columns * 3 + 7, columns * 3 + 8],
        );
        f
    };

    let enemy_frames = {
        let mut f = HashMap::new();
        f.insert(
            combat::EnemyType::Bat,
            [columns * 4 + 3, columns * 4 + 4, columns * 4 + 5],
        );
        f.insert(
            combat::EnemyType::Ghost,
            [columns * 4 + 6, columns * 4 + 7, columns * 4 + 8],
        );
        f
    };

    commands.insert_resource(CharacterSheet {
        handle,
        player_frames,
        enemy_frames,
    });
}

fn update_player(
    mut sprites: Query<(&PlayerDirection, &mut FrameAnimation), Changed<PlayerDirection>>,
    characters: Res<CharacterSheet>,
) {
    for (direction, mut animation) in sprites.iter_mut() {
        animation.frames = characters
            .player_frames
            .get(&direction.0)
            .expect("animations for each direction are inserted in `load`")
            .to_vec();
    }
}

fn animate(
    mut sprites_query: Query<(&mut TextureAtlasSprite, &mut FrameAnimation)>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation) in sprites_query.iter_mut() {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            animation.tick();
            sprite.index = animation.frame();
        }
    }
}
