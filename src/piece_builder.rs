use bevy::{prelude::*, sprite::Anchor};

use rand::Rng;

use crate::{DespawnOnRestart, RotationState};

pub const SQUARE_SIZE: f32 = 30.0;

#[derive(Component, Clone)]
pub struct Active;

#[derive(Component, Clone)]
pub struct Ghost;

#[derive(Bundle)]
pub struct PieceBundle {
    spatial_bundle: SpatialBundle,
    type_bundle: PieceType,
    despawn: DespawnOnRestart,
    rotation_state: RotationState,
}

#[derive(Component, Clone, Eq, PartialEq, Debug)]
pub enum PieceType {
    Straight,
    L,
    ReverseL,
    T,
    Z,
    ReverseZ,
    Square,
    Ghost(Box<PieceType>),
}

impl PieceType {
    fn sprite(&self) -> Sprite {
        let color = match self {
            PieceType::Straight => Color::rgb_u8(12, 175, 200),
            PieceType::L => Color::rgb_u8(218, 118, 31),
            PieceType::ReverseL => Color::rgb_u8(31, 111, 235),
            PieceType::T => Color::rgb_u8(178, 23, 163),
            PieceType::Z => Color::rgb_u8(51, 134, 24),
            PieceType::ReverseZ => Color::rgb_u8(195, 17, 40),
            PieceType::Square => Color::rgb_u8(205, 180, 2),
            PieceType::Ghost(_) => Color::rgba(0.0, 0.0, 0.0, 0.5),
        };

        Sprite {
            color,
            rect: Some(Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE)),
            anchor: Anchor::BottomLeft,
            ..default()
        }
    }
}

impl PieceBundle {
    pub fn new(piece_type: &PieceType, pos: &Vec3) -> PieceBundle {
        PieceBundle {
            spatial_bundle: SpatialBundle {
                transform: Transform::from_xyz(pos.x, pos.y, pos.z),
                visibility: Visibility::Visible,
                ..default()
            },
            type_bundle: piece_type.clone(),
            despawn: DespawnOnRestart,
            rotation_state: default(),
        }
    }
}

#[derive(Bundle)]
pub struct PiecePartBundle {
    sprite_bundle: SpriteBundle,
    despawn: DespawnOnRestart,
}

impl PiecePartBundle {
    pub fn new(piece_type: &PieceType, pos: &Vec3) -> PiecePartBundle {
        PiecePartBundle {
            sprite_bundle: SpriteBundle {
                sprite: piece_type.sprite(),
                transform: Transform::from_xyz(pos.x, pos.y, pos.z),
                ..default()
            },
            despawn: DespawnOnRestart,
        }
    }
}

pub fn get_random_piece() -> PieceType {
    match rand::thread_rng().gen_range(0..7) {
        0 => PieceType::Straight,
        1 => PieceType::L,
        2 => PieceType::ReverseL,
        3 => PieceType::T,
        4 => PieceType::Z,
        5 => PieceType::ReverseZ,
        6 => PieceType::Square,
        _ => panic!("Invalid random number"),
    }
}

pub fn build_active_piece(commands: &mut Commands, piece_type: &PieceType, pos: Vec3) {
    internal_build_piece(commands, piece_type, pos, Some(Active), false);
}

pub fn build_piece(commands: &mut Commands, piece_type: &PieceType, pos: Vec3) -> [Entity; 5] {
    return internal_build_piece(commands, piece_type, pos, None, false);
}

