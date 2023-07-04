use bevy::prelude::*;

use strum::EnumCount;

use crate::{ascii, fadeout, player::Player, GameState, RESOLUTION, TILE_SIZE};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stats>()
            .add_state::<State>()
            .add_event::<Event>()
            .insert_resource(MenuSelection {
                selected: MenuOption::Fight,
            })
            .insert_resource(AttackAnimation {
                timer: Timer::from_seconds(0.7, TimerMode::Repeating),
                flash_speed: 0.1,
                shake: Shake {
                    max_distance: 0.1,
                    current_distance: 0.,
                },
            })
            // camera
            .add_system(camera.in_set(OnUpdate(GameState::Combat)))
            // ui
            .add_system(spawn_menu.in_schedule(OnEnter(GameState::Combat)))
            .add_system(spawn_player_health.in_schedule(OnEnter(GameState::Combat)))
            .add_system(highlight_selected_button.in_set(OnUpdate(GameState::Combat)))
            .add_system(despawn_menu.in_schedule(OnExit(GameState::Combat)))
            .add_system(despawn_text.in_schedule(OnExit(GameState::Combat)))
            // player
            .add_system(player_goes_first.in_schedule(OnEnter(GameState::Combat)))
            .add_system(spawn_player_health.in_schedule(OnEnter(GameState::Combat)))
            .add_system(input.in_set(OnUpdate(GameState::Combat)))
            // enemy
            .add_system(spawn_enemy.in_schedule(OnEnter(GameState::Combat)))
            .add_system(enemy_turn.in_set(OnUpdate(State::EnemyTurn)))
            .add_system(despawn_enemy.in_schedule(OnExit(GameState::Combat)))
            // damage calculation
            .add_system(
                // without the `after`s here we were somehow staying in
                // `State::EnemyTurn` for an extra frame, causing the enemy to
                // attack twice.
                damage_calculation
                    .after(enemy_turn)
                    .after(input)
                    .in_set(OnUpdate(GameState::Combat)),
            )
            // attack effects
            .add_system(attack_effects.in_set(OnUpdate(State::PlayerAttack)))
            .add_system(attack_effects.in_set(OnUpdate(State::EnemyAttack)));
    }
}

#[derive(Component)]
struct Enemy;

pub struct Event {
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
    PlayerAttack,
    EnemyTurn,
    EnemyAttack,
    Exiting,
}

#[derive(Resource)]
pub struct AttackAnimation {
    timer: Timer,
    flash_speed: f32,
    shake: Shake,
}

pub struct Shake {
    max_distance: f32,
    current_distance: f32,
}

impl Shake {
    fn tick(&mut self, progress: f32) {
        use std::f32::consts::PI;

        let progress_radians = progress * (2. * PI);
        let shake_progress = progress_radians.sin();

        self.current_distance = self.max_distance * shake_progress;
    }
}

fn player_goes_first(mut combat_state: ResMut<NextState<State>>) {
    combat_state.set(State::PlayerTurn);
}

fn camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    attack_animation: Res<AttackAnimation>,
) {
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation.x = attack_animation.shake.current_distance;
    camera_transform.translation.y = 0.;
}

fn input(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<Event>,
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
                event_writer.send(Event {
                    target,
                    damage_amount: player_stats.attack,
                    next_state: State::PlayerAttack,
                })
            }
            MenuOption::Run => fadeout::create(&mut commands, GameState::Overworld, &ascii),
        }
    }
}

fn damage_calculation(
    mut commands: Commands,
    ascii: Res<ascii::Sheet>,
    mut event_reader: EventReader<Event>,
    text_query: Query<&Transform, With<Text>>,
    mut target_query: Query<(&Children, &mut Stats)>,
    mut state: ResMut<NextState<State>>,
) {
    for event in event_reader.iter() {
        let (target_children, mut target_stats) = target_query
            .get_mut(event.target)
            .expect("Fighting target without stats!");

        target_stats.health = std::cmp::max(
            target_stats.health - (event.damage_amount - target_stats.defense),
            0,
        );

        for child in target_children {
            if let Ok(transform) = text_query.get(*child) {
                commands.entity(*child).despawn_recursive();

                let new_health = ascii::spawn_text(
                    &mut commands,
                    &ascii,
                    &format!("Health: {}", target_stats.health),
                    transform.translation,
                );
                commands
                    .entity(new_health)
                    .insert(Text)
                    // TODO: find a better solution to this
                    .insert(Visibility::Visible);
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

fn enemy_turn(
    mut event_writer: EventWriter<Event>,
    enemy_query: Query<&Stats, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player = player_query.single();
    let enemy_stats = enemy_query.single();

    event_writer.send(Event {
        target: player,
        damage_amount: enemy_stats.attack,
        next_state: State::EnemyAttack,
    })
}

fn attack_effects(
    mut attack_animation: ResMut<AttackAnimation>,
    time: Res<Time>,
    mut enemy_graphics_query: Query<&mut Visibility, With<Enemy>>,
    state: Res<bevy::prelude::State<State>>,
    mut next_state: ResMut<NextState<State>>,
) {
    attack_animation.timer.tick(time.delta());

    let mut enemy_visibility = enemy_graphics_query.iter_mut().next().unwrap();

    match state.0 {
        State::PlayerAttack => {
            if attack_animation.timer.elapsed_secs() % attack_animation.flash_speed
                > attack_animation.flash_speed / 2.
            {
                *enemy_visibility = Visibility::Hidden;
            } else {
                *enemy_visibility = Visibility::Inherited;
            }
        }
        State::EnemyAttack => {
            let progress = attack_animation.timer.percent();
            attack_animation.shake.tick(progress);
        }
        s => unreachable!("{}", format!("unhandled attacking state: {s:?}")),
    }

    if attack_animation.timer.just_finished() {
        // it's possible the previous frame of the animation left the enemy
        // invisible.
        *enemy_visibility = Visibility::Inherited;

        match state.0 {
            State::PlayerAttack => next_state.set(State::EnemyTurn),
            State::EnemyAttack => next_state.set(State::PlayerTurn),
            s => unreachable!("{}", format!("unhandled attack state: {s:?}")),
        }
    }
}

#[derive(Component)]
struct Text;

fn spawn_enemy(mut commands: Commands, ascii: Res<ascii::Sheet>) {
    let enemy_health = 3;
    let health_text = ascii::spawn_text(
        &mut commands,
        &ascii,
        &format!("Health: {}", enemy_health),
        Vec3::new(-4.5 * TILE_SIZE, 2. * TILE_SIZE, 100.),
    );
    commands.entity(health_text).insert(Text);

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

// NOTE: using Player here is a bad idea, because the Player sprite is invisible
// which means we need to explicitly make it visible...
fn spawn_player_health(
    mut commands: Commands,
    ascii: Res<ascii::Sheet>,
    player_query: Query<(Entity, &Stats, &Transform), With<Player>>,
) {
    let (player, stats, transform) = player_query.single();
    let health_text = format!("Health: {}", stats.health);

    let text = ascii::spawn_text(
        &mut commands,
        &ascii,
        &health_text,
        Vec3::new(-RESOLUTION + TILE_SIZE, -1. + TILE_SIZE, 0.) - transform.translation,
    );

    commands
        .entity(text)
        .insert(Text)
        // since the Player's overworld avatar is hidden, we need to explicitly
        // set the text to  visibile to prevent it from inheriting the parent's
        // visibility.
        .insert(Visibility::Visible);
    commands.entity(player).add_child(text);
}

fn despawn_text(mut commands: Commands, query: Query<Entity, With<Text>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// MENU & SELECTION

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
