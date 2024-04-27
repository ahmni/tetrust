use std::collections::HashSet;

use crate::{
    fix_position, get_col, get_row, internal_pause_music, Active, Collision, CollisionEvent,
    GameMusic, GameState, PauseGameEvent, PiecePlacedEvent, PieceType, PlacedPieces,
    RestartGameEvent, BOTTOM_GRID, LEFT_GRID, RIGHT_GRID, SQUARE_SIZE,
};
use bevy::prelude::*;

#[derive(Resource)]
pub struct MovementTimer(pub Timer);

#[derive(Component, Default, Debug)]
pub enum RotationState {
    #[default]
    Zero, // zero rotates
    R,   // clockwise or "right" rotation
    L,   // counter-clockwise or "left" rotation
    Two, // two rotations starting from zero
}

fn get_rotation_state(rotation_state: &RotationState, is_clockwise: bool) -> RotationState {
    if is_clockwise {
        match rotation_state {
            RotationState::Zero => RotationState::R,
            RotationState::R => RotationState::Two,
            RotationState::Two => RotationState::L,
            RotationState::L => RotationState::Zero,
        }
    } else {
        match rotation_state {
            RotationState::Zero => RotationState::L,
            RotationState::L => RotationState::Two,
            RotationState::Two => RotationState::R,
            RotationState::R => RotationState::Zero,
        }
    }
}

const KICK_VALUES: [[(i32, i32); 5]; 8] = [
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)], // 0->R
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],     // R-> 0
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],     // R-> 2
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)], // 2->R
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],    // 2-> L
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],  // L->2
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],  // L-> 0
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],    // 0->L
];

const I_KICK_VALUES: [[(i32, i32); 5]; 8] = [
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)], // 0->R
    [(0, 0), (2, 0), (-1, 0), (1, 2), (-2, -1)], // R-> 0
    [(0, 0), (-1, 0), (2, 0), (1, 2), (2, -1)],  // R->2
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)], // 2->R
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)], // 2-> L
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)], // L->2
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)], // L-> 0
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)], // 0->L
];

fn get_kick_values(
    current_state: &RotationState,
    next_state: &RotationState,
    piece_type: &PieceType,
    placed_pieces: &PlacedPieces,
    child_translations: Vec<Vec3>,
) -> Option<(i32, i32)> {
    // check if piece is colliding
    let index = match (current_state, next_state) {
        (RotationState::Zero, RotationState::R) => 0,
        (RotationState::R, RotationState::Zero) => 1,
        (RotationState::R, RotationState::Two) => 2,
        (RotationState::Two, RotationState::R) => 3,
        (RotationState::Two, RotationState::L) => 4,
        (RotationState::L, RotationState::Two) => 5,
        (RotationState::L, RotationState::Zero) => 6,
        (RotationState::Zero, RotationState::L) => 7,
        _ => panic!("impossible rotation transition"),
    };
    let kick_values = match piece_type {
        PieceType::Straight => I_KICK_VALUES[index],
        _ => KICK_VALUES[index],
    };

    println!("current state: {:?}", current_state);
    println!("next state: {:?}", next_state);

    for &kick in kick_values.iter() {
        let mut new_child_translations = vec![];
        for translation in child_translations.iter() {
            println!("original translation {:?}", translation);
            new_child_translations.push(Vec3::new(
                translation.x + (kick.0 as f32) * SQUARE_SIZE,
                translation.y + (kick.1 as f32) * SQUARE_SIZE,
                translation.z,
            ));
        }

        let mut can_kick = true;
        for translation in new_child_translations.iter() {
            println!("checking translation {:?}", translation);
            println!("kick {:?}", kick);
            if !is_valid_position(translation, placed_pieces) {
                can_kick = false;
                break;
            }
        }

        if can_kick {
            return Some(kick);
        }
    }

    None
}

fn is_valid_position(translation: &Vec3, placed_pieces: &PlacedPieces) -> bool {
    if translation.x < LEFT_GRID
        || translation.x > RIGHT_GRID - SQUARE_SIZE
        || translation.y < BOTTOM_GRID
    {
        println!("wall collision");
        return false;
    }

    let row = get_row(translation.y);
    let col = get_col(translation.x);
    if placed_pieces.0[row][col].is_some() {
        println!("piece collision");
        return false;
    }

    println!("valid position");
    true
}

