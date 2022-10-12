use super::hex::Hex;
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    ui::{FocusPolicy, Interaction},
};
use bevy_mod_picking::*;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_obj::ObjPlugin)
            .add_startup_system(setup)
            .add_stage_after(CoreStage::Update, "Post", SystemStage::parallel())
            .add_system_to_stage("Post", update_board);
    }
}

#[derive(Debug)]
pub struct Board {
    turn: usize,
    player_order: Vec<usize>,
    territories: Vec<Territory>,
}

#[derive(Debug)]
struct Territory {
    owner: usize,
    dice: u32,
    connections: Vec<usize>,
}

impl Board {
    pub fn generate() -> (Self, Vec<(Transform, Tile)>, Vec<Transform>, Vec<Vec2>) {
        let mut rng = rand::thread_rng();
        let mut map = HashMap::new();
        let mut territories = Vec::new();
        let mut positions = Vec::new();

        // generate territories
        for i in 0..16 {
            territories.push(Territory {
                owner: rng.gen_range(0..2),
                dice: rng.gen_range(1..5),
                connections: Vec::new(),
            });

            let q = rng.gen_range(-10..10);
            let r = rng.gen_range(-10..10);
            let hex = Hex::new(q, r, -q - r);
            map.insert(hex, i);
            positions.push(hex.to_grid() * 10.0);
        }

        // spread territories
        for _ in 0..4 {
            for (hex, territory) in map.clone().iter() {
                for direction in Hex::orthogonal() {
                    let neighbor = *hex + direction;
                    if !map.contains_key(&neighbor) {
                        map.insert(neighbor, *territory);
                    }
                }
            }
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

        // generate render data
        let mut tiles = Vec::new();
        let mut edges = Vec::new();
        for (hex, territory_index) in map.iter() {
            // add node data
            let transform = Transform::from_translation(hex.to_grid().extend(0.0) * 10.0)
                .with_scale(Vec3::splat(10.0));
            tiles.push((
                transform,
                Tile {
                    index: *territory_index,
                },
            ));

            // add edge data
            for direction in Hex::orthogonal() {
                let neighbor = *hex + direction;
                let neighbor_index = map.get(&neighbor).unwrap_or(&usize::MAX);
                if territory_index != neighbor_index {
                    let center = hex.to_grid() * 10.0;
                    let neighbor = neighbor.to_grid() * 10.0;
                    let transform = Transform::from_translation(center.extend(0.5))
                        .looking_at(
                            center.extend(0.5) - Vec3::Z,
                            (neighbor - center).extend(0.0),
                        )
                        .with_scale(Vec3::splat(10.0));
                    edges.push(transform);
                }
            }
        }

        let board = Self {
            turn: 0,
            player_order: vec![0, 1],
            territories,
        };

        (board, tiles, edges, positions)
    }

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
            && self.territories[first].dice > 0
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
        let scores = self.scores().1;
        let (player, score) = scores[self.turn];

        let mut player_teritories = Vec::new();
        for territory in self.territories.iter_mut() {
            if territory.owner == player {
                player_teritories.push(territory);
            }
        }
        for _ in 0..score {
            player_teritories
                .choose_mut(&mut rand::thread_rng())
                .unwrap()
                .dice += 1;
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

#[derive(Component)]
pub struct Tile {
    pub index: usize,
}

#[derive(Component)]
struct Dice;

pub struct BoardRenderData {
    positions: Vec<Vec2>,
    pub colours: Vec<Color>,
    pub selected: Option<usize>,
    pub hovered: Option<usize>,
    meshes: (Handle<Mesh>, Handle<Mesh>, Handle<Mesh>),
    materials: Vec<(
        Handle<ColorMaterial>,
        Handle<ColorMaterial>,
        Handle<ColorMaterial>,
    )>,
    selected_material: Handle<ColorMaterial>,
    selected_material_hover: Handle<ColorMaterial>,
}

fn setup(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // spawn in tiles
    let colours = vec![
        Color::rgb_u8(0, 147, 2),
        Color::rgb_u8(255, 255, 3),
        Color::rgb_u8(180, 126, 254),
        Color::rgb_u8(255, 127, 255),
        Color::rgb_u8(179, 255, 4),
        Color::rgb_u8(255, 127, 1),
        Color::rgb_u8(255, 88, 89),
        Color::rgb_u8(178, 255, 254),
    ];

    let meshes = (
        mesh_assets.add(Mesh::from(shape::Quad::default())),
        asset_server.load("hexagon.obj"),
        asset_server.load("edge.obj"),
    );

    let dice_texture = asset_server.load("dice.png");
    let mut materials = Vec::new();
    for i in 0..8 {
        materials.push((
            material_assets.add(ColorMaterial::from(colours[i])),
            material_assets.add(ColorMaterial::from(colours[i] * 0.8)),
            material_assets.add(ColorMaterial {
                color: colours[i],
                texture: Some(dice_texture.clone()),
            }),
        ));
    }

    let (board, tiles, edges, positions) = Board::generate();
    for (transform, tile) in tiles {
        let owner = board.territories[tile.index].owner;
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                transform,
                mesh: meshes.1.clone().into(),
                material: materials[owner].0.clone(),
                ..default()
            })
            .insert_bundle((
                tile,
                PickableMesh::default(),
                Hover::default(),
                FocusPolicy::Block,
                Interaction::None,
            ));
    }
    for transform in edges {
        commands.spawn_bundle(MaterialMesh2dBundle {
            transform,
            mesh: meshes.2.clone().into(),
            material: material_assets.add(Color::rgb_u8(0, 0, 0).into()),
            ..default()
        });
    }

    let selected_material = material_assets.add(ColorMaterial::from(Color::rgb_u8(240, 240, 240)));
    let selected_material_hover =
        material_assets.add(ColorMaterial::from(Color::rgb_u8(255, 255, 255)));

    commands.insert_resource(board);
    commands.insert_resource(BoardRenderData {
        positions,
        colours,
        selected: None,
        hovered: None,
        meshes,
        materials,
        selected_material,
        selected_material_hover,
    });
}

fn update_board(
    mut commands: Commands,
    board: Res<Board>,
    board_render_data: Res<BoardRenderData>,
    dice_query: Query<Entity, With<Dice>>,
    mut tile_query: Query<(&Tile, &mut Handle<ColorMaterial>)>,
) {
    for (tile, mut material) in tile_query.iter_mut() {
        if tile.index == board_render_data.selected.unwrap_or(usize::MAX) {
            if tile.index == board_render_data.hovered.unwrap_or(usize::MAX) {
                *material = board_render_data.selected_material_hover.clone();
            } else {
                *material = board_render_data.selected_material.clone();
            }
        } else {
            if tile.index == board_render_data.hovered.unwrap_or(usize::MAX) {
                let owner = board.territories[tile.index].owner;
                *material = board_render_data.materials[owner].1.clone();
            } else {
                let owner = board.territories[tile.index].owner;
                *material = board_render_data.materials[owner].0.clone();
            }
        }
    }

    for dice in dice_query.iter() {
        commands.entity(dice).despawn();
    }

    for i in 0..board.territories.len() {
        let dice_count = board.territories[i].dice;
        let pos = board_render_data.positions[i];
        let owner = board.territories[i].owner;

        for j in 0..dice_count {
            let transform = Transform::default()
                .with_translation(pos.extend(j as f32 + 1.0) + Vec3::new(0.0, j as f32 * 15.0, 0.0))
                .with_scale(Vec3::splat(45.0));

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: board_render_data.meshes.0.clone().into(),
                    material: board_render_data.materials[owner].2.clone(),
                    ..default()
                })
                .insert(Dice);
        }
    }
}
