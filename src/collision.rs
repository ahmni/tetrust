use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};

use crate::{Active, BOTTOM_GRID, LEFT_GRID, RIGHT_GRID, SQUARE_SIZE, TOP_GRID};

// Collider system:
//
// - Check if any active piece is colliding with left or right wall
// - If so, push piece back inside the game
// - Also check if any active piece is colliding with bottom wall
// - If so, deactivate the piece and spawn a new one. Store old piece y position
//      - check if tetris has been made
#[derive(Component)]
pub struct Collider;

#[derive(Debug)]
enum Collision {
    Left,
    Right,
    Bottom,
    Top,
}

pub fn check_in_bounds(
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    child_query: Query<(&GlobalTransform), Without<Children>>,
) {
    let (children, mut transform) = query.get_single_mut().unwrap();

    for &child in children.iter() {
        let child_global_transform = child_query.get(child).unwrap();
        let child_global_translation = child_global_transform.translation();
        fix_position(child_global_translation, &mut transform);
    }
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
