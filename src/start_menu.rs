use bevy::prelude::*;

use crate::{ascii, fadeout, GameState};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load)
            .add_systems(Update, start_button.run_if(in_state(GameState::StartMenu)))
            .add_systems(OnExit(GameState::StartMenu), unload);
    }
}

#[derive(Component)]
struct ButtonActive;

#[derive(Resource)]
struct UiAssets {
    font: Handle<Font>,
    button: Handle<Image>,
    button_pressed: Handle<Image>,
}

fn load(mut commands: Commands, assets: Res<AssetServer>) {
    let ui_assets = UiAssets {
        font: assets.load("QuattrocentoSans-Bold.ttf"),
        button: assets.load("button.png"),
        button_pressed: assets.load("button_pressed.png"),
    };

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(20.),
                    height: Val::Percent(10.),
                    margin: UiRect::all(Val::Auto),
                    ..Default::default()
                },
                image: ui_assets.button.clone().into(),
                ..Default::default()
            },
            ButtonActive,
        ))
        .with_children(|b| {
            b.spawn(TextBundle {
                text: Text::from_section(
                    "Start Game",
                    TextStyle {
                        font: ui_assets.font.clone(),
                        font_size: 40.,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ..Default::default()
            });
        });

    commands.insert_resource(ui_assets);
}

fn start_button(
    mut commands: Commands,
    entity_query: Query<(Entity, &ButtonActive, &Interaction), Changed<Interaction>>,
    mut image_query: Query<&mut UiImage>,
    ui_assets: Res<UiAssets>,
    ascii: Res<ascii::Sheet>,
) {
    if let Ok((entity, _, interaction)) = entity_query.get_single() {
        let mut image = image_query.get_mut(entity).unwrap();

        match interaction {
            Interaction::Pressed => {
                *image = ui_assets.button_pressed.clone().into();
                commands.entity(entity).remove::<ButtonActive>();
                fadeout::create(&mut commands, GameState::Overworld, &ascii);
            }
            Interaction::Hovered | Interaction::None => {
                *image = ui_assets.button.clone().into();
            }
        }
    }
}

fn unload(mut commands: Commands, button_query: Query<Entity, With<Button>>) {
    for entity in button_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
