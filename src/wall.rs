use bevy::prelude::*;

pub const BOTTOM_GRID: f32 = -300.0;
pub const LEFT_GRID: f32 = -150.0;
pub const RIGHT_GRID: f32 = 150.0;
pub const TOP_GRID: f32 = 300.0;

pub const WALL_THICKNESS: f32 = 10.0;

pub const GRID_LINE_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
pub const GRID_LINE_THICKNESS: f32 = 2.0;

const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

#[derive(Bundle)]
pub struct WallBundle {
    sprite_bundle: SpriteBundle,
}

pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    /// Location of the *center* of the wall, used in `transform.translation()`
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_GRID - GRID_LINE_THICKNESS * 2.0, 0.),
            WallLocation::Right => Vec2::new(RIGHT_GRID + GRID_LINE_THICKNESS * 2.0, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_GRID - GRID_LINE_THICKNESS * 2.0),
            WallLocation::Top => Vec2::new(0., TOP_GRID + GRID_LINE_THICKNESS * 2.0),
        }
    }
    /// (x, y) dimensions of the wall, used in `transform.scale()`
    fn size(&self) -> Vec2 {
        let arena_height = TOP_GRID - BOTTOM_GRID;
        let arena_width = RIGHT_GRID - LEFT_GRID;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    pub fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
        }
    }
}
