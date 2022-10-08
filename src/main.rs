use bevy::prelude::*;
use bevy_mod_picking::*;
use board::{Board, Node};
use rand::Rng;

mod board;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ui::UiPlugin)
        .add_plugin(board::BoardPlugin)
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(SelectionState { current: None })
        .add_startup_system(setup)
        .add_system(print_events)
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

fn print_events(
    mut events: EventReader<PickingEvent>,
    mut node_entitys: Query<(&mut Node, Entity)>,
    mut selection_state: ResMut<SelectionState>,
    mut board: ResMut<Board>,
) {
    let mut to_deselect = Vec::new();

    let mut rng = rand::thread_rng();

    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            for (mut node, entity) in node_entitys.iter_mut() {
                if entity == *e {
                    match selection_state.current {
                        None => {
                            selection_state.current = Some((node.index, entity));
                            node.selected = true;
                        }
                        Some((first, first_entity)) => {
                            if first == node.index {
                                selection_state.current = None;
                                to_deselect.push(first_entity);
                            }

                            let second = node.index;

                            if board.teritories[first].dice > 1 {
                                if board.teritories[first].connections.contains(&second)
                                    && board.teritories[first].owner
                                        != board.teritories[second].owner
                                {
                                    // roll the dice!
                                    let mut first_total = 0;
                                    for _ in 0..board.teritories[first].dice {
                                        first_total += rng.gen_range(1..6);
                                    }

                                    let mut second_total = 0;
                                    for _ in 0..board.teritories[second].dice {
                                        second_total += rng.gen_range(1..6);
                                    }

                                    if first_total > second_total {
                                        println!(
                                            "Attack!! {} vs {}: win!",
                                            first_total, second_total
                                        );
                                        board.teritories[second].owner =
                                            board.teritories[first].owner;
                                        board.teritories[second].dice =
                                            board.teritories[first].dice - 1;
                                        board.teritories[first].dice = 1;
                                    } else {
                                        println!(
                                            "Attack!! {} vs {}: loss...",
                                            first_total, second_total
                                        );
                                        board.teritories[first].dice = 1;
                                    }

                                    selection_state.current = None;
                                    to_deselect.push(first_entity);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let PickingEvent::Hover(e) = event {
            let entity = match e {
                HoverEvent::JustEntered(e) => e,
                HoverEvent::JustLeft(e) => e,
            };
            for (mut node, node_entity) in node_entitys.iter_mut() {
                if node_entity == *entity {
                    match e {
                        HoverEvent::JustEntered(_) => {
                            node.hovered = true;
                        }
                        HoverEvent::JustLeft(_) => {
                            node.hovered = false;
                        }
                    }
                }
            }
        }
    }

    for (mut node, node_entity) in node_entitys.iter_mut() {
        if to_deselect.contains(&node_entity) {
            node.selected = false;
        }
    }
}
