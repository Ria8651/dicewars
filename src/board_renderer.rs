use super::board::{Board, BoardGenSettings};
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    ui::{FocusPolicy, Interaction},
};
use bevy_mod_picking::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_obj::ObjPlugin)
            .add_event::<RegenerateBoardEvent>()
            .insert_resource(BoardGenSettings {
                player_count: 3,
                board_size: 20,
            })
            .add_startup_system(setup)
            .add_stage_after(CoreStage::Update, "Post", SystemStage::parallel())
            .add_system_to_stage("Post", update_board);
    }
}

// this squishes the board verticly to make it look like it has perspective
pub const SCALE: Vec2 = Vec2::new(12.0, 9.0);

#[derive(Component)]
pub struct Tile {
    pub index: usize,
}

#[derive(Component)]
struct Edge;

#[derive(Component)]
struct Dice;

pub struct BoardRenderData {
    positions: Vec<Vec2>,
    pub colours: Vec<Color>,
    pub selected: Option<usize>,
    pub hovered: Option<usize>,
    pub attackable: Vec<usize>,
    // quad, hexagon, edge
    meshes: (Handle<Mesh>, Handle<Mesh>, Handle<Mesh>),
    // territory normal, territory hovered, territory attackable, dice material
    materials: Vec<(
        Handle<ColorMaterial>,
        Handle<ColorMaterial>,
        Handle<ColorMaterial>,
        Handle<ColorMaterial>,
    )>,
    edge_material: Handle<ColorMaterial>,
    selected_material: Handle<ColorMaterial>,
    selected_material_hover: Handle<ColorMaterial>,
}

pub struct RegenerateBoardEvent;

fn setup(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut regenerate_board_event: EventWriter<RegenerateBoardEvent>,
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
            material_assets.add(ColorMaterial::from(colours[i] * 0.9)),
            material_assets.add(ColorMaterial {
                color: colours[i],
                texture: Some(dice_texture.clone()),
            }),
        ));
    }

    let edge_material = material_assets.add(ColorMaterial::from(Color::rgb_u8(0, 0, 0)));
    let selected_material = material_assets.add(ColorMaterial::from(Color::rgb_u8(240, 240, 240)));
    let selected_material_hover =
        material_assets.add(ColorMaterial::from(Color::rgb_u8(255, 255, 255)));

    regenerate_board_event.send(RegenerateBoardEvent);

    // add empty board so it doesn't crash
    commands.insert_resource(Board {
        turn: 0,
        player_order: Vec::new(),
        territories: Vec::new(),
    });
    commands.insert_resource(BoardRenderData {
        positions: Vec::new(),
        colours,
        selected: None,
        hovered: None,
        attackable: Vec::new(),
        meshes,
        materials,
        edge_material,
        selected_material,
        selected_material_hover,
    });
}

fn update_board(
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut board_render_data: ResMut<BoardRenderData>,
    dice_query: Query<Entity, With<Dice>>,
    mut tile_query: Query<(&Tile, &mut Handle<ColorMaterial>)>,
    tile_entity_query: Query<Entity, With<Tile>>,
    edge_entity_query: Query<Entity, With<Edge>>,
    mut regenerate_board_event: EventReader<RegenerateBoardEvent>,
    board_gen_settings: Res<BoardGenSettings>,
) {
    // update material handles
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
                if board_render_data.attackable.contains(&tile.index) {
                    let owner = board.territories[tile.index].owner;
                    *material = board_render_data.materials[owner].2.clone();
                } else {
                    let owner = board.territories[tile.index].owner;
                    *material = board_render_data.materials[owner].0.clone();
                }
            }
        }
    }

    // update dice
    for dice in dice_query.iter() {
        commands.entity(dice).despawn();
    }

    let dice_size = 16.0;
    for i in 0..board.territories.len() {
        let dice_count = board.territories[i].dice;
        let pos = board_render_data.positions[i];
        let owner = board.territories[i].owner;

        for j in 0..dice_count {
            let loop_height = 4;
            let offset = Vec3::new(
                -dice_size,
                dice_size / 2.0 - loop_height as f32 * dice_size,
                -(loop_height as f32 + 1.0) / 1000.0,
            );

            let mut dice_pos = Vec3::Y * j as f32 * dice_size
                + pos.extend((loop_height + j) as f32 / 1000.0 + 1.0);
            if j >= loop_height {
                dice_pos += offset;
            }

            dice_pos.z += -pos.y + 500.0;

            let transform = Transform::default()
                .with_translation(dice_pos)
                .with_scale(Vec3::splat(dice_size * 3.0));

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: board_render_data.meshes.0.clone().into(),
                    material: board_render_data.materials[owner].3.clone(),
                    ..default()
                })
                .insert(Dice);
        }
    }

    // generate new board
    for _ in regenerate_board_event.iter() {
        // delete old board
        for tile in tile_entity_query.iter() {
            commands.entity(tile).despawn();
        }
        for edge in edge_entity_query.iter() {
            commands.entity(edge).despawn();
        }

        // generate new board
        let (new_board, tiles, positions) = Board::generate(&board_gen_settings);
        for (transform, tile, edges) in tiles {
            let owner = new_board.territories[tile.index].owner;
            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: board_render_data.meshes.1.clone().into(),
                    material: board_render_data.materials[owner].0.clone(),
                    ..default()
                })
                .insert_bundle((
                    tile,
                    PickableMesh::default(),
                    Hover::default(),
                    FocusPolicy::Block,
                    Interaction::None,
                ))
                .with_children(|parent| {
                    for transform in edges {
                        parent
                            .spawn_bundle(MaterialMesh2dBundle {
                                transform,
                                mesh: board_render_data.meshes.2.clone().into(),
                                material: board_render_data.edge_material.clone(),
                                ..default()
                            })
                            .insert(Edge);
                    }
                });
        }

        board_render_data.positions = positions;
        *board = new_board;
    }
}
