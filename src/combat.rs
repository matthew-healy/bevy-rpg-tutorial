use bevy::prelude::*;
use strum::EnumCount;

use crate::{ascii, fadeout, player::Player, GameState, RESOLUTION, TILE_SIZE};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stats>()
            .add_state::<State>()
            .add_event::<FightEvent>()
            .insert_resource(MenuSelection {
                selected: MenuOption::Fight,
            })
            .add_system(combat_camera.in_set(OnUpdate(GameState::Combat)))
            .add_system(input.in_set(OnUpdate(GameState::Combat)))
            .add_system(damage_calculation.in_set(OnUpdate(GameState::Combat)))
            .add_system(highlight_selected_button.in_set(OnUpdate(GameState::Combat)))
            .add_system(enemy_turn.in_set(OnUpdate(State::EnemyTurn)))
            .add_system(spawn_enemy.in_schedule(OnEnter(GameState::Combat)))
            .add_system(despawn_enemy.in_schedule(OnExit(GameState::Combat)))
            .add_system(spawn_menu.in_schedule(OnEnter(GameState::Combat)))
            .add_system(despawn_menu.in_schedule(OnExit(GameState::Combat)));
    }
}

#[derive(Component)]
struct Enemy;

struct FightEvent {
    target: Entity,
    damage_amount: isize,
    next_state: State,
}

#[derive(Component, Reflect)]
pub struct Stats {
    pub health: isize,
    pub max_health: isize,
    pub attack: isize,
    pub defense: isize,
}

#[derive(PartialEq, Eq, Component, Clone, Copy, strum_macros::EnumCount)]
enum MenuOption {
    // NOTE: the order of items here is important as we do conversions to & from
    // `isize` in `input`. Be wary of this if changing.
    Fight,
    Run,
}

#[derive(Resource)]
pub struct MenuSelection {
    selected: MenuOption,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default, States)]
pub enum State {
    #[default]
    PlayerTurn,
    EnemyTurn,
    Exiting,
}

fn input(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut fight_event_writer: EventWriter<FightEvent>,
    player_query: Query<&Stats, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut menu_state: ResMut<MenuSelection>,
    ascii: Res<ascii::Sheet>,
    combat_state: Res<bevy::prelude::State<State>>,
) {
    if combat_state.0 != State::PlayerTurn {
        return;
    }

    let mut new_selection = menu_state.selected as isize;

    if keyboard.just_pressed(KeyCode::A) {
        new_selection -= 1;
    }

    if keyboard.just_pressed(KeyCode::D) {
        new_selection += 1;
    }

    let menu_size = MenuOption::COUNT as isize;

    new_selection = (new_selection + menu_size) % menu_size;

    menu_state.selected = match new_selection {
        0 => MenuOption::Fight,
        1 => MenuOption::Run,
        _ => unreachable!("Bad menu selection"),
    };

    if keyboard.just_pressed(KeyCode::Return) {
        match menu_state.selected {
            MenuOption::Fight => {
                let player_stats = player_query.single();
                let target = enemy_query.iter().next().unwrap();
                fight_event_writer.send(FightEvent {
                    target,
                    damage_amount: player_stats.attack,
                    next_state: State::EnemyTurn,
                })
            }
            MenuOption::Run => fadeout::create(&mut commands, GameState::Overworld, &ascii),
        }
    }
}

fn damage_calculation(
    mut commands: Commands,
    ascii: Res<ascii::Sheet>,
    mut fight_event_reader: EventReader<FightEvent>,
    text_query: Query<&ascii::Text>,
    mut target_query: Query<(&Children, &mut Stats)>,
    mut state: ResMut<NextState<State>>,
) {
    for event in fight_event_reader.iter() {
        let (target_children, mut target_stats) = target_query
            .get_mut(event.target)
            .expect("Fighting target without stats!");

        target_stats.health = std::cmp::max(
            target_stats.health - (event.damage_amount - target_stats.defense),
            0,
        );

        for child in target_children {
            if text_query.get(*child).is_ok() {
                commands.entity(*child).despawn_recursive();

                let new_health = ascii::spawn_text(
                    &mut commands,
                    &ascii,
                    &format!("Health: {}", target_stats.health),
                    Vec3::new(-4.5 * TILE_SIZE, 2. * TILE_SIZE, 100.),
                );
                commands.entity(event.target).add_child(new_health);
            }
        }

        if target_stats.health == 0 {
            fadeout::create(&mut commands, GameState::Overworld, &ascii);
            state.set(State::Exiting);
        } else {
            state.set(event.next_state);
        }
    }
}

