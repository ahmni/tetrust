use crate::{
    is_valid_positions, Active, ClearEvent, GameOverEvent, Ghost, HoldPieceEvent, PiecePlacedEvent,
    PlacedPieces, SQUARE_SIZE, TOP_GRID,
};
use bevy::prelude::*;

#[derive(Default)]
pub struct GhostState {
    should_update_position: bool,
}

pub fn update_ghost_position(
    active_query: Query<(&Children, &Transform), With<Active>>,
    placed_pieces: Res<PlacedPieces>,
    ev_piece_placed: EventReader<PiecePlacedEvent>,
    ev_hold: EventReader<HoldPieceEvent>,
    ev_clear: EventReader<ClearEvent>,
    ev_game_over: EventReader<GameOverEvent>,
    mut state: Local<GhostState>,
    mut ghost_query: Query<(&Children, &mut Transform), (With<Ghost>, Without<Active>)>,
    mut child_query: Query<&mut Transform, Without<Children>>,
) {
    if ev_game_over.len() > 0 {
        return;
    }

    let (active_children, active_transform) = if let Ok(piece) = active_query.get_single() {
        piece
    } else {
        // if no active piece, we don't update the ghost position even if an event was received
        // so we should update once we get a new active piece
        state.should_update_position = true;
        return;
    };

    let (ghost_children, mut ghost_transform) = if let Ok(piece) = ghost_query.get_single_mut() {
        piece
    } else {
        return;
    };

    ghost_transform.translation.x = active_transform.translation.x;
    ghost_transform.translation.y = active_transform.translation.y;

    // only update ghost y position if piece was placed or held. Necessary as the global transform
    // origin can be different depending on the piece type
    if !ev_piece_placed.is_empty()
        || !ev_hold.is_empty()
        || !ev_clear.is_empty()
        || state.should_update_position
    {
        state.should_update_position = false;
        println!("active position {}", active_transform.translation.y);
    }

    let mut child_global_translations: Vec<Vec3> = Vec::new();
    for (active_child, ghost_child) in active_children.iter().zip(ghost_children.iter()) {
        let active_child_translation = child_query.get(*active_child).unwrap().translation;
        let mut ghost_child_transform = child_query.get_mut(*ghost_child).unwrap();
        ghost_child_transform.translation = active_child_translation;

        // child global transform at this point is not updated, so we need to update it manually
        // for position calculations
        let ghost_global_transform: GlobalTransform =
            *ghost_transform.as_ref() * GlobalTransform::from(*ghost_child_transform);
        child_global_translations.push(ghost_global_transform.translation());
    }

    // keep ghost piece at the bottom of the grid
    while is_valid_positions(&child_global_translations, &placed_pieces) {
        ghost_transform.translation.y -= SQUARE_SIZE;
        for translation in child_global_translations.iter_mut() {
            translation.y -= SQUARE_SIZE;
        }
    }

    // go up until position is valid
    while !is_valid_positions(&child_global_translations, &placed_pieces)
        && ghost_transform.translation.y < TOP_GRID
    {
        ghost_transform.translation.y += SQUARE_SIZE;
        for translation in child_global_translations.iter_mut() {
            translation.y += SQUARE_SIZE;
        }
    }
}
