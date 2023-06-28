use bevy::prelude::*;

use crate::{ascii, fadeout, GameState};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(test_exit.in_set(OnUpdate(GameState::Combat)))
            .add_system(combat_camera.in_set(OnUpdate(GameState::Combat)))
            .add_system(spawn_enemy.in_schedule(OnEnter(GameState::Combat)))
            .add_system(despawn_enemy.in_schedule(OnExit(GameState::Combat)));
    }
}

#[derive(Component)]
struct Enemy;

fn spawn_enemy(mut commands: Commands, ascii: Res<ascii::Sheet>) {
    let sprite = ascii::spawn_sprite(
        &mut commands,
        &ascii,
        'b' as usize,
        Color::rgb(0.8, 0.8, 0.8),
        Vec3::new(0., 0.5, 100.),
    );

    commands
        .entity(sprite)
        .insert(Enemy)
        .insert(Name::new("Bat"));
}

fn despawn_enemy(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn combat_camera(mut camera_query: Query<&mut Transform, With<Camera>>) {
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation.x = 0.;
    camera_transform.translation.y = 0.;
}

fn test_exit(mut commands: Commands, keyboard: Res<Input<KeyCode>>, ascii: Res<ascii::Sheet>) {
    if keyboard.just_pressed(KeyCode::Space) {
        fadeout::create(&mut commands, GameState::Overworld, &ascii);
    }
}