fn rotate_clockwise(
    children: &Children,
    rotation_state: &mut RotationState,
    placed_pieces: Res<PlacedPieces>,
    piece_type: &PieceType,
    parent_translation: &Vec3,
    child_query: &mut Query<(&GlobalTransform, &mut Transform), Without<Children>>,
    translation: &mut Vec3,
) -> bool {
    let next_state = get_rotation_state(rotation_state, true);
    let mut translations_to_apply: Vec<Vec3> = vec![];
    let mut global_translations: Vec<Vec3> = vec![];
    for &child in children.iter() {
        let child_transform = child_query.get(child).unwrap();
        let child_translation = child_transform.1.translation;
        let child_global_translation = child_transform.0.translation();
        println!("child translation {:?}", child_global_translation);

        let new_translation = Vec3::new(
            child_translation.y,
            -child_translation.x,
            child_translation.z,
        );
        let new_global_translation = *parent_translation + new_translation;
        //println!("new translation {:?}", new_translation);
        //println!("new global translation {:?}", new_global_translation);
        translations_to_apply.push(new_translation);
        global_translations.push(new_global_translation);
    }

    let kick_values = if let Some(values) = get_kick_values(
        rotation_state,
        &next_state,
        &piece_type,
        &placed_pieces,
        global_translations,
    ) {
        values
    } else {
        return false;
    };

    for (i, &child) in children.iter().enumerate() {
        let mut child_transform = child_query.get_mut(child).unwrap();
        child_transform.1.translation = translations_to_apply[i];
    }

    translation.x += (kick_values.0 as f32) * SQUARE_SIZE;
    translation.y += (kick_values.1 as f32) * SQUARE_SIZE;

    *rotation_state = next_state;

    true
}

#[derive(Component)]
pub struct Hold;

#[derive(Event)]
pub struct MoveEvent;

#[derive(Event)]
pub struct RotateEvent;

pub fn user_rotate_active(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    placed_pieces: Res<PlacedPieces>,
    mut query: Query<
        (
            &Children,
            &mut Transform,
            &PieceType,
            &mut RotationState,
            &GlobalTransform,
        ),
        With<Active>,
    >,
    mut child_query: Query<(&GlobalTransform, &mut Transform), Without<Children>>,
    mut ev_rotate: EventWriter<RotateEvent>,
) {
    let (children, mut transform, piece_type, mut rotation_state, global_transform) =
        if let Ok(piece) = query.get_single_mut() {
            piece
        } else {
            return;
        };
    //rotate left
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        if rotate_clockwise(
            children,
            &mut rotation_state,
            placed_pieces,
            piece_type,
            &global_transform.translation(),
            &mut child_query,
            &mut transform.translation,
        ) {
            ev_rotate.send(RotateEvent);
        }
    }
    //rotate right
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        let mut translations_to_apply: Vec<Vec3> = vec![];
        for &child in children.iter() {
            let child_transform = child_query.get_mut(child).unwrap();
            let child_translation = child_transform.1.translation;
            let child_global_translation = child_transform.0.translation();
            let new_translation = Vec3::new(
                -child_translation.y,
                child_translation.x,
                child_translation.z,
            );
            let new_global_translation = child_global_translation + new_translation;
            fix_position(new_global_translation, &mut transform);

            translations_to_apply.push(new_translation);
            //child_transform.1.translation = new_translation;
        }

        for (i, &child) in children.iter().enumerate() {
            let mut child_transform = child_query.get_mut(child).unwrap();
            child_transform.1.translation = translations_to_apply[i];
        }
        ev_rotate.send(RotateEvent);
    }
}

#[derive(Event)]
pub struct AttemptPlaceEvent(pub Entity);

#[derive(Resource)]
pub struct GracePeriodTimer(pub Timer);

#[derive(Component)]
pub struct AttemptingPlace;

