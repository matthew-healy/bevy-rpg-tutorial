use bevy::prelude::*;

use crate::player::EncounterTracker;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.register_type::<EncounterTracker>();
        }
    }
}
