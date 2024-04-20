mod collision;
mod piece;
mod user_actions;
mod wall;

use bevy::prelude::*;
use collision::*;
use piece::*;
use user_actions::*;
use wall::*;

fn setup(
    mut commands: Commands,
    mut next_pieces: ResMut<NextPieces>,
    mut placed_pieces: ResMut<PlacedPieces>,
) {
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

    // build placed piece grid
    for _ in 0..20 {
        let row: Vec<Entity> = Vec::new();
        placed_pieces.0.push(row);
    }

    let starting_piece: PieceType = PieceType::Straight;

    build_active_piece(&mut commands, starting_piece, Vec3::new(0.0, 240.0, -1.0));

    for _ in 0..3 {
        let new_piece = get_random_piece();
        let entities = build_piece(&mut commands, new_piece, Vec3::new(0.0, 0.0, -1.0));
        // only push the parent which is always the first entiy
        next_pieces.0.push(entities[0]);
    }
}

#[derive(Event)]
struct PiecePlacedEvent(Entity);

fn shift_active_down(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    child_query: Query<&GlobalTransform, Without<Children>>,
    time: Res<Time>,
    mut ev_piece_placed: EventWriter<PiecePlacedEvent>,
    mut query: Query<(&Children, Entity, &mut Transform), (With<Active>, Without<Parent>)>,
    mut timer: ResMut<ActiveTimer>,
) {
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        timer.0.reset();
    }

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let (children, entity, mut transform) = query.get_single_mut().unwrap();

    println!("{:?}", transform.translation.y);
    if transform.translation.y > BOTTOM_GRID - SQUARE_SIZE
        && !keyboard_input.pressed(KeyCode::ArrowDown)
    {
        transform.translation.y -= SQUARE_SIZE;
    }

    println!("{:?}", transform.translation);
}

// start a timer
// Add component to piece marking that it is trying to be placed
//      Remove this component if not colliding with bottom_grid or another piece
// if timer finishes and has marker component, place piece`

#[derive(Component)]
struct Placing;

