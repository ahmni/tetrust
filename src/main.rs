mod piece;
mod user_actions;
mod wall;

use bevy::prelude::*;
use piece::*;
use user_actions::*;
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

    let starting_piece: PieceType = PieceType::ReverseL;

    build_piece(&mut commands, starting_piece, Vec3::new(0.0, 210.0, -1.0));
}

#[derive(Event)]
struct PiecePlacedEvent(Entity);

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
        build_piece(&mut commands, new_piece, Vec3::new(0.0, 210.0, -1.0));
    }
}

// Collider system:
// - Check if any active piece is colliding with left or right wall
// - If so, push peace back inside the game
// - Also check if any active piece is colliding with bottom wall
// - If so, deactivate the piece and spawn a new one. Store old piece y position

#[derive(Resource)]
struct ActiveTimer(Timer);

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
