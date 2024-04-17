use crate::{fix_position, Active, BOTTOM_GRID, LEFT_GRID, RIGHT_GRID, SQUARE_SIZE, TOP_GRID};
use bevy::prelude::*;

#[derive(Resource)]
pub struct MovementTimer(pub Timer);

#[derive(Component)]
pub struct Hold;

pub fn user_rotate_active(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    mut child_query: Query<(&GlobalTransform, &mut Transform), Without<Children>>,
) {
    println!("query {:?}", query);
    let (children, mut transform) = query.get_single_mut().unwrap();
    //rotate left
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
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
    }
    //rotate right
    if keyboard_input.just_pressed(KeyCode::KeyE) {
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
    }
}

pub fn user_move_actives(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
    time: Res<Time>,
    mut movement_timer: ResMut<MovementTimer>,
) {
    if !movement_timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let (children, mut transform) = query.get_single_mut().unwrap();

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= SQUARE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += SQUARE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::Space) {
        // TODO: change to go down until collide
        direction.y += BOTTOM_GRID;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= SQUARE_SIZE;
    }

    for &child in children.iter() {
        let child_transform = child_query.get(child).unwrap();
        let child_translation = child_transform.translation();
        let new_translation = child_translation + direction;
        if new_translation.x < LEFT_GRID
            || new_translation.x > RIGHT_GRID - SQUARE_SIZE
            || new_translation.y < BOTTOM_GRID
        {
            return;
        }
    }

    transform.translation += direction;
    //print!("{:?}", transform.translation);
}
