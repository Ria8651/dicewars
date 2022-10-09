use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    ui::{FocusPolicy, Interaction},
};
use bevy_mod_picking::*;
use rand::{seq::SliceRandom, Rng};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_obj::ObjPlugin)
            .insert_resource(Board::generate())
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
pub struct Territory {
    owner: usize,
    dice: u32,
    connections: Vec<usize>,
}

impl Board {
    pub fn generate() -> Self {
        let mut territories = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..16 {
            let mut connections = Vec::new();
            for _ in 0..1 {
                connections.push(rng.gen_range(0..16));
            }
            territories.push(Territory {
                owner: rng.gen_range(0..8),
                dice: rng.gen_range(1..6),
                connections,
            });
        }

        let mut player_order = vec![0, 1, 2, 3, 4, 5, 6, 7];
        player_order.shuffle(&mut rng);

        Self {
            turn: 0,
            player_order,
            territories,
        }
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
struct Arrow;

#[derive(Component)]
pub struct Node {
    pub index: usize,
    pub hovered: bool,
    pub selected: bool,
}

#[derive(Component)]
struct Dice;

pub struct BoardRenderData {
    square_mesh: Handle<Mesh>,
    positions: Vec<Vec2>,
    pub colours: Vec<Color>,
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
    mut board: ResMut<Board>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    let mut positions = vec![Vec2::new(0.0, 0.0); 16];
    for i in 0..board.territories.len() {
        positions[i] = Vec2::new(rng.gen_range(-300.0..300.0), rng.gen_range(-300.0..300.0));
    }
    for i in 0..board.territories.len() {
        board.territories[i].connections = Vec::new();
        for j in 0..board.territories.len() {
            if positions[i].distance(positions[j]) < 200.0 {
                board.territories[i].connections.push(j);
            }
        }
    }

    // spawn in nodes
    let mesh_handle = asset_server.load("hexagon.obj");

    let dice_texture = asset_server.load("dice.png");
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
    let mut node_materials = Vec::new();
    for i in 0..8 {
        node_materials.push((
            materials.add(ColorMaterial::from(colours[i])),
            materials.add(ColorMaterial::from(colours[i] * 0.8)),
            materials.add(ColorMaterial {
                color: colours[i],
                texture: Some(dice_texture.clone()),
            }),
        ));
    }

    for i in 0..board.territories.len() {
        let owner = board.territories[i].owner;
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                transform: Transform::default()
                    .with_translation(positions[i].extend(0.5))
                    .with_scale(Vec3::new(15.0, 13.0, 1.0)),
                mesh: mesh_handle.clone().into(),
                material: node_materials[owner].0.clone(),
                ..default()
            })
            .insert(Node {
                index: i,
                selected: false,
                hovered: false,
            })
            .insert_bundle((
                PickableMesh::default(),
                Hover::default(),
                FocusPolicy::Block,
                Interaction::None,
            ));
    }

    // spawn in connections
    let square_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

    // let arrow_mesh_handle = asset_server.load("arrow_head.obj");

    for i in 0..board.territories.len() {
        for j in 0..board.territories[i].connections.len() {
            let p1 = positions[i];
            let p2 = positions[board.territories[i].connections[j]];
            let pos = ((p1 + p2) / 2.0).extend(0.0);

            let transform = Transform::default()
                .with_translation(pos)
                .looking_at(pos - Vec3::Z, pos - p1.extend(0.0))
                .with_scale(Vec3::new(1.0, (p2 - p1).length(), 1.0));

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: square_mesh.clone().into(),
                    material: material_handle.clone(),
                    ..default()
                })
                .insert(Arrow);

            // spawn arrow on end of connection
            // let transform = Transform::default()
            //     .with_translation(pos)
            //     .looking_at(pos - Vec3::Z, p2.extend(0.0) - p1.extend(0.0))
            //     .with_scale(Vec3::splat(5.0));

            // commands
            //     .spawn_bundle(MaterialMesh2dBundle {
            //         transform,
            //         mesh: arrow_mesh_handle.clone().into(),
            //         material: material_handle.clone(),
            //         ..default()
            //     })
            //     .insert(Arrow);
        }
    }

    let selected_material = materials.add(ColorMaterial::from(Color::rgb_u8(240, 240, 240)));
    let selected_material_hover = materials.add(ColorMaterial::from(Color::rgb_u8(255, 255, 255)));

    commands.insert_resource(BoardRenderData {
        square_mesh,
        positions,
        colours,
        materials: node_materials,
        selected_material,
        selected_material_hover,
    });
}

fn update_board(
    mut commands: Commands,
    board: Res<Board>,
    board_render_data: Res<BoardRenderData>,
    dice_query: Query<Entity, With<Dice>>,
    mut node_query: Query<(&Node, &mut Handle<ColorMaterial>)>,
) {
    for (node, mut material) in node_query.iter_mut() {
        if node.selected {
            if node.hovered {
                *material = board_render_data.selected_material_hover.clone();
            } else {
                *material = board_render_data.selected_material.clone();
            }
        } else {
            if node.hovered {
                let owner = board.territories[node.index].owner;
                *material = board_render_data.materials[owner].1.clone();
            } else {
                let owner = board.territories[node.index].owner;
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
                    mesh: board_render_data.square_mesh.clone().into(),
                    material: board_render_data.materials[owner].2.clone(),
                    ..default()
                })
                .insert(Dice);
        }
    }
}
