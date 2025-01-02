use std::collections::HashSet;

use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

use crate::{Active, AttemptPlaceEvent, BOTTOM_GRID, LEFT_GRID, RIGHT_GRID, SQUARE_SIZE, TOP_GRID};

// - Check if any active piece is colliding with left or right wall
// - If so, push piece back inside the game
// - Also check if any active piece is colliding with bottom wall
// - If so, deactivate the piece and spawn a new one. Store old piece y position
//      - check if tetris has been made

#[derive(Component)]
pub struct Placed;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Collision {
    Left,
    Right,
    Top,
    Bottom,
    Corner,
}

#[derive(Event)]
pub struct CollisionEvent {
    pub collision: HashSet<Collision>,
}

pub fn check_collision(
    child_query: Query<&GlobalTransform, Without<Children>>,
    collidee_query: Query<&GlobalTransform, (With<Placed>, Without<Active>)>,
    mut query: Query<(&Children, Entity), With<Active>>,
    mut ev_attempt_place: EventWriter<AttemptPlaceEvent>,
    mut ev_collision: EventWriter<CollisionEvent>,
) {
    let (children, entity) = if let Ok(piece) = query.get_single_mut() {
        piece
    } else {
        return;
    };

    let mut collision_set: HashSet<Collision> = HashSet::new();
    let mut should_place_piece = check_wall_collision(children, &child_query, &mut collision_set);

    check_piece_collision(collidee_query, children, &child_query, &mut collision_set);

    if !collision_set.is_empty() {
        ev_collision.send(CollisionEvent {
            collision: collision_set.clone(),
        });
    }

    if collision_set.contains(&Collision::Top) {
        should_place_piece = true;
    }

    if should_place_piece {
        ev_attempt_place.send(AttemptPlaceEvent(entity));
    }

    //if is_game_over {
    //    ev_game_over.send(GameOverEvent);
    //}
}

pub fn check_piece_collision(
    collidee_query: Query<&GlobalTransform, (With<Placed>, Without<Active>)>,
    children: &Children,
    child_query: &Query<&GlobalTransform, Without<Children>>,
    collision_set: &mut HashSet<Collision>,
) -> bool {
    let mut is_game_over = false;
    for collider_transform in collidee_query.iter() {
        for &child in children.iter() {
            let global_transform = child_query.get(child).unwrap();
            let collision = collision(
                BoundingCircle::new(global_transform.translation().truncate(), SQUARE_SIZE / 2.),
                Aabb2d::new(
                    collider_transform.translation().truncate(),
                    Vec2::new(SQUARE_SIZE / 2., SQUARE_SIZE / 2.),
                ),
            );

            if let Some(collision) = collision {
                //println!("collision: {:?}", collision);
                match collision {
                    Collision::Left => {
                        collision_set.insert(Collision::Left);
                    }
                    Collision::Right => {
                        collision_set.insert(Collision::Right);
                    }
                    Collision::Top => {
                        if global_transform.translation().y > TOP_GRID - SQUARE_SIZE * 2.0 {
                            is_game_over = true;
                        }
                        collision_set.insert(Collision::Top);
                    }
                    Collision::Corner => {
                        collision_set.insert(Collision::Corner);
                    }
                    _ => {}
                }
            }
        }
    }

    is_game_over
}

pub fn check_wall_collision(
    children: &Children,
    child_query: &Query<&GlobalTransform, Without<Children>>,
    ev_collision: &mut HashSet<Collision>,
) -> bool {
    let mut should_place_piece = false;
    for &child in children.iter() {
        let child_transform = child_query.get(child).unwrap();
        let child_translation = child_transform.translation();
        // note: currently we only care about colliding with bottom grid
        if child_translation.x <= LEFT_GRID {
            ev_collision.insert(Collision::Right);
        }
        if child_translation.x >= RIGHT_GRID - SQUARE_SIZE {
            ev_collision.insert(Collision::Left);
        }
        if child_translation.y <= BOTTOM_GRID {
            ev_collision.insert(Collision::Top);
            should_place_piece = true;
        }
        if child_translation.y > TOP_GRID - SQUARE_SIZE {}
    }

    should_place_piece
}

fn collision(collider_bb: BoundingCircle, collidee_bb: Aabb2d) -> Option<Collision> {
    if !collider_bb.intersects(&collidee_bb) {
        if collider_bb.aabb_2d().intersects(&collidee_bb) {
            return Some(Collision::Corner);
        }
        return None;
    }

    let closest = collidee_bb.closest_point(collider_bb.center());
    let offset = collider_bb.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    return Some(side);
}
