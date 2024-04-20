use std::collections::HashSet;

use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

use crate::{Active, PiecePlacedEvent, BOTTOM_GRID, LEFT_GRID, RIGHT_GRID, SQUARE_SIZE, TOP_GRID};

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
}

#[derive(Event)]
pub struct CollisionEvent {
    pub collision: HashSet<Collision>,
}

pub fn check_in_bounds(
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
) {
    let (children, mut transform) = query.get_single_mut().unwrap();

    for &child in children.iter() {
        let child_global_transform = child_query.get(child).unwrap();
        let child_global_translation = child_global_transform.translation();
        fix_position(child_global_translation, &mut transform);
    }
}

pub fn check_collision(
    mut query: Query<(&Children, Entity), With<Active>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
    collidee_query: Query<&GlobalTransform, (With<Placed>, Without<Active>)>,
    mut ev_piece_placed: EventWriter<PiecePlacedEvent>,
    mut ev_collision: EventWriter<CollisionEvent>,
) {
    let (children, entity) = query.get_single_mut().unwrap();

    if check_wall_collision(children, &entity, &child_query, &mut ev_piece_placed) {
        return;
    }

    let mut collision_set: HashSet<Collision> = HashSet::new();

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
                        //transform.translation.y += SQUARE_SIZE;
                        if global_transform.translation().y > TOP_GRID - SQUARE_SIZE * 2.0 {
                            // TODO: implement game over state
                            panic!("Game Over");
                        }
                        ev_piece_placed.send(PiecePlacedEvent(entity));
                        return;
                    }
                    Collision::Bottom => {}
                }
            }
        }
    }

    if !collision_set.is_empty() {
        ev_collision.send(CollisionEvent {
            collision: collision_set,
        });
    }
}

fn check_wall_collision(
    children: &Children,
    entity: &Entity,
    child_query: &Query<&GlobalTransform, Without<Children>>,
    ev_piece_placed: &mut EventWriter<PiecePlacedEvent>,
) -> bool {
    for &child in children.iter() {
        let child_transform = child_query.get(child).unwrap();
        let child_translation = child_transform.translation();
        if child_translation.x < LEFT_GRID {}
        if child_translation.x > RIGHT_GRID - SQUARE_SIZE {}
        if child_translation.y <= BOTTOM_GRID {
            ev_piece_placed.send(PiecePlacedEvent(*entity));
            return true;
        }
        if child_translation.y > TOP_GRID - SQUARE_SIZE {}
    }
    false
}

// TODO: Square rotate does not work with this function
pub fn fix_position(translation: Vec3, transform: &mut Transform) {
    let mut fixed_position = false;
    if translation.x < LEFT_GRID {
        transform.translation.x += SQUARE_SIZE;
        fixed_position = true;
    }
    if translation.x > RIGHT_GRID - SQUARE_SIZE {
        transform.translation.x -= SQUARE_SIZE;
        fixed_position = true;
    }
    if translation.y < BOTTOM_GRID {
        println!("translation.y: {}", translation.y);
        transform.translation.y += SQUARE_SIZE;
        fixed_position = true;
    }
    if translation.y > TOP_GRID - SQUARE_SIZE {
        transform.translation.y -= SQUARE_SIZE;
        fixed_position = true;
    }
    if fixed_position {
        println!("fixed position");
    }
}

// Currently
fn collision(collider_bb: BoundingCircle, collidee_bb: Aabb2d) -> Option<Collision> {
    if !collider_bb.intersects(&collidee_bb) {
        return None;
    }

    //println!("collider_bb: {:?}", collider_bb);
    //println!("collidee_bb: {:?}", collidee_bb);

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
