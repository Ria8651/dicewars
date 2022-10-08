use bevy::prelude::*;
use bevy_mod_picking::*;
use board::{Board, Node};

mod board;
mod fps_counter;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ui::UiPlugin)
        .add_plugin(board::BoardPlugin)
        .add_plugin(fps_counter::FpsCounter)
        .add_plugins(DefaultPickingPlugins)
        .add_event::<GameStateEvent>()
        .insert_resource(SelectionState { current: None })
        .add_startup_system(setup)
        .add_system(process_game)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert_bundle(PickingCameraBundle::default());
}

struct SelectionState {
    current: Option<(usize, Entity)>,
}

pub enum GameStateEvent {
    FinishTurn,
}

fn process_game(
    mut events: EventReader<PickingEvent>,
    mut node_entitys: Query<(&mut Node, Entity)>,
    mut selection_state: ResMut<SelectionState>,
    mut board: ResMut<Board>,
    mut game_state_events: EventReader<GameStateEvent>,
) {
    let mut to_deselect = Vec::new();

    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            for (mut node, entity) in node_entitys.iter_mut() {
                if entity == *e {
                    match selection_state.current {
                        None => {
                            if board.owner(node.index) == board.current_player() {
                                selection_state.current = Some((node.index, entity));
                                node.selected = true;
                            }
                        }
                        Some((first, first_entity)) => {
                            if first == node.index {
                                selection_state.current = None;
                                to_deselect.push(first_entity);
                                continue;
                            }

                            let second = node.index;
                            if board.available_moves(first).contains(&second) {
                                board.make_move(first, second);

                                selection_state.current = None;
                                to_deselect.push(first_entity);
                            }
                        }
                    }
                }
            }
        }
        // if let PickingEvent::Hover(e) = event {
        //     let entity = match e {
        //         HoverEvent::JustEntered(e) => e,
        //         HoverEvent::JustLeft(e) => e,
        //     };
        //     for (mut node, node_entity) in node_entitys.iter_mut() {
        //         if node_entity == *entity {
        //             match e {
        //                 HoverEvent::JustEntered(_) => {
        //                     node.hovered = true;
        //                 }
        //                 HoverEvent::JustLeft(_) => {
        //                     node.hovered = false;
        //                 }
        //             }
        //         }
        //     }
        // }
        for (mut node, _) in node_entitys.iter_mut() {
            node.hovered = false;
        }
        if let Some((first, first_entity)) = selection_state.current {
            for (mut node, entity) in node_entitys.iter_mut() {
                if entity == first_entity {
                    continue;
                }
                if board.available_moves(first).contains(&node.index) {
                    node.hovered = true;
                }
            }
        }
    }

    for (mut node, node_entity) in node_entitys.iter_mut() {
        if to_deselect.contains(&node_entity) {
            node.selected = false;
        }
    }

    for game_state_event in game_state_events.iter() {
        match game_state_event {
            GameStateEvent::FinishTurn => {
                board.finish_turn();
            }
        }
    }
}
