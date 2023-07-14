use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{
    ascii, combat, fadeout,
    tilemap::{self, EncounterSpawner},
    util::hide,
    GameState, TILE_SIZE,
};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
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
    active: bool,
}

fn movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    wall_query: Query<&Transform, (With<tilemap::Collider>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform) = player_query.single_mut();

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

    let target = transform.translation + Vec3::new(x_delta, 0., 0.);
    if !wall_query
        .iter()
        .any(|&wall| would_collide(target, wall.translation))
    {
        transform.translation = target;
    }

    let target = transform.translation + Vec3::new(0., y_delta, 0.);
    if !wall_query
        .iter()
        .any(|&wall| would_collide(target, wall.translation))
    {
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
    let (mut player, mut visibility) = query.single_mut();
    player.active = true;
    *visibility = Visibility::Inherited;
}

fn spawn(mut commands: Commands, ascii: Res<ascii::Sheet>) {
    let player = ascii::spawn_sprite(
        &mut commands,
        ascii.as_ref(),
        1,
        Color::rgb(0.3, 0.3, 0.9),
        Vec3::new(2. * TILE_SIZE, -2. * TILE_SIZE, 900.),
        Vec3::splat(1.),
    );

    commands
        .entity(player)
        .insert(Name::new("Player"))
        .insert(Player {
            speed: 3.,
            active: true,
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

    let background = ascii::spawn_sprite(
        &mut commands,
        ascii.as_ref(),
        0,
        Color::rgb(0.5, 0.5, 0.5),
        Vec3::new(0., 0., -1.),
        Vec3::splat(1.),
    );

    commands.entity(background).insert(Name::new("Background"));

    commands.entity(player).push_children(&[background]);
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
