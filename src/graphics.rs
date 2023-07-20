use bevy::{prelude::*, utils::HashMap};

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
    bat_frames: [usize; 3],
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

pub fn spawn_bat(
    commands: &mut Commands,
    characters: &CharacterSheet,
    translation: Vec3,
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(characters.bat_frames[0]);
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
            frames: characters.bat_frames.to_vec(),
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
    let atlas =
        TextureAtlas::from_grid(image, Vec2::splat(16.), 12, 8, Some(Vec2::splat(2.)), None);
    let handle = texture_atlases.add(atlas);

    let columns = 12;

    let player_frames = {
        let mut f = HashMap::new();
        f.insert(
            Direction::Down,
            [columns * 0 + 0, columns * 0 + 1, columns * 0 + 2],
        );
        f.insert(
            Direction::Left,
            [columns * 1 + 0, columns * 1 + 1, columns * 1 + 2],
        );
        f.insert(
            Direction::Right,
            [columns * 2 + 0, columns * 2 + 1, columns * 2 + 2],
        );
        f.insert(
            Direction::Up,
            [columns * 3 + 0, columns * 3 + 1, columns * 3 + 2],
        );
        f
    };

    commands.insert_resource(CharacterSheet {
        handle,
        player_frames,
        bat_frames: [12 * 4 + 3, 12 * 4 + 4, 12 * 4 + 5],
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
