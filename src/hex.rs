use bevy::prelude::*;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl Hex {
    pub const fn new(q: i32, r: i32, s: i32) -> Self {
        Self { q, r, s }
    }

    pub fn to_grid(&self) -> Vec2 {
        let sqrt3 = 3.0f32.sqrt();
        let x = sqrt3 * self.q as f32 + sqrt3 / 2.0 * self.r as f32;
        let y = (3.0 / 2.0) * self.r as f32;

        return Vec2::new(x, y);
    }

    pub const fn orthogonal() -> [Hex; 6] {
        [
            Hex::new(1, -1, 0),
            Hex::new(-1, 1, 0),
            Hex::new(0, 1, -1),
            Hex::new(0, -1, 1),
            Hex::new(1, 0, -1),
            Hex::new(-1, 0, 1),
        ]
    }
}

impl std::ops::Add for Hex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            q: self.q + other.q,
            r: self.r + other.r,
            s: self.s + other.s,
        }
    }
}

#[derive(Component)]
pub struct TmpMapTile;

// fn map_setup(mut map_generation: ResMut<MapGeneration>) {
//     use rand::Rng;
//     let mut rng = rand::thread_rng();

//     for i in 0..10 {
//         let q = rng.gen_range(-10..10);
//         let r = rng.gen_range(-10..10);
//         map_generation.map.insert(Hex::new(q, r, -q - r), i);
//     }
// }

// fn map_generation_system(
//     mut map_generation: ResMut<MapGeneration>,
//     mut egui_context: ResMut<EguiContext>,
// ) {
//     egui::Window::new("Map Gen").show(egui_context.ctx_mut(), |ui| {
//         if ui.button("Spread Hex").clicked() {
//             for (hex, territory) in map_generation.map.clone().iter() {
//                 for direction in Hex::orthogonal() {
//                     let neighbor = *hex + direction;
//                     if !map_generation.map.contains_key(&neighbor) {
//                         map_generation.map.insert(neighbor, *territory);
//                     }
//                 }
//             }
//         }
//     });
// }

// fn map_renderer_system(
//     mut commands: Commands,
//     map_generation: Res<MapGeneration>,
//     tiles: Query<Entity, With<TmpMapTile>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     for tile in tiles.iter() {
//         commands.entity(tile).despawn();
//     }

//     let mesh_handle = asset_server.load("hexagon.obj");
//     let material_handle = materials.add(ColorMaterial::from(Color::GRAY));

//     for (hex, _) in map_generation.map.iter() {
//         let transform = Transform::from_translation(hex.to_grid().extend(0.0) * 10.0)
//             .with_scale(Vec3::splat(10.0));
//         commands
//             .spawn_bundle(MaterialMesh2dBundle {
//                 transform,
//                 mesh: mesh_handle.clone().into(),
//                 material: material_handle.clone(),
//                 ..default()
//             })
//             .insert(TmpMapTile);
//     }
// }