use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{ascii, tilemap, TILE_SIZE};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn)
            .add_system(movement)
            .add_system(camera_follow.after(movement));
    }
}

#[derive(Component)]
pub struct Player {
    speed: f32,
}

fn movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    wall_query: Query<&Transform, (With<tilemap::Collider>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform) = player_query.single_mut();

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
    if !would_collide(target, &wall_query) {
        transform.translation = target;
    }

    let target = transform.translation + Vec3::new(0., y_delta, 0.);
    if !would_collide(target, &wall_query) {
        transform.translation = target;
    }
}

fn would_collide(
    target_player_pos: Vec3,
    wall_query: &Query<&Transform, (With<tilemap::Collider>, Without<Player>)>,
) -> bool {
    for wall_transform in wall_query.iter() {
        let collision = collide(
            target_player_pos,
            Vec2::splat(TILE_SIZE * 0.9),
            wall_transform.translation,
            Vec2::splat(TILE_SIZE),
        );

        if collision.is_some() {
            return true;
        }
    }
    false
}

fn spawn(mut commands: Commands, ascii: Res<ascii::Sheet>) {
    let player = ascii::spawn_sprite(
        &mut commands,
        ascii.as_ref(),
        1,
        Color::rgb(0.3, 0.3, 0.9),
        Vec3::new(2. * TILE_SIZE, -2. * TILE_SIZE, 900.),
    );

    commands
        .entity(player)
        .insert(Name::new("Player"))
        // TODO: in the tutorial the speed is `3.`, but here I had to use a
        // much larger number in order to get roughly the same movement speed.
        // This seems likely to be related to the weirdness around `TILE_SIZE`,
        // but I still don't know what exactly changed in bevy to require such
        // different numbers.
        .insert(Player { speed: 7. });

    let background = ascii::spawn_sprite(
        &mut commands,
        ascii.as_ref(),
        0,
        Color::rgb(0.5, 0.5, 0.5),
        Vec3::new(0., 0., -1.),
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
