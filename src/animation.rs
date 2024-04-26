use crate::{
    build_piece, get_random_piece, move_piece_to_board, Active, AttemptingPlace, NextPieces,
    PiecePlacedEvent, PieceType, PlacedPieces, RIGHT_GRID, SQUARE_SIZE,
};
use bevy::prelude::*;
#[derive(Resource)]
pub struct AttemptingPlaceAnimationTimer(pub Timer);

pub fn placing_piece_animation(
    query: Query<(&mut Children, Option<&AttemptingPlace>), With<Active>>,
    mut child_query: Query<&mut Sprite>,
    mut timer: ResMut<AttemptingPlaceAnimationTimer>,
    time: Res<Time>,
) {
    let (children, _) = if let Ok((children, None)) = query.get_single() {
        for child in children.iter() {
            let mut sprite = child_query.get_mut(*child).unwrap();
            sprite.color.set_a(1.0);
        }
        return;
    } else if let Ok((children, Some(_))) = query.get_single() {
        println!("attempting to place piece");
        (children, ())
    } else {
        return;
    };

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for child in children.iter() {
        let mut sprite = child_query.get_mut(*child).unwrap();
        let old_alpha = sprite.color.a();
        sprite
            .color
            .set_a(old_alpha - timer.0.duration().as_secs_f32());
        println!("timer duration: {}", timer.0.elapsed_secs());
    }
}

#[derive(Resource)]
pub struct FlashingAnimationTimer(pub Timer);

pub fn place_piece_animation(
    mut ev_place_piece: EventReader<PiecePlacedEvent>,
    query: Query<&Children>,
    mut child_query: Query<&mut Sprite>,
    mut flashed: Local<Vec<Entity>>,
    mut timer: ResMut<FlashingAnimationTimer>,
    time: Res<Time>,
) {
    if !flashed.is_empty() {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }
        for entity in flashed.iter() {
            if let Ok(mut sprite) = child_query.get_mut(*entity) {
                sprite.color.set_l(0.5);
            }
        }
        flashed.clear();
        return;
    }

    for ev in ev_place_piece.read() {
        let children = if let Ok(children) = query.get(ev.0) {
            children
        } else {
            return;
        };

        for child in children.iter() {
            let mut sprite = child_query.get_mut(*child).unwrap();
            sprite.color.set_a(1.0);
            sprite.color.set_l(0.8);
            flashed.push(*child);
        }
    }
}

#[derive(Resource)]
pub struct ClearingAnimationTimer(pub Timer);

pub fn clear_rows(
    time: Res<Time>,
    mut commands: Commands,
    mut ev_clear: EventReader<crate::ClearEvent>,
    mut placed_pieces: ResMut<PlacedPieces>,
    mut clearing: Local<Vec<Vec<(usize, usize)>>>,
    mut rows_to_remove: Local<Vec<usize>>,
    mut timer: ResMut<ClearingAnimationTimer>,
    mut child_query: Query<&mut Transform, Without<Children>>,
    mut next_pieces: ResMut<NextPieces>,
    mut next_piece_query: Query<(&Children, &mut Transform, &PieceType), With<Children>>,
) {
    if !clearing.is_empty() {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }
        match clearing.pop() {
            Some(vec) => {
                for (row, col) in vec {
                    let entity = if let Some(entity) = placed_pieces.0[row][col] {
                        entity
                    } else {
                        continue;
                    };
                    commands.entity(entity).despawn();
                    println!("despawning entity: {:?} at {row} {col}", entity);
                    placed_pieces.0[row].remove(col);
                    println!("clearing: {:?}", clearing);
                }
                if clearing.is_empty() {
                    // shift down
                    let biggest_row = rows_to_remove.iter().max().unwrap_or(&0);
                    let mut amount_to_shift = 1;
                    for i in (0..*biggest_row).rev() {
                        let row = &mut placed_pieces.0[i];
                        if rows_to_remove.contains(&i) {
                            amount_to_shift += 1;
                            continue;
                        }
                        for entity in row.iter().filter(|&entity| entity.is_some()) {
                            let mut transform = if let Ok(it) = child_query.get_mut(entity.unwrap())
                            {
                                it
                            } else {
                                continue;
                            };
                            transform.translation.y -= SQUARE_SIZE * amount_to_shift as f32;
                        }
                        placed_pieces.0[i + amount_to_shift] = row.clone();
                        let row = &mut placed_pieces.0[i];
                        for entity in row.iter_mut() {
                            *entity = None;
                        }
                    }

                    let next_piece = next_pieces.0.remove(0);
                    let (children, mut transform, piece_type) =
                        next_piece_query.get_mut(next_piece).unwrap();
                    for child in children {
                        commands.entity(*child).insert(Active);
                    }
                    commands.entity(next_piece).insert(Active);
                    // if square or straight, position needs to move shifted up and to the right by SQUARE_SIZE
                    // / 2.0
                    move_piece_to_board(&piece_type, &mut transform.translation);

                    let new_piece = get_random_piece();
                    let entities = build_piece(
                        &mut commands,
                        new_piece,
                        Vec3::new(RIGHT_GRID * 1.5, 90.0, -1.0),
                    );
                    next_pieces.0.push(entities[0]);
                }
                return;
            }

            None => {}
        }
    }
    for ev in ev_clear.read() {
        for col in 0..10 {
            let mut indices: Vec<(usize, usize)> = vec![];
            for row in &ev.0 {
                indices.push((*row, col));
            }
            clearing.push(indices);
        }
        *rows_to_remove = ev.0.clone();
    }
}