pub fn try_to_place_piece(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut ev_attempt_place: EventReader<AttemptPlaceEvent>,
    mut ev_piece_placed: EventWriter<PiecePlacedEvent>,
    mut grace_period_timer: ResMut<GracePeriodTimer>,
    mut query: Query<Entity, (With<Active>, With<Children>)>,
) {
    let ev = if let Some(events) = ev_attempt_place.read().next() {
        events
    } else {
        grace_period_timer.0.reset();
        let entity = if let Ok(piece) = query.get_single_mut() {
            piece
        } else {
            return;
        };
        commands.entity(entity).remove::<AttemptingPlace>();
        return;
    };

    let piece = if let Ok(piece) = query.get(ev.0) {
        piece
    } else {
        return;
    };

    commands.entity(piece).insert(AttemptingPlace);
    if !grace_period_timer.0.tick(time.delta()).just_finished() {
        ev_attempt_place.clear();
        return;
    }

    ev_piece_placed.send(PiecePlacedEvent(piece));

    commands.entity(piece).remove::<AttemptingPlace>();
    grace_period_timer.0.reset();
}

pub fn user_move_actives(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
    time: Res<Time>,
    mut movement_timer: ResMut<MovementTimer>,
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    mut ev_collision: EventReader<CollisionEvent>,
    mut ev_move: EventWriter<MoveEvent>,
    mut ev_rotate: EventReader<RotateEvent>,
) {
    if ev_rotate.read().count() > 0 {
        return;
    }
    if !movement_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut collisions: HashSet<&Collision> = HashSet::new();

    for collision in ev_collision.read() {
        collisions.extend(&collision.collision);
    }

    let (children, mut transform) = if let Ok(piece) = query.get_single_mut() {
        piece
    } else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowLeft) && !collisions.contains(&Collision::Right) {
        direction.x -= SQUARE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) && !collisions.contains(&Collision::Left) {
        direction.x += SQUARE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::Space) {
        // TODO: change to go down until collide
        direction.y += BOTTOM_GRID;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) && !collisions.contains(&Collision::Top) {
        direction.y -= SQUARE_SIZE;
    }

    for &child in children.iter() {
        let child_transform = child_query.get(child).unwrap();
        let child_translation = child_transform.translation();
        let new_translation = child_translation + direction;

        if new_translation.y < BOTTOM_GRID {
            direction.y = 0.0;
        }
    }

    if direction.x != 0.0 {
        ev_move.send(MoveEvent);
    }
    if direction.y != 0.0 && direction.x != 0.0 && collisions.contains(&Collision::Corner) {
        direction.y = 0.0;
    }

    transform.translation += direction;
    //print!("{:?}", transform.translation);
}

#[derive(Component)]
pub struct PauseMenu;

pub fn pause_game(
    game_state: Res<State<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    mut pause_menu_query: Query<&mut Visibility, With<PauseMenu>>,
    mut ev_pause: EventReader<PauseGameEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut pause_menu = pause_menu_query.single_mut();
    if ev_pause.read().next().is_some() {
        toggle_menu(
            game_state,
            &mut next_state,
            music_controller,
            &mut pause_menu,
        );
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        toggle_menu(
            game_state,
            &mut next_state,
            music_controller,
            &mut pause_menu,
        );
    }
}

pub fn toggle_menu(
    game_state: Res<State<GameState>>,
    next_state: &mut ResMut<NextState<GameState>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    menu_visibility: &mut Visibility,
) {
    match game_state.get() {
        GameState::Paused => {
            next_state.set(GameState::Playing);
            internal_pause_music(music_controller);
            *menu_visibility = Visibility::Hidden;
        }
        GameState::Playing => {
            next_state.set(GameState::Paused);
            internal_pause_music(music_controller);
            *menu_visibility = Visibility::Visible;
        }
        GameState::GameOver => {
            next_state.set(GameState::Playing);
            *menu_visibility = Visibility::Hidden;
        }
        GameState::Title => {
            next_state.set(GameState::Playing);
            *menu_visibility = Visibility::Hidden;
        }
    }
}

pub fn user_restart(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ev_restart: EventWriter<RestartGameEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        ev_restart.send(RestartGameEvent);
    }
}
