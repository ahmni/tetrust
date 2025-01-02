use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    build_piece, get_random_piece, Active, ClearEvent, Collision, CollisionEvent, DropPieceEvent,
    GameOverEvent, Hold, PieceType, Placed, RestartGameEvent, RotateEvent, BOTTOM_GRID, LEFT_GRID,
    RIGHT_GRID, SQUARE_SIZE, TOP_GRID,
};

#[derive(Resource)]
pub struct DropTimer(pub Timer);

#[derive(Resource)]
pub struct NextPieces(pub Vec<Entity>);

pub struct CanHoldPiece(bool);

impl Default for CanHoldPiece {
    fn default() -> Self {
        CanHoldPiece(true)
    }
}

#[derive(Resource, Debug)]
pub struct PlacedPieces(pub Vec<Vec<Option<Entity>>>);

//#[derive(Resource)]
//struct PlaceGracePeriod(Timer);

#[derive(Event)]
pub struct PiecePlacedEvent(pub Entity);

#[derive(Event)]
pub struct HoldPieceEvent;

// start a timer
// Add component to piece marking that it is trying to be placed
//      Remove this component if not colliding with bottom_grid or another piece
// if timer finishes and has marker component, place piece`

pub fn place_piece(
    query: Query<&Children, With<Active>>,
    mut child_query: Query<(&GlobalTransform, &mut Transform), Without<Children>>,
    mut commands: Commands,
    mut next_piece_query: Query<(&Children, &mut Transform, &PieceType), With<Children>>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
    mut ev_clear: EventWriter<ClearEvent>,
    mut next_pieces: ResMut<NextPieces>,
    mut placed_pieces: ResMut<PlacedPieces>,
    mut ev_game_over: EventWriter<GameOverEvent>,
) {
    let ev = if let Some(event) = ev_piece_placed.read().next() {
        event
    } else {
        return;
    };

    // place piece
    let children = query.get(ev.0).unwrap();
    for &child in children.iter() {
        let (child_global_transform, mut child_transform) = child_query.get_mut(child).unwrap();
        let row = get_row_no_clamp(child_global_transform.translation().y);
        let col = get_col(child_global_transform.translation().x);

        *child_transform = child_global_transform.compute_transform();
        println!("row: {}, col: {}", row, col);
        if row >= 20 {
            println!("game over");
            ev_game_over.send(GameOverEvent);
            return;
        }
        placed_pieces.0[row][col] = Some(child);

        commands.entity(child).remove::<Active>();
        commands.entity(child).insert(Placed);
        commands.entity(child).remove_parent();
    }
    commands.entity(ev.0).despawn();

    // check for full rows
    let rows_to_remove: Vec<usize> = placed_pieces
        .0
        .iter()
        .enumerate()
        .filter(|(_, row)| row.iter().all(|&cell| cell.is_some()))
        .filter(|(_, row)| row.len() == 10)
        .map(|(i, _)| i)
        .collect();
    //println!("{:?}", rows_to_remove);
    //println!("{:?}", placed_pieces);

    if rows_to_remove.len() > 0 {
        println!("placed_pieces: {:?}", placed_pieces.0);
        ev_clear.send(ClearEvent(rows_to_remove));
        return;
    }

    // get next piece
    let next_piece = next_pieces.0.remove(0);
    let (children, mut transform, piece_type) = next_piece_query.get_mut(next_piece).unwrap();
    for child in children {
        commands.entity(*child).insert(Active);
    }
    commands.entity(next_piece).insert(Active);
    // if square or straight, position needs to move shifted up and to the right by SQUARE_SIZE
    // / 2.0
    move_piece_to_board(&piece_type, &mut transform.translation, &placed_pieces);

    let new_piece = get_random_piece();
    let entities = build_piece(
        &mut commands,
        &new_piece,
        Vec3::new(RIGHT_GRID * 1.5, 90.0, -1.0),
    );
    next_pieces.0.push(entities[0]);
}

pub fn move_piece_to_board(
    piece_type: &PieceType,
    translation: &mut Vec3,
    placed_pieces: &PlacedPieces,
) {
    match piece_type {
        PieceType::Square => {
            *translation = Vec3::new(-SQUARE_SIZE / 2.0, 270.0 - SQUARE_SIZE / 2., -1.0);
        }
        PieceType::Straight => {
            *translation = Vec3::new(SQUARE_SIZE / 2.0, 240.0 + SQUARE_SIZE / 2., -1.0);
        }
        _ => {
            *translation = Vec3::new(0.0, 240.0, -1.0);
        }
    }

    // check if we need to move the piece up to avoid collision with placed pieces
    let avoid_collision_offset = placed_pieces
        .0
        .iter()
        .take(2)
        .filter(|row| row.iter().any(|cell| cell.is_some()))
        .collect::<Vec<_>>()
        .len() as f32
        * SQUARE_SIZE;
    translation.y += avoid_collision_offset;

    println!("avoid_collision_offset : {:?}", avoid_collision_offset);
}

