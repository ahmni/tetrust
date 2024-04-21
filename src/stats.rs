use std::time::Duration;

use bevy::prelude::*;

use crate::{get_row, Active, DropTimer, PiecePlacedEvent};

#[derive(Component)]
pub struct Score(pub u32);

#[derive(Component)]
pub struct Level(pub u32);

#[derive(Event)]
pub struct ClearEvent(pub u32);

fn get_score(rows_cleared: u32) -> u32 {
    match rows_cleared {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0, // never hapens
    }
}

pub fn score(
    mut score: Query<(&mut Score, &mut Text)>,
    piece_query: Query<&Children, With<Active>>,
    child_query: Query<&GlobalTransform, With<Active>>,
    level: Query<&Level>,
    mut ev_clear: EventReader<ClearEvent>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
) {
    let (mut score, mut text) = score.get_single_mut().unwrap();

    for ev in ev_piece_placed.read() {
        let children = piece_query.get(ev.0).unwrap();
        let min_row = children
            .iter()
            .map(|&child| {
                let child_global_transform = child_query.get(child).unwrap();
                let child_global_translation = child_global_transform.translation();
                get_row(child_global_translation.y)
            })
            .min()
            .unwrap()
            - 1;

        score.0 += 1 * min_row as u32;
    }

    let level = level.single();
    for ev in ev_clear.read() {
        score.0 += get_score(ev.0) * level.0;
    }
    text.sections[0].value = score.0.to_string();
}

pub fn level(
    mut drop_timer: ResMut<DropTimer>,
    mut level_query: Query<(&mut Level, &mut Text)>,
    mut ev_clear: EventReader<ClearEvent>,
    mut lines_cleared: Local<u32>,
) {
    for ev in ev_clear.read() {
        *lines_cleared += ev.0;
    }
    if *lines_cleared < 10 {
        return;
    }

    *lines_cleared = *lines_cleared - 10;

    let prev_duration = drop_timer.0.duration();
    drop_timer
        .0
        .set_duration(prev_duration - Duration::from_millis(25));

    // increase level
    let (mut level, mut text) = level_query.single_mut();

    level.0 += 1;
    text.sections[0].value = level.0.to_string();
}
