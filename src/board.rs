use super::{
    board_renderer::{Tile, SCALE},
    hex::Hex,
};
use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Board {
    pub turn: usize,
    pub player_order: Vec<usize>,
    pub territories: Vec<Territory>,
}

#[derive(Debug)]
pub struct Territory {
    pub owner: usize,
    pub dice: u32,
    connections: Vec<usize>,
}

pub struct BoardGenSettings {
    pub player_count: usize,
    pub board_size: usize,
}

impl Board {
    pub fn generate(
        board_gen_settings: &BoardGenSettings,
    ) -> (Self, Vec<(Transform, Tile, Vec<Transform>)>, Vec<Vec2>) {
        let mut rng = rand::thread_rng();
        let mut map = HashMap::new();
        let mut territories = Vec::new();
        let mut territory_tiles = Vec::new();
        let mut positions = Vec::new();

        // helper function
        fn generate_options(
            i: usize,
            territory_tiles: &mut Vec<Vec<Hex>>,
            map: &mut HashMap<Hex, usize>,
            board_gen_settings: &BoardGenSettings,
        ) -> Vec<Hex> {
            let mut options = Vec::new();
            let outer;
            if i == usize::MAX {
                outer = territory_tiles.iter();
            } else {
                outer = territory_tiles[i..=i].iter();
            }

            for territory in outer {
                for tile in territory.iter() {
                    for neighbor in Hex::orthogonal() {
                        let neighbor = *tile + neighbor;
                        if !map.contains_key(&neighbor)
                            && neighbor.distance(Hex::ZERO) < board_gen_settings.board_size as i32
                        {
                            options.push(neighbor);
                        }
                    }
                }
            }
            options
        }

        // main terrain gen loop
        let num_territories = 25;
        let territory_size = 10;
        let mut i = 0;
        'outer: for _ in 0..num_territories {
            // create new territory
            let options = if i == 0 {
                vec![Hex::new(0, 0, 0)]
            } else {
                generate_options(
                    usize::MAX,
                    &mut territory_tiles,
                    &mut map,
                    board_gen_settings,
                )
            };

            if let Some(tile) = options.choose(&mut rng) {
                territories.push(Territory {
                    owner: 0,
                    dice: 1,
                    connections: Vec::new(),
                });
                territory_tiles.push(vec![*tile]);
                map.insert(*tile, i);
            } else {
                break;
            }

            // expand territory one tile at a time
            for _ in 0..territory_size {
                let options =
                    generate_options(i, &mut territory_tiles, &mut map, board_gen_settings);
                if let Some(tile) = options.choose(&mut rng) {
                    territory_tiles[i].push(*tile);
                    map.insert(*tile, i);
                } else {
                    // discard if territory can no longer expand
                    while let Some(tile) = territory_tiles[i].pop() {
                        map.remove(&tile);
                    }
                    let _ = territories.pop();
                    let _ = territory_tiles.pop();

                    continue 'outer;
                }
            }

            // expand whole territory to make it smoother
            let options = generate_options(i, &mut territory_tiles, &mut map, board_gen_settings);
            for tile in options {
                territory_tiles[i].push(tile);
                map.insert(tile, i);
            }

            i += 1;
        }

        // generate connections
        for (hex, territory) in map.iter() {
            for direction in Hex::orthogonal() {
                let neighbor = *hex + direction;
                if map.contains_key(&neighbor) {
                    let territory = territories.get_mut(*territory).unwrap();
                    let neighbor = map.get(&neighbor).unwrap();
                    if !territory.connections.contains(neighbor) {
                        territory.connections.push(*neighbor);
                    }
                }
            }
        }

        // generate positions
        for territory in territory_tiles.iter() {
            let mut position = Vec2::new(0.0, 0.0);
            for tile in territory {
                position += tile.to_grid() * SCALE;
            }
            positions.push(position / (territory.len() as f32));
        }

