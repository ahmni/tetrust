use std::collections::HashSet;

use crate::{
    fix_position, internal_pause_music, Active, Collision, CollisionEvent, GameMusic, GameState,
    PauseGameEvent, PiecePlacedEvent, BOTTOM_GRID, LEFT_GRID, RIGHT_GRID, SQUARE_SIZE,
};
use bevy::prelude::*;

#[derive(Resource)]
pub struct MovementTimer(pub Timer);

#[derive(Component)]
pub struct Hold;

#[derive(Event)]
pub struct MoveEvent;

#[derive(Event)]
pub struct RotateEvent;

pub fn user_rotate_active(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    mut child_query: Query<(&GlobalTransform, &mut Transform), Without<Children>>,
    mut ev_rotate: EventWriter<RotateEvent>,
) {
    let (children, mut transform) = query.get_single_mut().unwrap();
    //rotate left
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        let mut translations_to_apply: Vec<Vec3> = vec![];
        for &child in children.iter() {
            let child_transform = child_query.get(child).unwrap();
            let child_translation = child_transform.1.translation;
            let child_global_translation = child_transform.0.translation();

            let new_translation = Vec3::new(
                child_translation.y,
                -child_translation.x,
                child_translation.z,
            );
            let new_global_translation = child_global_translation + new_translation;
            //println!("new translation {:?}", new_translation);
            //println!("new global translation {:?}", new_global_translation);
            fix_position(new_global_translation, &mut transform);

            translations_to_apply.push(new_translation);
        }

        for (i, &child) in children.iter().enumerate() {
            let mut child_transform = child_query.get_mut(child).unwrap();
            child_transform.1.translation = translations_to_apply[i];
        }
        ev_rotate.send(RotateEvent);
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

pub fn try_to_place_piece(
    mut ev_attempt_place: EventReader<AttemptPlaceEvent>,
    mut ev_piece_placed: EventWriter<PiecePlacedEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut grace_period_timer: ResMut<GracePeriodTimer>,
    query: Query<(&Children), With<Active>>,
    time: Res<Time>,
) {
    let ev_iter = ev_attempt_place.read().next();
    if ev_iter.is_none() {
        grace_period_timer.0.reset();
        return;
    }
    let ev = ev_iter.unwrap();

    if !grace_period_timer.0.tick(time.delta()).just_finished()
        && !keyboard_input.pressed(KeyCode::ArrowDown)
    {
        //    //ev_attempt_place.clear();
        return;
    }

    if !query.get(ev.0).is_ok() {
        return;
    }

    ev_piece_placed.send(PiecePlacedEvent(ev.0));

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
) {
    if !movement_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let default_collision_event = CollisionEvent {
        collision: HashSet::new(),
    };

    let collisions = &ev_collision
        .read()
        .next()
        .unwrap_or(&default_collision_event)
        .collision;

    let (children, mut transform) = query.get_single_mut().unwrap();

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
        if new_translation.x < LEFT_GRID || new_translation.x > RIGHT_GRID - SQUARE_SIZE {
            direction.x = 0.0;
        }

        if new_translation.y < BOTTOM_GRID {
            direction.y = 0.0;
        }
    }

    if direction.x != 0.0 {
        ev_move.send(MoveEvent);
    }
    transform.translation += direction;
    //print!("{:?}", transform.translation);
}

#[derive(Component)]
pub struct PauseMenu;

pub fn pause_game(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    pause_menu_query: Query<&mut Visibility, With<PauseMenu>>,
    mut ev_pause: EventReader<PauseGameEvent>,
) {
    if ev_pause.read().next().is_some() {
        toggle_pause(
            game_state,
            &mut next_state,
            music_controller,
            pause_menu_query,
        );
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        toggle_pause(
            game_state,
            &mut next_state,
            music_controller,
            pause_menu_query,
        );
    }
}

pub fn toggle_pause(
    game_state: Res<State<GameState>>,
    next_state: &mut ResMut<NextState<GameState>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    mut pause_menu_query: Query<&mut Visibility, With<PauseMenu>>,
) {
    match game_state.get() {
        GameState::Paused => {
            next_state.set(GameState::Playing);
            internal_pause_music(music_controller);
            *pause_menu_query.get_single_mut().unwrap() = Visibility::Hidden;
        }
        GameState::Playing => {
            next_state.set(GameState::Paused);
            internal_pause_music(music_controller);
            *pause_menu_query.get_single_mut().unwrap() = Visibility::Visible;
        }
        _ => {}
    }
}
