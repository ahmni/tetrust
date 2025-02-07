use std::time::Duration;

use bevy::prelude::*;

use crate::{get_row, Active, DropTimer, PiecePlacedEvent};

#[derive(Component)]
pub struct Score(pub u32);

#[derive(Component)]
pub struct Level(pub u32);

#[derive(Event)]
pub struct ClearEvent(pub Vec<usize>);

#[derive(Event)]
pub struct LevelUpEvent;

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
    piece_query: Query<&Children, With<Active>>,
    child_query: Query<&GlobalTransform, With<Active>>,
    level: Query<&Level>,
    mut score: Query<(&mut Score, &mut Text)>,
    mut ev_clear: EventReader<ClearEvent>,
    mut ev_piece_placed: EventReader<PiecePlacedEvent>,
    mut ev_restart: EventReader<crate::RestartGameEvent>,
) {
    if ev_restart.read().next().is_some() {
        for (mut score, mut text) in score.iter_mut() {
            score.0 = 0;
            text.sections[0].value = score.0.to_string();
        }
        return;
    }

    let mut score_to_add = 0;
    for ev in ev_piece_placed.read() {
        let children = if let Ok(some) = piece_query.get(ev.0) {
            some
        } else {
            return;
        };
        let min_row = children
            .iter()
            .map(|&child| {
                let child_global_transform = if let Ok(some) = child_query.get(child) {
                    some
                } else {
                    return 0;
                };
                let child_global_translation = child_global_transform.translation();
                get_row(child_global_translation.y)
            })
            .min()
            .unwrap_or(0);

        let min_row = if min_row > 0 { min_row - 1 } else { min_row };

        score_to_add += 1 * min_row as u32;
    }

    let level = level.iter().next().unwrap();
    for ev in ev_clear.read() {
        score_to_add += get_score(ev.0.len() as u32) * level.0;
    }

    for (mut score, mut text) in score.iter_mut() {
        score.0 += score_to_add;
        text.sections[0].value = score.0.to_string();
    }
}

pub fn level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut drop_timer: ResMut<DropTimer>,
    mut level_query: Query<(&mut Level, &mut Text)>,
    mut ev_clear: EventReader<ClearEvent>,
    mut lines_cleared: Local<u32>,
    mut ev_level_up: EventWriter<LevelUpEvent>,
    mut ev_restart: EventReader<crate::RestartGameEvent>,
) {
    if ev_restart.read().next().is_some() {
        for (mut level, mut text) in level_query.iter_mut() {
            level.0 = 0;
            text.sections[0].value = level.0.to_string();
        }
        return;
    }

    let should_increase_level = keyboard_input.just_pressed(KeyCode::KeyL);
    for ev in ev_clear.read() {
        *lines_cleared += ev.0.len() as u32;
    }
    if *lines_cleared < 10 && !should_increase_level {
        return;
    }

    *lines_cleared = *lines_cleared % 10;
    // increase level

    let mut cur_level = 0;
    for (mut level, mut text) in level_query.iter_mut() {
        level.0 += 1;
        text.sections[0].value = level.0.to_string();
        cur_level = level.0;
    }
    ev_level_up.send(LevelUpEvent);

    // increase game speed
    let prev_duration = drop_timer.0.duration();
    if cur_level <= 10 {
        let new_duration = prev_duration - Duration::from_millis(60);
        drop_timer.0.set_duration(new_duration);
        return;
    }

    if cur_level == 13 || cur_level == 16 || cur_level == 19 || cur_level == 29 {
        let new_duration = prev_duration - Duration::from_millis(60);
        drop_timer.0.set_duration(new_duration);
    }
}
