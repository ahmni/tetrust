use bevy::prelude::*;

use crate::{toggle_menu, GameMusic, GameState, RestartGameEvent, TitleMenu};

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct PauseButton;

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Event)]
pub struct PauseGameEvent;

pub fn pause_button(
    pause_button: Query<&Interaction, (Changed<Interaction>, With<PauseButton>)>,
    mut ev_pause: EventWriter<PauseGameEvent>,
) {
    for interaction in &mut pause_button.iter() {
        if *interaction == Interaction::Pressed {
            println!("Pause button pressed");
            ev_pause.send(PauseGameEvent);
        }
    }
}

#[derive(Component)]
pub struct RestartButton;

pub fn restart_button(
    restart_button: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut ev_restart: EventWriter<RestartGameEvent>,
) {
    for interaction in &mut restart_button.iter() {
        if *interaction == Interaction::Pressed {
            ev_restart.send(RestartGameEvent);
        }
    }
}

#[derive(Component)]
pub struct TitleButton;

pub fn title_button(
    title_button: Query<&Interaction, (Changed<Interaction>, With<TitleButton>)>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    music_controller: Query<&AudioSink, With<GameMusic>>,
    mut query: Query<&mut Visibility, With<TitleMenu>>,
) {
    for interaction in &mut title_button.iter() {
        if *interaction == Interaction::Pressed {
            toggle_menu(
                game_state,
                &mut next_state,
                music_controller,
                &mut query.single_mut(),
            );

            return;
        }
    }
}
