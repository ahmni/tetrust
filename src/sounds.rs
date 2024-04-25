use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
    window::WindowFocused,
};

use crate::{
    ClearEvent, GameOverEvent, HoldPieceEvent, LevelUpEvent, MoveEvent, PiecePlacedEvent,
    RotateEvent,
};

#[derive(Component)]
pub struct GameMusic;

pub fn sound_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/game_music.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.5),
                ..default()
            },
        },
        GameMusic,
    ));

    //commands.spawn((
    //    AudioBundle {
    //        source: asset_server.load("sounds/game_over.mp3"),
    //        settings: PlaybackSettings {
    //            mode: PlaybackMode::Despawn,
    //            ..default()
    //        },
    //    },
    //    SoundEffect(SoundEffectType::GameOver),
    //));
}

// TODO: pause on game over and pause
pub fn pause_music(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    mut ev_window: EventReader<WindowFocused>,
) {
    for event in ev_window.read() {
        if event.focused {
            if let Ok(sink) = music_controller.get_single() {
                sink.play();
            }
        } else {
            if let Ok(sink) = music_controller.get_single() {
                sink.pause();
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyM) {
        internal_pause_music(music_controller);
    }
}

pub fn internal_pause_music(music_controller: Query<&AudioSink, With<GameMusic>>) {
    if let Ok(sink) = music_controller.get_single() {
        sink.toggle();
    }
}

pub fn sound_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_placed: EventReader<PiecePlacedEvent>,
    mut ev_clear: EventReader<ClearEvent>,
    mut ev_rotate: EventReader<RotateEvent>,
    mut ev_move: EventReader<MoveEvent>,
    mut ev_game_over: EventReader<GameOverEvent>,
    mut ev_level_up: EventReader<LevelUpEvent>,
    mut ev_hold: EventReader<HoldPieceEvent>,
) {
    for _ in ev_placed.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/drop.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }

    for ev in ev_clear.read() {
        println!("lines cleared: {:?}", ev.0);
        if ev.0 == 4 {
            commands.spawn(AudioBundle {
                source: asset_server.load("sounds/tetris.mp3"),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            });
        } else {
            commands.spawn(AudioBundle {
                source: asset_server.load("sounds/line_clear.mp3"),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            });
        }
    }

    for _ in ev_rotate.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/rotate.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }

    for _ in ev_move.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/move.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }

    for _ in ev_game_over.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/game_over.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }

    for _ in ev_level_up.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/level_up.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }

    for _ in ev_hold.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/hold.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }
}