        // distribue territoryes between players
        let mut territorys_left = (0..territories.len()).collect::<Vec<_>>();
        let mut player_territorys = vec![Vec::new(); board_gen_settings.player_count];
        'outer2: loop {
            for i in 0..board_gen_settings.player_count {
                let index = rng.gen_range(0..territorys_left.len());
                let territory = territorys_left.remove(index);
                territories[territory].owner = i;
                player_territorys[i].push(territory);

                if territorys_left.is_empty() {
                    break 'outer2;
                }
            }
        }

        // distribute dice to territories
        let average_dice_per_territory = 3;
        let total_dice = (average_dice_per_territory - 1) * territories.len();
        let dice_per_player = total_dice / board_gen_settings.player_count;

        for i in 0..board_gen_settings.player_count {
            let mut dice_left = dice_per_player;
            while dice_left > 0 {
                let index = rng.gen_range(0..player_territorys[i].len());
                let territory = player_territorys[i][index];
                let territory = territories.get_mut(territory).unwrap();
                if territory.dice < 8 {
                    territory.dice += 1;
                    dice_left -= 1;
                }
            }
        }

        // generate render data
        let mut tiles = Vec::new();
        for (hex, territory_index) in map.iter() {
            // add node data
            let transform = Transform::from_translation((hex.to_grid() * SCALE).extend(0.0))
                .with_scale(SCALE.extend(1.0));

            // add edge data
            let mut edges = Vec::new();
            for direction in Hex::orthogonal() {
                let neighbor = *hex + direction;
                let neighbor_index = map.get(&neighbor).unwrap_or(&usize::MAX);
                if territory_index != neighbor_index {
                    let center = hex.to_grid();
                    let neighbor = neighbor.to_grid();
                    let transform = Transform::from_xyz(0.0, 0.0, 0.5)
                        .looking_at(-Vec3::Z, (neighbor - center).extend(0.0));
                    edges.push(transform);
                }
            }

            tiles.push((
                transform,
                Tile {
                    index: *territory_index,
                },
                edges,
            ));
        }

        // random player order
        let mut player_order = (0..board_gen_settings.player_count).collect::<Vec<_>>();
        player_order.shuffle(&mut rng);

        let board = Self {
            turn: 0,
            player_order,
            territories,
        };

        (board, tiles, positions)
    }

    /// note: this funciton assumes that the move is valid, [Board::available_moves] should be used to check
    /// if the move is valid first.
    pub fn make_move(&mut self, first: usize, second: usize) {
        let mut rng = rand::thread_rng();

        // roll the dice!
        let mut first_total = 0;
        for _ in 0..self.territories[first].dice {
            first_total += rng.gen_range(1..6);
        }

        let mut second_total = 0;
        for _ in 0..self.territories[second].dice {
            second_total += rng.gen_range(1..6);
        }

        if first_total > second_total {
            println!("Attack!! {} vs {}: win!", first_total, second_total);
            self.territories[second].owner = self.territories[first].owner;
            self.territories[second].dice = self.territories[first].dice - 1;
            self.territories[first].dice = 1;
        } else {
            println!("Attack!! {} vs {}: loss...", first_total, second_total);
            self.territories[first].dice = 1;
        }
    }

    pub fn available_moves(&self, first: usize) -> Vec<usize> {
        let mut moves = Vec::new();
        if self.territories[first].owner == self.player_order[self.turn]
            && self.territories[first].dice > 1
        {
            for second in self.territories[first].connections.iter() {
                if self.territories[first].owner != self.territories[*second].owner {
                    moves.push(*second);
                }
            }
        }
        moves
    }

    fn count_territories(&self) -> Vec<u32> {
        let mut income = vec![0; 8];
        for territory in self.territories.iter() {
            income[territory.owner] += 1;
        }
        income
    }

    pub fn finish_turn(&mut self) {
        let mut rng = rand::thread_rng();
        let scores = self.scores().1;
        let (player, score) = scores[self.turn];

        let mut player_teritories = Vec::new();
        for territory in self.territories.iter_mut() {
            if territory.owner == player && territory.dice < 8 {
                player_teritories.push(territory);
            }
        }
        for _ in 0..score {
            loop {
                if player_teritories.len() > 0 {
                    let index = rng.gen_range(0..player_teritories.len());
                    let territory = &mut player_teritories[index];
                    if territory.dice < 8 {
                        territory.dice += 1;
                        break;
                    } else {
                        player_teritories.remove(index);
                    }
                } else {
                    // add extra dice to player's bonus
                    break;
                }
            }
        }

        let territory_counts = self.count_territories();
        for i in 0..territory_counts.len() {
            if territory_counts[i] == 0 {
                for j in 0..self.player_order.len() {
                    if self.player_order[j] == i {
                        println!("Player {} has lost!", i);
                        self.player_order.remove(j);
                        if j <= self.turn {
                            self.turn -= 1;
                        }
                        break;
                    }
                }
            }
        }

        self.turn += 1;
        if self.turn >= self.player_order.len() {
            self.turn = 0;
        }
    }

    pub fn owner(&self, territory: usize) -> usize {
        self.territories[territory].owner
    }

    pub fn current_player(&self) -> usize {
        self.player_order[self.turn]
    }

    pub fn scores(&self) -> (usize, Vec<(usize, u32)>) {
        // for now the score is the number of territories, i cant figure out the actual rules
        let counts = self.count_territories();
        let mut scores = Vec::new();
        for player in self.player_order.iter() {
            scores.push((*player, counts[*player]));
        }
        (self.turn, scores)
    }
}
