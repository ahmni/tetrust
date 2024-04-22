mod collision;
mod piece_actions;
mod piece_builder;
mod sounds;
mod stats;
mod user_actions;
mod wall;

use bevy::prelude::*;
use collision::*;
use piece_actions::*;
use piece_builder::*;
use sounds::*;
use stats::*;
use user_actions::*;
use wall::*;

const SCORE_Y: f32 = -120.0;

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
                "1",
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

fn setup(
    mut commands: Commands,
    mut next_pieces: ResMut<NextPieces>,
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
        let row: Vec<Entity> = Vec::new();
        placed_pieces.0.push(row);
    }

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
        .insert_resource(DropTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .insert_resource(MovementTimer(Timer::from_seconds(
            0.075,
            TimerMode::Repeating,
        )))
        .insert_resource(PlacedPieces(Vec::new()))
        .insert_resource(NextPieces(Vec::new()))
        .insert_resource(GracePeriodTimer(Timer::from_seconds(0.5, TimerMode::Once)))
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
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, text_setup, sound_setup))
        .add_systems(
            Update,
            (
                (
                    shift_active_down,
                    user_move_actives,
                    user_rotate_active,
                    check_collision,
                    check_in_bounds,
                    try_to_place_piece,
                    score,
                    place_piece,
                    position_next_pieces,
                    hold_piece,
                    level,
                )
                    .chain(),
                pause_music,
                sound_effects,
            ),
        )
        .run();
}
