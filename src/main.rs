mod animation;
mod collision;
mod piece_actions;
mod piece_builder;
mod sounds;
mod stats;
mod ui;
mod user_actions;
mod wall;

use std::time::Duration;

use animation::*;
use bevy::prelude::*;
use collision::*;
use piece_actions::*;
use piece_builder::*;
use sounds::*;
use stats::*;
use ui::*;
use user_actions::*;
use wall::*;

const SCORE_Y: f32 = -120.0;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Playing,
    Paused,
    GameOver,
}

fn text_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bold_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let reg_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let text_style = TextStyle {
        font: bold_font.clone(),
        font_size: 42.0,
        color: Color::WHITE,
    };
    let text_justification = JustifyText::Center;

    commands.spawn(Text2dBundle {
        text: Text::from_section("NEXT", text_style.clone()).with_justify(text_justification),
        transform: Transform::from_xyz(RIGHT_GRID * 1.6, NEXT_PIECE_Y + 104.0, 0.0),
        ..default()
    });

    commands.spawn(Text2dBundle {
        text: Text::from_section("HOLD", text_style.clone()).with_justify(text_justification),
        transform: Transform::from_xyz(LEFT_GRID * 1.7, HOLD_PIECE_Y + 84.0, 0.0),
        ..default()
    });

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "TETRUST",
            TextStyle {
                font: bold_font.clone(),
                font_size: 60.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_justify(text_justification),
        transform: Transform::from_xyz(0.0, 340.0, 0.0),
        ..default()
    });

    commands.spawn(Text2dBundle {
        text: Text::from_section("SCORE", text_style.clone()).with_justify(text_justification),
        transform: Transform::from_xyz(LEFT_GRID * 1.7, SCORE_Y, 0.0),
        ..default()
    });
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font: reg_font.clone(),
                    font_size: 42.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_justify(text_justification),
            transform: Transform::from_xyz(LEFT_GRID * 1.7, SCORE_Y - 40.0, 0.0),
            ..default()
        },
        Score(0),
    ));

    commands.spawn(Text2dBundle {
        text: Text::from_section("LEVEL", text_style.clone()).with_justify(text_justification),
        transform: Transform::from_xyz(LEFT_GRID * 1.7, SCORE_Y - 80.0, 0.0),
        ..default()
    });

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font: reg_font.clone(),
                    font_size: 42.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_justify(text_justification),
            transform: Transform::from_xyz(LEFT_GRID * 1.7, SCORE_Y - 120.0, 0.0),
            ..default()
        },
        Level(1),
    ));
}

fn pause_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    align_self: AlignSelf::Stretch,
                    justify_self: JustifySelf::Stretch,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    align_content: AlignContent::FlexStart,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            PauseMenu,
        ))
        .id();

    let button = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((TextBundle {
                text: Text::from_section(
                    "Paused",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 60.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },));

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PauseButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    RestartButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Restart",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        })
        .id();

    commands.entity(root).add_child(button);
}

#[derive(Component)]
pub struct TitleMenu;

fn title_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    align_self: AlignSelf::Stretch,
                    justify_self: JustifySelf::Stretch,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    align_content: AlignContent::FlexStart,
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                visibility: Visibility::Visible,
                ..Default::default()
            },
            TitleMenu,
        ))
        .id();

    let button = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((TextBundle {
                text: Text::from_section(
                    "TETRUST",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 60.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },));
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    TitleButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        })
        .id();

    commands.entity(root).add_child(button);
}

#[derive(Component)]
pub struct GameOverMenu;

fn game_over_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    align_self: AlignSelf::Stretch,
                    justify_self: JustifySelf::Stretch,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    align_content: AlignContent::FlexStart,
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            GameOverMenu,
        ))
        .id();

    let button = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((TextBundle {
                text: Text::from_section(
                    "Game Over",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 60.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },));
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    RestartButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Restart",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        })
        .id();

    commands.entity(root).add_child(button);
}

#[derive(Component)]
pub struct DespawnOnRestart;

fn game_over(
    mut next_state: ResMut<NextState<GameState>>,
    mut ev_game_over: EventReader<GameOverEvent>,
    mut game_over_menu: Query<&mut Visibility, With<GameOverMenu>>,
) {
    if ev_game_over.read().next().is_some() {
        next_state.set(GameState::GameOver);
        *game_over_menu.single_mut() = Visibility::Visible;
    }
    ev_game_over.clear();
}

#[derive(Event)]
pub struct RestartGameEvent;

