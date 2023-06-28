use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::player::EncounterTracker;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.register_type::<EncounterTracker>()
                .add_plugin(WorldInspectorPlugin::new());
        }
    }
}
