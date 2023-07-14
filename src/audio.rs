use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{combat, GameState};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<BgTrack>()
            .add_audio_channel::<CombatTrack>()
            .add_audio_channel::<SfxTrack>()
            .add_systems(PreStartup, load)
            .add_systems(Startup, start_bgm)
            .add_systems(OnEnter(GameState::Overworld), resume::<BgTrack>)
            .add_systems(OnExit(GameState::Overworld), pause::<BgTrack>)
            .add_systems(OnEnter(GameState::Combat), start_combat_track)
            .add_systems(Update, play_hit.run_if(in_state(GameState::Combat)))
            .add_systems(OnEnter(combat::State::Reward), play_reward)
            .add_systems(OnExit(GameState::Combat), stop::<CombatTrack>);
    }
}

#[derive(Resource)]
struct State {
    handles: Handles,
}

struct Handles {
    bgm: Handle<AudioSource>,
    combat: Handle<AudioSource>,
    hit: Handle<AudioSource>,
    reward: Handle<AudioSource>,
}

#[derive(Resource)]
struct SfxTrack;

#[derive(Resource)]
struct BgTrack;

#[derive(Resource)]
struct CombatTrack;

fn load(
    mut commands: Commands,
    bg_channel: Res<AudioChannel<BgTrack>>,
    combat_channel: Res<AudioChannel<CombatTrack>>,
    sfx_channel: Res<AudioChannel<SfxTrack>>,
    assets: Res<AssetServer>,
) {
    let handles = {
        let bgm = assets.load("bip-bop.ogg");
        let combat = assets.load("ganxta.ogg");
        let hit = assets.load("hit.wav");
        let reward = assets.load("reward.wav");

        Handles {
            bgm,
            combat,
            hit,
            reward,
        }
    };

    commands.insert_resource(State { handles });

    let volume = 0.5;
    bg_channel.set_volume(volume);
    combat_channel.set_volume(volume);
    sfx_channel.set_volume(volume);
}

fn start_bgm(audio: Res<AudioChannel<BgTrack>>, state: Res<State>) {
    audio.play(state.handles.bgm.clone()).looped();
}

fn resume<T: Resource>(audio: Res<AudioChannel<T>>) {
    audio.resume();
}

fn pause<T: Resource>(audio: Res<AudioChannel<T>>) {
    audio.pause();
}

fn stop<T: Resource>(audio: Res<AudioChannel<T>>) {
    audio.stop();
}

fn start_combat_track(audio: Res<AudioChannel<CombatTrack>>, state: Res<State>) {
    audio.play(state.handles.combat.clone()).looped();
}

fn play_hit(
    audio: Res<AudioChannel<SfxTrack>>,
    state: Res<State>,
    mut event_reader: EventReader<combat::Event>,
) {
    let cnt = event_reader.iter().count();
    if cnt > 0 {
        audio.play(state.handles.hit.clone());
    }
}

fn play_reward(audio: Res<AudioChannel<SfxTrack>>, state: Res<State>) {
    audio.play(state.handles.reward.clone());
}
