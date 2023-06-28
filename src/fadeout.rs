use bevy::prelude::*;

use crate::{ascii, GameState};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(run);
    }
}

#[derive(Component)]
pub struct ScreenFade {
    pub alpha: f32,
    pub sent: bool,
    next_state: GameState,
    timer: Timer,
}

pub(crate) fn create(commands: &mut Commands, next_state: GameState, ascii: &Res<ascii::Sheet>) {
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.color = Color::rgba(0.1, 0.1, 0.15, 0.0);
    sprite.custom_size = Some(Vec2::splat(100000.));

    commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(0., 0., 999.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScreenFade {
            alpha: 0.,
            sent: false,
            next_state,
            timer: Timer::from_seconds(1., TimerMode::Once),
        })
        .insert(Name::new("Fadeout"));
}

fn run(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScreenFade, &mut TextureAtlasSprite)>,
    mut state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    for (entity, mut fade, mut sprite) in query.iter_mut() {
        fade.timer.tick(time.delta());
        if fade.timer.percent() < 0.5 {
            fade.alpha = fade.timer.percent() * 2.;
        } else {
            fade.alpha = fade.timer.percent_left() * 2.;
        }
        sprite.color.set_a(fade.alpha);

        if fade.timer.percent() > 0.5 && !fade.sent {
            state.set(fade.next_state);
            fade.sent = true;
        }

        if fade.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
