use bevy::{prelude::*, sprite::Anchor};

use crate::Collider;

use rand::Rng;

pub const SQUARE_SIZE: f32 = 30.0;

#[derive(Component)]
pub struct Active;

#[derive(Bundle)]
pub struct PieceBundle {
    spatial_bundle: SpatialBundle,
    collider: Collider,
}

pub enum PieceType {
    Straight,
    L,
    ReverseL,
    T,
    Z,
    ReverseZ,
    Square,
}

impl PieceType {
    fn sprite(&self) -> Sprite {
        let color = match self {
            PieceType::Straight => Color::rgb(0.0, 1.0, 1.0),
            PieceType::L => Color::rgb(1.0, 0.0, 1.0),
            PieceType::ReverseL => Color::rgb(1.0, 1.0, 0.0),
            PieceType::T => Color::rgb(0.0, 0.0, 1.0),
            PieceType::Z => Color::rgb(0.0, 1.0, 0.0),
            PieceType::ReverseZ => Color::rgb(1.0, 0.0, 0.0),
            PieceType::Square => Color::rgb(1.0, 1.0, 1.0),
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
    pub fn new(pos: &Vec3) -> PieceBundle {
        PieceBundle {
            spatial_bundle: SpatialBundle {
                transform: Transform::from_xyz(pos.x, pos.y, pos.z),
                visibility: Visibility::Visible,
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Bundle)]
pub struct PiecePartBundle {
    sprite_bundle: SpriteBundle,
}

impl PiecePartBundle {
    pub fn new(piece_type: &PieceType, pos: &Vec3) -> PiecePartBundle {
        PiecePartBundle {
            sprite_bundle: SpriteBundle {
                sprite: piece_type.sprite(),
                transform: Transform::from_xyz(pos.x, pos.y, pos.z),
                ..default()
            },
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

pub fn build_piece(commands: &mut Commands, piece_type: PieceType, pos: Vec3) {
    match piece_type {
        PieceType::Straight => {
            // Needed to change origin pivot point when rotating
            // https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
            let straight_pos = Vec3::new(pos.x + SQUARE_SIZE / 2., pos.y + SQUARE_SIZE / 2., pos.z);

            let piece = PieceBundle::new(&straight_pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &straight_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(straight_pos.x, straight_pos.y + SQUARE_SIZE, straight_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(straight_pos.x, straight_pos.y - SQUARE_SIZE, straight_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        straight_pos.x,
                        straight_pos.y - SQUARE_SIZE * 2.0,
                        straight_pos.z,
                    ),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
        PieceType::ReverseL => {
            let piece = PieceBundle::new(&pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands.spawn(PiecePartBundle::new(&piece_type, &pos)).id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x + SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x - SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x - SQUARE_SIZE, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
        PieceType::L => {
            let piece = PieceBundle::new(&pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands.spawn(PiecePartBundle::new(&piece_type, &pos)).id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x + SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x - SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x + SQUARE_SIZE, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
        PieceType::Z => {
            let piece = PieceBundle::new(&pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands.spawn(PiecePartBundle::new(&piece_type, &pos)).id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x + SQUARE_SIZE, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x - SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
        PieceType::ReverseZ => {
            let piece = PieceBundle::new(&pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands.spawn(PiecePartBundle::new(&piece_type, &pos)).id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x - SQUARE_SIZE, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x + SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
        PieceType::T => {
            let piece = PieceBundle::new(&pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands.spawn(PiecePartBundle::new(&piece_type, &pos)).id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x, pos.y + SQUARE_SIZE, pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x + SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(pos.x - SQUARE_SIZE, pos.y, pos.z),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
        PieceType::Square => {
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
            let piece = PieceBundle::new(&square_pos);
            let piece = commands.spawn((piece, Active)).id();

            let child1 = commands
                .spawn(PiecePartBundle::new(&piece_type, &square_pos))
                .id();
            let child2 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(square_pos.x + SQUARE_SIZE, square_pos.y, square_pos.z),
                ))
                .id();
            let child3 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(square_pos.x, square_pos.y + SQUARE_SIZE, square_pos.z),
                ))
                .id();
            let child4 = commands
                .spawn(PiecePartBundle::new(
                    &piece_type,
                    &Vec3::new(
                        square_pos.x + SQUARE_SIZE,
                        square_pos.y + SQUARE_SIZE,
                        square_pos.z,
                    ),
                ))
                .id();
            commands
                .entity(piece)
                .push_children(&[child1, child2, child3, child4]);
        }
    }
}