fn internal_build_piece(
    commands: &mut Commands,
    piece_type: &PieceType,
    pos: Vec3,
    active: Option<Active>,
    is_ghost: bool,
) -> [Entity; 5] {
    match piece_type {
        PieceType::Straight => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            // Needed to change origin pivot point when rotating
            // https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
            let straight_pos = Vec3::new(pos.x + SQUARE_SIZE / 2., pos.y + SQUARE_SIZE / 2., pos.z);

            let piece = PieceBundle::new(&piece_type, &straight_pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(SQUARE_SIZE / 2., SQUARE_SIZE / 2., 0.0);

            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x + SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x - SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        relative_pos.x - SQUARE_SIZE * 2.0,
                        relative_pos.y,
                        relative_pos.z,
                    ),
                ))
                .id();

            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );

            return [piece, child1, child2, child3, child4];
        }
        PieceType::ReverseL => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            let piece = PieceBundle::new(&piece_type, &pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(0., 0., 0.0);
            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x + SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x - SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        relative_pos.x - SQUARE_SIZE,
                        relative_pos.y + SQUARE_SIZE,
                        relative_pos.z,
                    ),
                ))
                .id();
            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );
            return [piece, child1, child2, child3, child4];
        }
        PieceType::L => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            let piece = PieceBundle::new(&piece_type, &pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(0., 0., 0.0);
            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x + SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x - SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        relative_pos.x + SQUARE_SIZE,
                        relative_pos.y + SQUARE_SIZE,
                        relative_pos.z,
                    ),
                ))
                .id();
            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );
            return [piece, child1, child2, child3, child4];
        }
        PieceType::Z => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            let piece = PieceBundle::new(&piece_type, &pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(0., 0., 0.0);
            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x, relative_pos.y + SQUARE_SIZE, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        relative_pos.x + SQUARE_SIZE,
                        relative_pos.y + SQUARE_SIZE,
                        relative_pos.z,
                    ),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x - SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );
            return [piece, child1, child2, child3, child4];
        }
        PieceType::ReverseZ => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            let piece = PieceBundle::new(&piece_type, &pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(0., 0., 0.0);
            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x, relative_pos.y + SQUARE_SIZE, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        relative_pos.x - SQUARE_SIZE,
                        relative_pos.y + SQUARE_SIZE,
                        relative_pos.z,
                    ),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x + SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );
            return [piece, child1, child2, child3, child4];
        }
        PieceType::T => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            let piece = PieceBundle::new(&piece_type, &pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(0., 0., 0.0);
            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x, relative_pos.y + SQUARE_SIZE, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x + SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x - SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );
            return [piece, child1, child2, child3, child4];
        }
        PieceType::Square => {
            let piece_type = if is_ghost {
                &PieceType::Ghost(Box::new(piece_type.clone()))
            } else {
                piece_type
            };

            // Needed to change origin pivot point when rotating
            // https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
            // Sidenote: the reason why we have to subtract the offset isntead of adding is because
            // we are "moving" the square left and down relative to (0,0) so that the pivot point is in the center

            // Deeper Explanation:
            // The center of the bottom left block is the original pivot point ((0,0) as its relative position) of as the rest of the blocks are
            // built relative to the bottom left block. The pivot point should be in the center of the square,
            // so we need to offset the originl pivot point by the center of the botom-left block
            // so that the relative position of the bottom left block is (-15,-15), and the center of the square is at (0,0).
            let square_pos = Vec3::new(pos.x - SQUARE_SIZE / 2., pos.y - SQUARE_SIZE / 2., pos.z);
            let piece = PieceBundle::new(&piece_type, &square_pos);
            let piece = commands.spawn(piece).id();

            let relative_pos = Vec3::new(-SQUARE_SIZE / 2., -SQUARE_SIZE / 2., 0.0);
            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &relative_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x + SQUARE_SIZE, relative_pos.y, relative_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(relative_pos.x, relative_pos.y + SQUARE_SIZE, relative_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        relative_pos.x + SQUARE_SIZE,
                        relative_pos.y + SQUARE_SIZE,
                        relative_pos.z,
                    ),
                ))
                .id();
            combine_piece_parts(
                commands,
                piece,
                active,
                vec![child1, child2, child3, child4],
            );
            return [piece, child1, child2, child3, child4];
        }
        PieceType::Ghost(piece_type) => {
            return internal_build_piece(commands, piece_type, pos, None, true);
        }
    }
}

fn combine_piece_parts(
    commands: &mut Commands,
    piece: Entity,
    active: Option<Active>,
    children: Vec<Entity>,
) {
    if let Some(active) = active {
        for entity in [piece].iter().chain(&children) {
            commands.entity(*entity).insert(active.clone());
        }
    }
    commands.entity(piece).push_children(&children);
}