pub fn restart_game(
    game_state: Res<State<GameState>>,
    entities_to_despawn: Query<Entity, With<DespawnOnRestart>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    mut commands: Commands,
    mut ev_restart: EventReader<RestartGameEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut placed_pieces: ResMut<PlacedPieces>,
    mut next_pieces: ResMut<NextPieces>,
    mut game_over_menu: Query<&mut Visibility, (With<GameOverMenu>, Without<PauseMenu>)>,
    mut drop_timer: ResMut<DropTimer>,
) {
    if ev_restart.read().next().is_some() {
        for row in placed_pieces.0.iter_mut() {
            for entity in row.iter_mut() {
                *entity = None;
            }
        }

        next_pieces.0.clear();

        for entity in entities_to_despawn.iter() {
            commands.entity(entity).despawn();
        }

        setup_pieces(commands, next_pieces);
        toggle_menu(
            game_state,
            &mut next_state,
            music_controller,
            &mut game_over_menu.single_mut(),
        );

        drop_timer.0.set_duration(Duration::from_millis(500));
    }

    ev_restart.clear();
}

fn setup(
    next_pieces: ResMut<NextPieces>,
    mut commands: Commands,
    mut placed_pieces: ResMut<PlacedPieces>,
) {
    commands.spawn(Camera2dBundle::default());

    // build walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // build grid
    for i in 0..((TOP_GRID - BOTTOM_GRID) / SQUARE_SIZE + 1.0) as i32 {
        commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(0.0, TOP_GRID - i as f32 * SQUARE_SIZE, 0.0),
            sprite: Sprite {
                color: GRID_LINE_COLOR,
                custom_size: Some(Vec2::new(RIGHT_GRID - LEFT_GRID, GRID_LINE_THICKNESS)),
                ..default()
            },
            ..default()
        });
    }

    for i in 0..((RIGHT_GRID - LEFT_GRID) / SQUARE_SIZE + 1.0) as i32 {
        commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(LEFT_GRID + i as f32 * SQUARE_SIZE, 0.0, 0.0),
            sprite: Sprite {
                color: GRID_LINE_COLOR,
                custom_size: Some(Vec2::new(GRID_LINE_THICKNESS, TOP_GRID - BOTTOM_GRID)),
                ..default()
            },
            ..default()
        });
    }

    // build placed piece grid
    for _ in 0..20 {
        let mut row: Vec<Option<Entity>> = Vec::with_capacity(10);
        row.resize(10, None);
        placed_pieces.0.push(row);
    }

    setup_pieces(commands, next_pieces);
}

pub fn setup_pieces(mut commands: Commands, mut next_pieces: ResMut<NextPieces>) {
    let starting_piece = get_random_piece();

    build_active_piece(&mut commands, starting_piece, Vec3::new(0.0, 240.0, -1.0));

    for _ in 0..3 {
        let new_piece = get_random_piece();

        let entities = build_piece(&mut commands, new_piece, Vec3::new(0.0, 0.0, -1.0));
        // only push the parent which is always the first entiy
        next_pieces.0.push(entities[0]);
    }
}

#[derive(Event)]
pub struct GameOverEvent;

fn main() {
    App::new()
        .init_state::<GameState>()
        .insert_resource(DropTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .insert_resource(MovementTimer(Timer::from_seconds(
            0.075,
            TimerMode::Repeating,
        )))
        .insert_resource(PlacedPieces(Vec::new()))
        .insert_resource(NextPieces(Vec::new()))
        .insert_resource(GracePeriodTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .insert_resource(AttemptingPlaceAnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .insert_resource(ClearingAnimationTimer(Timer::from_seconds(
            0.05,
            TimerMode::Repeating,
        )))
        .insert_resource(FlashingAnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        //.insert_resource(PlaceGracePeriod(Timer::from_seconds(0.25, TimerMode::Once)))
        .add_event::<PiecePlacedEvent>()
        .add_event::<CollisionEvent>()
        .add_event::<ClearEvent>()
        .add_event::<MoveEvent>()
        .add_event::<RotateEvent>()
        .add_event::<LevelUpEvent>()
        .add_event::<HoldPieceEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<AttemptPlaceEvent>()
        .add_event::<PauseGameEvent>()
        .add_event::<RestartGameEvent>()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (
                setup,
                text_setup,
                sound_setup,
                pause_setup,
                game_over_setup,
                title_menu_setup,
            ),
        )
        .add_systems(PreUpdate, position_next_pieces)
        .add_systems(
            Update,
            (
                (
                    shift_active_down,
                    user_move_actives,
                    user_rotate_active,
                    check_collision,
                    game_over,
                    check_in_bounds,
                    try_to_place_piece,
                    placing_piece_animation,
                    score,
                    place_piece_animation,
                    place_piece,
                    clear_rows,
                    level,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
                pause_game.run_if(not(in_state(GameState::GameOver))),
                pause_music,
                pause_button.run_if(in_state(GameState::Paused)),
                restart_button.run_if(not(in_state(GameState::Playing))),
                title_button.run_if(in_state(GameState::Title)),
                user_restart,
                restart_game,
                sound_effects,
                button_system,
            ),
        )
        .add_systems(PostUpdate, hold_piece)
        .run();
}
