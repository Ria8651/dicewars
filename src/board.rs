use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_obj::ObjPlugin)
            .insert_resource(Board::generate())
            .add_startup_system(setup)
            .add_system(update_board);
    }
}

#[derive(Debug)]
pub struct Board {
    pub teritories: Vec<Teritory>,
}

#[derive(Debug)]
pub struct Teritory {
    pub owner: Option<u32>,
    pub dice: u32,
    pub connections: Vec<usize>,
}

impl Board {
    pub fn generate() -> Self {
        let mut teritories = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..16 {
            let mut connections = Vec::new();
            for _ in 0..1 {
                connections.push(rng.gen_range(0..16));
            }
            teritories.push(Teritory {
                owner: None,
                dice: 0,
                connections,
            });
        }

        teritories[0].dice = 8;

        Self { teritories }
    }
}

#[derive(Component)]
struct BoardEntity;

#[derive(Component)]
struct Dice;

struct BoardRenderData {
    dice_mesh: Handle<Mesh>,
    dice_material: Handle<ColorMaterial>,
    positions: Vec<Vec2>,
}

fn setup(
    mut commands: Commands,
    board: Res<Board>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    let mut positions = vec![Vec2::new(0.0, 0.0); 16];

    // spawn in nodes
    let mesh_handle = meshes.add(Mesh::from(shape::Quad::default()));
    let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

    for i in 0..board.teritories.len() {
        positions[i] = Vec2::new(rng.gen_range(-300.0..300.0), rng.gen_range(-300.0..300.0));
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                transform: Transform::default()
                    .with_translation(positions[i].extend(0.0))
                    .with_scale(Vec3::splat(20.)),
                mesh: mesh_handle.clone().into(),
                material: material_handle.clone(),
                ..default()
            })
            .insert(BoardEntity);
    }

    // spawn in connections
    let mesh_handle = meshes.add(Mesh::from(shape::Quad::default()));
    let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

    let arrow_mesh_handle = asset_server.load("arrow_head.obj");

    for i in 0..board.teritories.len() {
        for j in 0..board.teritories[i].connections.len() {
            let p1 = positions[i];
            let p2 = positions[board.teritories[i].connections[j]];
            let pos = ((p1 + p2) / 2.0).extend(0.0);

            let transform = Transform::default()
                .with_translation(pos)
                .looking_at(pos - Vec3::Z, pos - p1.extend(0.0))
                .with_scale(Vec3::new(1.0, (p2 - p1).length(), 1.0));

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: mesh_handle.clone().into(),
                    material: material_handle.clone(),
                    ..default()
                })
                .insert(BoardEntity);

            // spawn arrow on end of connection
            let transform = Transform::default()
                .with_translation(pos)
                .looking_at(pos - Vec3::Z, p2.extend(0.0) - p1.extend(0.0))
                .with_scale(Vec3::splat(5.0));

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: arrow_mesh_handle.clone().into(),
                    material: material_handle.clone(),
                    ..default()
                })
                .insert(BoardEntity);
        }
    }

    let dice_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let dice_material = materials.add(ColorMaterial::from(Color::BLACK));

    commands.insert_resource(BoardRenderData {
        positions,
        dice_mesh,
        dice_material,
    });
}

fn update_board(
    mut commands: Commands,
    board: Res<Board>,
    board_render_data: Res<BoardRenderData>,
    dice_query: Query<Entity, With<Dice>>,
) {
    for dice in dice_query.iter() {
        commands.entity(dice).despawn();
    }

    for i in 0..board.teritories.len() {
        let dice_count = board.teritories[i].dice;
        let pos = board_render_data.positions[i];

        for j in 0..dice_count {
            let transform = Transform::default()
                .with_translation(pos.extend(1.0) + Vec3::new(0.0, j as f32 * 20.0, 0.0))
                .with_scale(Vec3::splat(15.0));

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    transform,
                    mesh: board_render_data.dice_mesh.clone().into(),
                    material: board_render_data.dice_material.clone(),
                    ..default()
                })
                .insert(Dice);
        }
    }
}
