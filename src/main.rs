mod piece;
mod wall;

use bevy::prelude::*;
use piece::*;
use wall::*;

#[derive(Component)]
struct Collider;

fn setup(mut commands: Commands) {
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

    let starting_piece: PieceType = PieceType::Straight;

    build_piece(&mut commands, starting_piece, Vec3::new(0.0, 0.0, -1.0));
}

#[derive(Event)]
struct PiecePlacedEvent(Entity);

fn user_rotate_active(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&Children, With<Active>>,
    mut child_query: Query<(&GlobalTransform, &mut Transform), Without<Children>>,
) {
    for children in query.iter_mut() {
        //rotate left
        if keyboard_input.just_pressed(KeyCode::KeyQ) {
            let mut translations_to_apply: Vec<Vec3> = vec![];
            for &child in children.iter() {
                let child_transform = child_query.get_mut(child).unwrap();
                let child_translation = child_transform.1.translation;
                let child_global_translation = child_transform.0.translation();

                let new_translation = Vec3::new(
                    child_translation.y,
                    -child_translation.x,
                    child_translation.z,
                );
                let new_global_translation = child_global_translation + new_translation;
                println!("new translation {:?}", new_translation);
                // TODO: instead of stopping rotation, allow and led collision system handle
                if new_global_translation.x < LEFT_GRID
                    || new_global_translation.x > RIGHT_GRID - SQUARE_SIZE
                    || new_global_translation.y < BOTTOM_GRID
                {
                    return;
                }

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
                println!("new translation {:?}", new_translation);
                println!("new global translation {:?}", new_global_translation);
                if new_global_translation.x < LEFT_GRID
                    || new_global_translation.x > RIGHT_GRID - SQUARE_SIZE
                    || new_global_translation.y < BOTTOM_GRID
                {
                    return;
                }
                translations_to_apply.push(new_translation);
                //child_transform.1.translation = new_translation;
            }

            for (i, &child) in children.iter().enumerate() {
                let mut child_transform = child_query.get_mut(child).unwrap();
                child_transform.1.translation = translations_to_apply[i];
            }
        }
    }
}

fn user_move_actives(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Children, &mut Transform), With<Active>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
    time: Res<Time>,
    mut movement_timer: ResMut<MovementTimer>,
) {
    if !movement_timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for (children, mut transform) in query.iter_mut() {
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

        //println!("iterating through children");
        for &child in children.iter() {
            let child_transform = child_query.get(child).unwrap();
            let child_translation = child_transform.translation();
            let new_translation = child_translation + direction;
            //println!("new translation {:?}", new_translation);
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
}

fn move_actives(
    mut ev_piece_placed: EventWriter<PiecePlacedEvent>,
    mut query: Query<(&Children, Entity, &mut Transform), With<Active>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
    time: Res<Time>,
    mut timer: ResMut<ActiveTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let (children, entity, mut transform) = query.get_single_mut().unwrap();

    for &child in children.iter() {
        let child_transform = child_query.get(child).unwrap();
        let child_translation = child_transform.translation().y;
        let new_translation = child_translation - SQUARE_SIZE;
        //println!("new translation {:?}", new_translation);
        if new_translation < BOTTOM_GRID {
            println!("piece placed");
            ev_piece_placed.send(PiecePlacedEvent(entity));
            return;
        }
    }

    transform.translation.y -= SQUARE_SIZE;

    println!("{:?}", transform.translation);
}

fn piece_placed(
    mut commands: Commands,
    query: Query<&Children, With<Active>>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
) {
    for ev in ev_piece_placed.read() {
        let children = query.get(ev.0).unwrap();
        for &child in children.iter() {
            commands.entity(child).remove::<Active>();
        }
        commands.entity(ev.0).remove::<Active>();
        let new_piece = get_random_piece();
        build_piece(&mut commands, new_piece, Vec3::new(0.0, 0.0, -1.0));
    }
}

// Collider system:
// - Check if any active piece is colliding with left or right wall
// - If so, push peace back inside the game
// - Also check if any active piece is colliding with bottom wall
// - If so, deactivate the piece and spawn a new one. Store old piece y position

#[derive(Resource)]
struct ActiveTimer(Timer);

#[derive(Resource)]
struct MovementTimer(Timer);

fn main() {
    App::new()
        .insert_resource(ActiveTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(MovementTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .add_event::<PiecePlacedEvent>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_actives,
                user_move_actives,
                user_rotate_active,
                piece_placed,
            ),
        )
        .run();
}