fn spawn_enemy(mut commands: Commands, ascii: Res<ascii::Sheet>) {
    let enemy_health = 3;
    let health_text = ascii::spawn_text(
        &mut commands,
        &ascii,
        &format!("Health: {}", enemy_health),
        Vec3::new(-4.5 * TILE_SIZE, 2. * TILE_SIZE, 100.),
    );

    let sprite = ascii::spawn_sprite(
        &mut commands,
        &ascii,
        'b' as usize,
        Color::rgb(0.8, 0.8, 0.8),
        Vec3::new(0., 0.5, 100.),
        Vec3::splat(1.),
    );

    commands
        .entity(sprite)
        .insert(Enemy)
        .insert(Stats {
            health: 3,
            max_health: 3,
            attack: 2,
            defense: 1,
        })
        .insert(Name::new("Bat"))
        .add_child(health_text);
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

fn spawn_menu(
    mut commands: Commands,
    ascii: Res<ascii::Sheet>,
    nineslice_indices: Res<ascii::NinesliceIndices>,
) {
    let box_height = 3.;
    let box_center_y = -1.0 + box_height * TILE_SIZE / 2.;

    let run_text = "Run";
    let run_width = (run_text.len() + 2) as f32;
    let run_center_x = RESOLUTION - (run_width * TILE_SIZE) / 2.;

    spawn_button(
        &mut commands,
        &ascii,
        &nineslice_indices,
        Vec3::new(run_center_x, box_center_y, 100.),
        run_text,
        MenuOption::Run,
        Vec2::new(run_width, box_height),
    );

    let fight_text = "Fight";
    let fight_width = (fight_text.len() + 2) as f32;
    let fight_center_x = RESOLUTION - (run_width * TILE_SIZE) - (fight_width * TILE_SIZE / 2.);

    spawn_button(
        &mut commands,
        &ascii,
        &nineslice_indices,
        Vec3::new(fight_center_x, box_center_y, 100.),
        fight_text,
        MenuOption::Fight,
        Vec2::new(fight_width, box_height),
    );
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuOption>>) {
    for button in query.iter() {
        commands.entity(button).despawn_recursive();
    }
}

fn spawn_button(
    commands: &mut Commands,
    ascii: &ascii::Sheet,
    indices: &ascii::NinesliceIndices,
    translation: Vec3,
    text: &str,
    id: MenuOption,
    size: Vec2,
) -> Entity {
    let nineslice = ascii::spawn_nineslice(commands, ascii, indices, size.x, size.y);

    let x_offset = (-size.x / 2. + 1.5) * TILE_SIZE;
    let text = ascii::spawn_text(commands, ascii, text, Vec3::new(x_offset, 0., 0.));

    commands
        .spawn_empty()
        .insert(SpatialBundle::default())
        .insert(Transform {
            translation,
            ..Default::default()
        })
        .insert(Name::new("Button"))
        .insert(id)
        .add_child(text)
        .add_child(nineslice)
        .id()
}

fn highlight_selected_button(
    menu_selection: Res<MenuSelection>,
    button_query: Query<(&Children, &MenuOption)>,
    nineslice_query: Query<&Children, With<ascii::Nineslice>>,
    mut sprites_query: Query<&mut TextureAtlasSprite>,
) {
    for (button_children, button_id) in button_query.iter() {
        for button_child in button_children.iter() {
            if let Ok(nineslice_children) = nineslice_query.get(*button_child) {
                for nineslice_child in nineslice_children.iter() {
                    if let Ok(mut sprite) = sprites_query.get_mut(*nineslice_child) {
                        if menu_selection.selected == *button_id {
                            sprite.color = Color::RED;
                        } else {
                            sprite.color = Color::WHITE;
                        }
                    }
                }
            }
        }
    }
}

fn enemy_turn(
    mut fight_event_writer: EventWriter<FightEvent>,
    enemy_query: Query<&Stats, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player = player_query.single();
    let enemy_stats = enemy_query.single();

    fight_event_writer.send(FightEvent {
        target: player,
        damage_amount: enemy_stats.attack,
        next_state: State::PlayerTurn,
    })
}
