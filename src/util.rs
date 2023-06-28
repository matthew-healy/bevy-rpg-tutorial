use bevy::prelude::*;

pub fn show<T: Component>(mut query: Query<&mut Visibility, With<T>>) {
    if let Ok(mut visibility) = query.get_single_mut() {
        *visibility = Visibility::Inherited;
    }
}

pub fn hide<T: Component>(mut query: Query<&mut Visibility, With<T>>) {
    if let Ok(mut visibility) = query.get_single_mut() {
        *visibility = Visibility::Hidden;
    }
}
