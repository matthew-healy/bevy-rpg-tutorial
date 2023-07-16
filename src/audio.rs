use bevy::prelude::*;

use crate::{combat, GameState};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, OverworldMusic::load)
            .add_systems(OnEnter(GameState::Overworld), OverworldMusic::play)
            .add_systems(OnExit(GameState::Overworld), OverworldMusic::pause)
            .add_systems(OnEnter(GameState::Combat), CombatMusic::load)
            .add_systems(Update, HitSfx::load.run_if(on_event::<combat::Event>()))
            .add_systems(OnEnter(combat::State::Reward), RewardSfx::load)
            .add_systems(OnExit(GameState::Combat), CombatMusic::despawn);
    }
}

trait Track: Component + Sized {
    fn load(
        commands: Commands,
        asset_server: Res<AssetServer>,
        query: Query<&AudioSink, With<Self>>,
    );

    fn play(query: Query<&AudioSink, With<Self>>);

    fn pause(query: Query<&AudioSink, With<Self>>);

    fn despawn(commands: Commands, query: Query<(Entity, &AudioSink), With<Self>>);
}

macro_rules! audio_component {
    ( $ty: ident, $asset_name: tt, $playback_setting: expr) => {
        #[derive(Component)]
        struct $ty;

        impl Track for $ty {
            fn load(
                mut commands: Commands,
                asset_server: Res<AssetServer>,
                query: Query<&AudioSink, With<$ty>>,
            ) {
                use bevy::ecs::query::QuerySingleError;
                match query.get_single() {
                    Err(QuerySingleError::NoEntities(_)) => {
                        use bevy::audio::Volume;
                        commands.spawn((
                            AudioBundle {
                                source: asset_server.load($asset_name),
                                settings: $playback_setting.with_volume(Volume::new_relative(0.6)),
                            },
                            $ty,
                        ));
                    }
                    Ok(_) => {
                        println!("Got single...")
                    }
                    Err(QuerySingleError::MultipleEntities(_)) => {
                        unreachable!("we should only have one of each track loaded at a time")
                    }
                }
            }

            fn play(query: Query<&AudioSink, With<$ty>>) {
                if let Ok(sink) = query.get_single() {
                    sink.play();
                }
            }

            fn pause(query: Query<&AudioSink, With<$ty>>) {
                if let Ok(sink) = query.get_single() {
                    sink.pause();
                }
            }

            fn despawn(mut commands: Commands, query: Query<(Entity, &AudioSink), With<$ty>>) {
                if let Ok((entity, sink)) = query.get_single() {
                    sink.stop();
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    };
}

audio_component!(OverworldMusic, "bip-bop.ogg", PlaybackSettings::LOOP);

audio_component!(CombatMusic, "ganxta.ogg", PlaybackSettings::LOOP);

audio_component!(HitSfx, "hit.wav", PlaybackSettings::REMOVE);

audio_component!(RewardSfx, "reward.wav", PlaybackSettings::REMOVE);