fn place_piece(
    mut commands: Commands,
    query: Query<&Children, With<Active>>,
    mut child_query: Query<(&GlobalTransform, &mut Transform), Without<Children>>,
    mut next_piece_query: Query<(&Children, &mut Transform, &PieceType), With<Children>>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
    mut next_pieces: ResMut<NextPieces>,
    mut placed_pieces: ResMut<PlacedPieces>,
    mut timer: ResMut<PlaceGracePeriod>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    //println!("should place piece");
    //
    //println!("timer elapsed: {:?}", timer.0.elapsed());
    //
    //if !timer.0.finished() {
    //    return;
    //}
    //println!("placing piece");
    let ev = ev_piece_placed.read().next();
    if ev.is_none() {
        return;
    }
    let ev = ev.unwrap();

    // place piece
    let children = query.get(ev.0).unwrap();
    for &child in children.iter() {
        let (child_global_transform, mut child_transform) = child_query.get_mut(child).unwrap();
        let row = get_row(child_global_transform.translation().y);
        *child_transform = child_global_transform.compute_transform();
        placed_pieces.0[row].push(child);
        commands.entity(child).remove::<Active>();
        commands.entity(child).insert(Placed);
        commands.entity(child).remove_parent();
    }
    commands.entity(ev.0).remove::<Active>();
    commands.entity(ev.0).insert(Placed);
    commands.entity(ev.0).despawn();

    // check for full rows
    let rows_to_remove: Vec<usize> = placed_pieces
        .0
        .iter()
        .enumerate()
        .filter(|(_, row)| row.len() == 10)
        .map(|(i, _)| i)
        .collect();
    println!("{:?}", rows_to_remove);
    println!("{:?}", placed_pieces);

    // despawn entities in full rows
    for row in &rows_to_remove {
        for entity in &placed_pieces.0[*row] {
            commands.entity(*entity).despawn();
        }
        placed_pieces.0[*row].clear();
    }
    println!("pieces removed after {:?}", placed_pieces);

    // shift rows down
    let biggest_row = rows_to_remove.iter().max().unwrap_or(&0);
    let mut amount_to_shift = 1;
    for i in (0..*biggest_row).rev() {
        let row = &mut placed_pieces.0[i];
        if rows_to_remove.contains(&i) {
            amount_to_shift += 1;
            continue;
        }
        for entity in row.iter() {
            let (_, mut transform) = if let Ok(it) = child_query.get_mut(*entity) {
                it
            } else {
                continue;
            };
            transform.translation.y -= SQUARE_SIZE * amount_to_shift as f32;
        }
        placed_pieces.0[i + amount_to_shift] = row.clone();
        let row = &mut placed_pieces.0[i];
        row.clear();
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
    move_piece_to_board(&piece_type, &mut transform.translation);

    let new_piece = get_random_piece();
    let entities = build_piece(
        &mut commands,
        new_piece,
        Vec3::new(RIGHT_GRID * 1.5, 90.0, -1.0),
    );
    next_pieces.0.push(entities[0]);
}

fn move_piece_to_board(piece_type: &PieceType, translation: &mut Vec3) {
    match piece_type {
        PieceType::Square => {
            *translation = Vec3::new(-SQUARE_SIZE / 2.0, 240.0 + SQUARE_SIZE / 2., -1.0);
        }
        PieceType::Straight => {
            *translation = Vec3::new(SQUARE_SIZE / 2.0, 240.0 + SQUARE_SIZE / 2., -1.0);
        }
        _ => {
            *translation = Vec3::new(0.0, 240.0, -1.0);
        }
    }
}

fn hold_piece(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(&Children, Entity, &mut Transform), (With<Active>, Without<Hold>)>,
    mut held_query: Query<(&Children, &PieceType, Entity, &mut Transform), With<Hold>>,
    mut next_piece_query: Query<
        (&Children, &mut Transform, &PieceType),
        (With<Children>, Without<Hold>, Without<Active>),
    >,
    mut next_pieces: ResMut<NextPieces>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
    // TODO: turn into component
    mut can_hold: Local<CanHoldPiece>,
) {
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

    let (children, entity, mut transform) = query.get_single_mut().unwrap();

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
        move_piece_to_board(&piece_type, &mut transform.translation)
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
        move_piece_to_board(&piece_type, &mut next_transform.translation);

        let new_piece = get_random_piece();
        let entities = build_piece(&mut commands, new_piece, Vec3::new(0.0, 210.0, -1.0));
        next_pieces.0.push(entities[0]);
    }
}

fn get_row(translation_y: f32) -> usize {
    (((TOP_GRID - SQUARE_SIZE) - translation_y) / SQUARE_SIZE) as usize
}

fn move_to_hold(translation: &mut Vec3) {
    *translation = Vec3::new(LEFT_GRID - SQUARE_SIZE * 4.0, 210.0, -1.0);
}

fn position_next_pieces(next_pieces: ResMut<NextPieces>, mut query: Query<&mut Transform>) {
    for (i, &piece) in next_pieces.0.iter().enumerate() {
        let translation = Vec3::new(RIGHT_GRID * 1.5, 210.0 - i as f32 * SQUARE_SIZE * 3., -1.0);
        let mut transform = query.get_mut(piece).unwrap();
        transform.translation = translation;
    }
}
#[derive(Resource)]
struct ActiveTimer(Timer);

#[derive(Resource)]
struct NextPieces(Vec<Entity>);

struct CanHoldPiece(bool);

impl Default for CanHoldPiece {
    fn default() -> Self {
        CanHoldPiece(true)
    }
}

#[derive(Resource, Debug)]
struct PlacedPieces(Vec<Vec<Entity>>);

#[derive(Resource)]
struct PlaceGracePeriod(Timer);

fn main() {
    App::new()
        .insert_resource(ActiveTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(MovementTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .insert_resource(PlacedPieces(Vec::new()))
        .insert_resource(NextPieces(Vec::new()))
        .insert_resource(PlaceGracePeriod(Timer::from_seconds(0.25, TimerMode::Once)))
        .add_event::<PiecePlacedEvent>()
        .add_event::<CollisionEvent>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((
                shift_active_down,
                user_move_actives,
                user_rotate_active,
                check_collision,
                check_in_bounds,
                place_piece,
                position_next_pieces,
                hold_piece,
            )
                .chain(),),
        )
        .run();
}
