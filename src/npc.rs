use bevy::prelude::*;

use crate::{ascii, combat, player::Player, GameState, TILE_SIZE};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // TODO: i'd prefer to have a (local?) speech state, and to react to
        //       entering & leaving that state.
        app.add_systems(
            Update,
            speech
                .before(textbox::despawn)
                .run_if(in_state(GameState::Overworld)),
        )
        .add_systems(
            Update,
            textbox::despawn.run_if(in_state(GameState::Overworld)),
        );
    }
}

#[derive(Component)]
pub enum Role {
    Healer,
}

fn speech(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &mut combat::Stats, &Transform)>,
    camera_query: Query<&Transform, With<Camera>>,
    npc_query: Query<(&Role, &Transform)>,
    mut keyboard: ResMut<Input<KeyCode>>,
    ascii: Res<ascii::Sheet>,
    indices: Res<ascii::NinesliceIndices>,
) {
    let (mut player, mut stats, player_transform) = player_query.single_mut();
    let camera_transform = camera_query.single();

    if !player.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        for (_, npc_transform) in npc_query.iter() {
            if npc_transform
                .translation
                .truncate()
                .distance(player_transform.translation.truncate())
                < TILE_SIZE * 1.5
            {
                player.active = false;
                stats.health = stats.max_health;

                textbox::spawn(
                    &mut commands,
                    &ascii,
                    &indices,
                    Vec2::new(0., 1. - 1.5 * TILE_SIZE) + camera_transform.translation.truncate(),
                    "It's me, Dylan!",
                );

                keyboard.clear();
            }
        }
    }
}

mod textbox {
    use crate::{ascii, player::Player, CLEAR, TILE_SIZE};
    use bevy::prelude::*;

    #[derive(Component)]
    pub(crate) struct Text;

    pub(crate) fn spawn(
        commands: &mut Commands,
        ascii: &ascii::Sheet,
        indices: &ascii::NinesliceIndices,
        translation: Vec2,
        text: &str,
    ) -> Entity {
        let width = text.len() as f32 + 2.;
        let nineslice = ascii::spawn_nineslice(commands, ascii, indices, width, 3.);
        let background = ascii::spawn_sprite(
            commands,
            ascii,
            0,
            CLEAR,
            Vec3::new(0., 0., -1.),
            Vec3::new(width, 3., 1.),
        );

        let x_offset = (-width / 2. + 1.5) * TILE_SIZE;
        let text = ascii::spawn_text(commands, ascii, text, Vec3::new(x_offset, 0., 0.));

        commands
            .spawn_empty()
            .insert(SpatialBundle::default())
            .insert(Transform {
                translation: translation.extend(900.),
                ..Default::default()
            })
            .insert(Name::new("Npc Text"))
            .insert(Text)
            .add_child(text)
            .add_child(background)
            .add_child(nineslice)
            .id()
    }

    pub(crate) fn despawn(
        mut commands: Commands,
        mut player_query: Query<&mut Player>,
        speech_query: Query<Entity, With<Text>>,
        mut keyboard: ResMut<Input<KeyCode>>,
    ) {
        if keyboard.just_pressed(KeyCode::Space) {
            let mut player = player_query.single_mut();
            player.active = true;
            for entity in speech_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            keyboard.clear();
        }
    }
}
