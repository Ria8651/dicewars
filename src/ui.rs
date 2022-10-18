use super::{
    board::{Board, BoardGenSettings},
    board_renderer::{BoardRenderData, RegenerateBoardEvent},
    GameStateEvent,
};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Slider},
    EguiContext, EguiPlugin,
};
use egui::{Color32, RichText};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(ui_system);
    }
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    board: Res<Board>,
    board_render_data: Res<BoardRenderData>,
    mut game_state_events: EventWriter<GameStateEvent>,
    mut regenerate_board_event: EventWriter<RegenerateBoardEvent>,
    mut board_gen_settings: ResMut<BoardGenSettings>,
) {
    egui::Window::new("Game menu").show(egui_context.ctx_mut(), |ui| {
        if ui.button("New game").clicked() {
            regenerate_board_event.send(RegenerateBoardEvent);
        }

        if ui.button("Finish turn").clicked() {
            game_state_events.send(GameStateEvent::FinishTurn);
        }

        ui.horizontal(|ui| {
            let (turn, scores) = board.scores();
            for i in 0..scores.len() {
                let colour = board_render_data.colours[scores[i].0];
                ui.label(
                    RichText::new(if i == turn {
                        format!("({:?})", scores[i].1)
                    } else {
                        format!("{:?}", scores[i].1)
                    })
                    .color(Color32::from_rgb(
                        (colour.r() * 255.0) as u8,
                        (colour.g() * 255.0) as u8,
                        (colour.b() * 255.0) as u8,
                    )),
                );
            }
        });

        ui.add(Slider::new(&mut board_gen_settings.player_count, 1..=8).text("Players"));
        ui.add(Slider::new(&mut board_gen_settings.board_size, 7..=50).text("Board size"));
    });
}
