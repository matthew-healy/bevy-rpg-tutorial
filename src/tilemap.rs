use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use bevy::prelude::*;

use crate::{ascii, TILE_SIZE};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_simple);
    }
}

#[derive(Component)]
pub struct Collider;

fn create_simple(mut commands: Commands, ascii: Res<ascii::Sheet>) {
    let file = File::open("assets/map.txt").expect("No map file found");
    let mut tiles = Vec::new();

    for (y, line) in BufReader::new(file).lines().enumerate() {
        if let Ok(line) = line {
            for (x, char) in line.chars().enumerate() {
                let tile = ascii::spawn_sprite(
                    &mut commands,
                    &ascii,
                    char as usize,
                    Color::rgb(0.9, 0.9, 0.9),
                    Vec3::new(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 100.),
                );
                if char == '#' {
                    commands.entity(tile).insert(Collider);
                }
                tiles.push(tile);
            }
        }
    }

    commands
        .spawn_empty()
        .insert(Name::new("Map"))
        .insert(SpatialBundle::default())
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .push_children(&tiles);
}