pub fn hold_piece(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    placed_pieces: Res<PlacedPieces>,
    mut commands: Commands,
    mut query: Query<(&Children, Entity, &mut Transform), (With<Active>, Without<Hold>)>,
    mut held_query: Query<(&Children, &PieceType, Entity, &mut Transform), With<Hold>>,
    mut next_piece_query: Query<
        (&Children, &mut Transform, &PieceType),
        (With<Children>, Without<Hold>, Without<Active>),
    >,
    mut next_pieces: ResMut<NextPieces>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
    mut ev_hold: EventWriter<HoldPieceEvent>,
    mut ev_reset_game: EventReader<RestartGameEvent>,
    // TODO: turn into component
    mut can_hold: Local<CanHoldPiece>,
) {
    if ev_reset_game.read().count() > 0 {
        can_hold.0 = true;
        ev_reset_game.clear();
    }
    if ev_piece_placed.read().count() > 0 {
        can_hold.0 = true;
    }
    if !keyboard_input.just_pressed(KeyCode::KeyC) {
        return;
    }
    if !can_hold.0 {
        return;
    }

    can_hold.0 = false;

    let (children, entity, mut transform) = if let Ok(piece) = query.get_single_mut() {
        piece
    } else {
        return;
    };

    for &child in children.iter() {
        commands.entity(child).remove::<Active>();
        commands.entity(child).insert(Hold);
    }

    commands.entity(entity).remove::<Active>();
    commands.entity(entity).insert(Hold);

    // move to hold
    move_to_hold(&mut transform.translation);
    if let Ok((held_children, piece_type, held_entity, mut transform)) = held_query.get_single_mut()
    {
        for &child in held_children.iter() {
            commands.entity(child).remove::<Hold>();
            commands.entity(child).insert(Active);
        }

        commands.entity(held_entity).remove::<Hold>();
        commands.entity(held_entity).insert(Active);
        // move to board
        move_piece_to_board(&piece_type, &mut transform.translation, &placed_pieces);
    } else {
        // pull from nextPieces
        let next_piece = next_pieces.0.remove(0);
        let (children, mut next_transform, piece_type) =
            next_piece_query.get_mut(next_piece).unwrap();
        for child in children {
            commands.entity(*child).insert(Active);
        }
        commands.entity(next_piece).insert(Active);
        // if square or straight, position needs to move shifted up and to the right by SQUARE_SIZE
        // / 2.0
        move_piece_to_board(&piece_type, &mut next_transform.translation, &placed_pieces);
        let new_piece = get_random_piece();
        let entities = build_piece(
            &mut commands,
            &new_piece,
            Vec3::new(0.0, NEXT_PIECE_Y, -1.0),
        );
        next_pieces.0.push(entities[0]);
    }

    ev_hold.send(HoldPieceEvent);
}

pub fn get_row_no_clamp(translation_y: f32) -> usize {
    let row = ((TOP_GRID - SQUARE_SIZE) - translation_y) / SQUARE_SIZE;
    if row < 0.0 {
        return 20;
    }
    return row as usize;
}

pub fn get_row(translation_y: f32) -> usize {
    let row = (((TOP_GRID - SQUARE_SIZE) - translation_y) / SQUARE_SIZE) as usize;
    return row.clamp(0, 19);
}

pub fn get_col(translation_x: f32) -> usize {
    let col = ((translation_x - LEFT_GRID) / SQUARE_SIZE) as usize;

    return col.clamp(0, 9);
}

pub const HOLD_PIECE_Y: f32 = 160.0;
pub const NEXT_PIECE_Y: f32 = 140.0;

fn move_to_hold(translation: &mut Vec3) {
    *translation = Vec3::new(LEFT_GRID - SQUARE_SIZE * 4.0, HOLD_PIECE_Y - 20.0, -1.0);
}

pub fn position_next_pieces(next_pieces: ResMut<NextPieces>, mut query: Query<&mut Transform>) {
    for (i, &piece) in next_pieces.0.iter().enumerate() {
        let translation = Vec3::new(
            RIGHT_GRID * 1.5,
            NEXT_PIECE_Y - i as f32 * SQUARE_SIZE * 3.,
            -1.0,
        );
        let mut transform = query.get_mut(piece).unwrap();
        transform.translation = translation;
    }
}

pub fn shift_active_down(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<Active>, Without<Parent>)>,
    mut timer: ResMut<DropTimer>,
    mut ev_collision: EventReader<CollisionEvent>,
    mut ev_rotate: EventReader<RotateEvent>,
    mut ev_drop_piece: EventReader<DropPieceEvent>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
) {
    if ev_rotate.read().count() > 0 {
        return;
    }
    if ev_drop_piece.read().count() > 0 {
        return;
    }
    if ev_piece_placed.read().count() > 0 {
        return;
    }

    let mut transform = if let Ok(piece) = query.get_single_mut() {
        piece
    } else {
        return;
    };

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        timer.0.reset();
    }

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut collisions: HashSet<&Collision> = HashSet::new();

    for collision in ev_collision.read() {
        collisions.extend(&collision.collision);
    }

    if collisions.contains(&Collision::Top) {
        return;
    }

    if transform.translation.y > BOTTOM_GRID - SQUARE_SIZE
        && !keyboard_input.pressed(KeyCode::ArrowDown)
    {
        transform.translation.y -= SQUARE_SIZE;
    }

    //println!("{:?}", transform.translation);
}
