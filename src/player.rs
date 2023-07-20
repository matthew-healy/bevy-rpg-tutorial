use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{
    ascii, combat, fadeout, graphics,
    tilemap::{self, EncounterSpawner},
    util::hide,
    GameState, TILE_SIZE,
};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Overworld), spawn.run_if(run_once()))
            .add_systems(OnEnter(GameState::Overworld), show_player)
            .add_systems(OnExit(GameState::Overworld), hide::<Player>)
            .add_systems(
                Update,
                encounter_check.run_if(in_state(GameState::Overworld)),
            )
            .add_systems(Update, movement.run_if(in_state(GameState::Overworld)))
            .add_systems(
                Update,
                camera_follow
                    .after(movement)
                    .run_if(in_state(GameState::Overworld)),
            );
    }
}

#[derive(Component, Reflect)]
pub struct EncounterTracker {
    timer: Timer,
}

#[derive(Component)]
pub struct Player {
    speed: f32,
    // TODO: is it enough to only run movement when we're in the Overworld state?
    pub active: bool,
    pub experience: usize,
}

pub enum LevelUpResult {
    NoChange,
    LevelUp,
}

impl Player {
    pub fn give_exp(&mut self, experience: usize, stats: &mut combat::Stats) -> LevelUpResult {
        self.experience += experience;
        if self.experience >= 50 {
            stats.health += 2;
            stats.max_health += 2;
            stats.attack += 1;
            stats.defense += 1;
            self.experience -= 50;

            LevelUpResult::LevelUp
        } else {
            LevelUpResult::NoChange
        }
    }
}

fn movement(
    mut player_query: Query<(&Player, &mut Transform, &mut graphics::PlayerDirection)>,
    wall_query: Query<&Transform, (With<tilemap::Collider>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform, mut direction) = player_query.single_mut();

    if !player.active {
        return;
    }

    let normalised_movement = player.speed * TILE_SIZE * time.delta_seconds();

    let mut y_delta = 0.;
    if keyboard.pressed(KeyCode::W) {
        y_delta += normalised_movement;
    }
    if keyboard.pressed(KeyCode::S) {
        y_delta -= normalised_movement;
    }

    let mut x_delta = 0.;
    if keyboard.pressed(KeyCode::A) {
        x_delta -= normalised_movement;
    }
    if keyboard.pressed(KeyCode::D) {
        x_delta += normalised_movement;
    }

    let target = transform.translation + Vec3::new(0., y_delta, 0.);
    if !wall_query
        .iter()
        .any(|&wall| would_collide(target, wall.translation))
    {
        if y_delta > 0. {
            direction.0 = graphics::Direction::Up;
        } else if y_delta < 0. {
            direction.0 = graphics::Direction::Down;
        }
        transform.translation = target;
    }

    let target = transform.translation + Vec3::new(x_delta, 0., 0.);
    if !wall_query
        .iter()
        .any(|&wall| would_collide(target, wall.translation))
    {
        if x_delta > 0. {
            direction.0 = graphics::Direction::Right;
        } else if x_delta < 0. {
            direction.0 = graphics::Direction::Left;
        }
        transform.translation = target;
    }
}

fn would_collide(target_player_pos: Vec3, wall: Vec3) -> bool {
    collide(
        target_player_pos,
        Vec2::splat(TILE_SIZE * 0.9),
        wall,
        Vec2::splat(TILE_SIZE),
    )
    .is_some()
}

fn encounter_check(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &mut EncounterTracker, &Transform)>,
    encounter_query: Query<&Transform, (With<EncounterSpawner>, Without<Player>)>,
    ascii: Res<ascii::Sheet>,
    time: Res<Time>,
) {
    let (mut player, mut encounter_tracker, player_transform) = player_query.single_mut();
    let player_pos = player_transform.translation;
    if encounter_query
        .iter()
        .any(|&encounter_tile| would_collide(player_pos, encounter_tile.translation))
    {
        encounter_tracker.timer.tick(time.delta());
        if encounter_tracker.timer.just_finished() {
            player.active = false;
            fadeout::create(&mut commands, GameState::Combat, &ascii)
        }
    }
}

fn show_player(mut query: Query<(&mut Player, &mut Visibility)>) {
    if let Ok((mut player, mut visibility)) = query.get_single_mut() {
        player.active = true;
        *visibility = Visibility::Inherited;
    }
}

fn spawn(mut commands: Commands, characters: Res<graphics::CharacterSheet>) {
    let initial_direction = graphics::Direction::Down;
    let initial_frames = characters.player_frames.get(&initial_direction).unwrap();
    commands
        // TODO: DirectionalAnimationBundle to configure all movement-related stuff?
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: initial_frames[0],
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_xyz(2. * TILE_SIZE, -2. * TILE_SIZE, 900.),
            texture_atlas: characters.handle.clone(),
            ..Default::default()
        })
        .insert(graphics::FrameAnimation {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            frames: initial_frames.to_vec(),
            current_frame: 0,
        })
        .insert(graphics::PlayerDirection(initial_direction))
        .insert(Name::new("Player"))
        .insert(Player {
            speed: 3.,
            active: true,
            experience: 0,
        })
        .insert(combat::Stats {
            health: 10,
            max_health: 10,
            attack: 2,
            defense: 1,
        })
        .insert(EncounterTracker {
            timer: Timer::from_seconds(1., TimerMode::Repeating),
        });
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
