//! This example demonstrates how to create a custom mesh,
//! assign a custom UV mapping for a custom texture,
//! and how to change the UV mapping at run-time.

mod hexgrid;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};

use bevy_egui::{egui, EguiContexts, EguiPlugin};

use bevy::{
    color::palettes::css::*,
    pbr::wireframe::{NoWireframe, Wireframe, WireframeColor, WireframeConfig, WireframePlugin},
};

use bevy::picking::pointer::PointerInteraction;
//use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use crate::hexgrid::{HexCoordinate, HexGrid, OffsetCoordinate};

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with `With`, they're usually not queried directly since they don't
// contain information within them.
#[derive(Component)]
struct CustomUV;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            EguiPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (input_handler, ui_system))
        .insert_resource(SelectedTile(None))
        .insert_resource(HexGrid::new(50, 25))
        .run();
}

#[derive(Resource)]
struct LoadingTexture {
    is_loaded: bool,
    handle: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Res<HexGrid>
) {

    // Import the custom texture.
    // let custom_texture_handle: Handle<Image> = asset_server
    //     .load_with_settings("textures/Tex1.png",
    //                         |s: &mut _| {
    //                             *s = ImageLoaderSettings {
    //                                 sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
    //                                     // rewriting mode to repeat image,
    //                                     address_mode_u: ImageAddressMode::Repeat,
    //                                     address_mode_v: ImageAddressMode::Repeat,
    //                                     ..default()
    //                                 }),
    //                                 ..default()
    //                             }
    //                         },
    //     );


    let test_grid = grid;
    let hex_mesh_handle: Handle<Mesh> = meshes.add(test_grid.triangulate_grid());
    //Render the mesh with the custom texture, and add the marker.
    // commands.spawn((
    //     Mesh3d(hex_mesh_handle),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color_texture: Some(custom_texture_handle),
    //         reflectance: 0.1,
    //         perceptual_roughness: 0.9,
    //         ..default()
    //     })),
    //     CustomUV,
    // ));

    commands
        .spawn((
            Mesh3d(hex_mesh_handle),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
            Wireframe,
            CustomUV,
        ))
        .observe(clicked_map);

    // commands
    //     .spawn((
    //         Mesh3d(hex_mesh_handle),
    //         MeshMaterial3d(materials.add(HexTerrainMaterial {
    //             color_texture: Some(asset_server.load("textures/Tex1.png")),
    //             alpha_mode: AlphaMode::Blend
    //         })),
    //         Wireframe,
    //         CustomUV,
    //     ))
    //     .observe(clicked_map);

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_transform =
        Transform::from_xyz(0.0, 120.0, 120.0).looking_at(Vec3::ZERO, Vec3::Y);

    let light_transform =
        Transform::from_xyz(40.0, 5.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn((Camera3d::default(), camera_transform));

    // Light up the scene.
    commands.spawn((DirectionalLight::default(), light_transform));

    // Text to describe the controls.
    // commands.spawn((
    //     Text::new("Controls:\nSpace: Change UVs\nX/Y/Z: Rotate\nR: Reset orientation"),
    //     Node {
    //         position_type: PositionType::Absolute,
    //         top: Val::Px(12.0),
    //         left: Val::Px(12.0),
    //         ..default()
    //     },
    // ));
}


// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    //mesh_query: Query<&Mesh3d, With<CustomUV>>,
    //mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    // if keyboard_input.pressed(KeyCode::KeyX) {
    //     for mut transform in &mut query {
    //         transform.rotate_x(time.delta_secs() / 1.2);
    //     }
    // }
    // if keyboard_input.pressed(KeyCode::KeyY) {
    //     for mut transform in &mut query {
    //         transform.rotate_y(time.delta_secs() / 1.2);
    //     }
    // }
    // if keyboard_input.pressed(KeyCode::KeyZ) {
    //     for mut transform in &mut query {
    //         transform.rotate_z(time.delta_secs() / 1.2);
    //     }
    // }
    if keyboard_input.pressed(KeyCode::KeyW) {
        for mut transform in &mut query {
            transform.translation -= Vec3::Z*(60.0*time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        for mut transform in &mut query {
            transform.translation += Vec3::Z*(60.0*time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        for mut transform in &mut query {
            transform.translation += Vec3::X*(60.0*time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        for mut transform in &mut query {
            transform.translation -= Vec3::X*(60.0*time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

#[derive(Resource)]
struct SelectedTile (Option<OffsetCoordinate>);
fn clicked_map(
    _: Trigger<Pointer<Click>>,
    pointers: Query<&PointerInteraction>,
    mut selected_tile: ResMut<SelectedTile>
) {
    for (point, _) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        //println!("{}, {}", point.x, point.z);
        let hex_idx = OffsetCoordinate::from_position(point);
        selected_tile.0 = Some(hex_idx);
        println!("{}, {}", hex_idx.x, hex_idx.z);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    selected_tile: Res<SelectedTile>,
    mut grid: ResMut<HexGrid>,
    mut commands: Commands,
    query: Query<Entity, With<Mesh3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //NE: 0
    // W: 1
    //SE: 2
    //SW: 3
    // E: 4
    //NW: 5
    let dir_names = ["N ", "NE", "SE", "S ", "SW", "NW"];
    let mut changed = false;
    egui::Window::new("Test").show(contexts.ctx_mut(), |ui| {
        match selected_tile.0 {
            None => {ui.label("No selected tile.");}
            Some(idx) => {
                ui.label(format!("Selected: {}, {}", idx.x, idx.z));

                let tile = &grid.cells[idx.x][idx.z];
                let height_refs = tile.height_refs.clone();
                for (i, &(hx, hz)) in height_refs.iter().enumerate() {
                    ui.label(dir_names[i]);
                    changed = ui.add(egui::Slider::new(&mut grid.heights[hx][hz], 0..=5)).changed() || changed;
                    ui.end_row();
                }
                ui.horizontal(|ui| {
                    if ui.button("Raise").clicked() {
                        for &(hx, hz) in height_refs.iter() {
                            grid.heights[hx][hz] += 1;
                        }
                        changed = true;
                    }
                    if ui.button("Lower").clicked() {
                        for &(hx, hz) in height_refs.iter() {
                            grid.heights[hx][hz] -= 1;
                        }
                        changed = true;
                    }
                    if ui.button("Flatten").clicked() {
                        let mut sum = 0;
                        for &(hx, hz) in height_refs.iter() {
                            sum += grid.heights[hx][hz];
                        }
                        sum = (sum as f32/6.0).round() as i32;
                        for &(hx, hz) in height_refs.iter() {
                            grid.heights[hx][hz] = sum;
                        }
                        changed = true;
                    }
                });

            }
        }
    });

    if changed {
        println!("Change!");
        for entity in query.iter() {
            commands.entity(entity).despawn();
            let hex_mesh_handle: Handle<Mesh> = meshes.add(grid.triangulate_grid());


            commands
                .spawn((
                    Mesh3d(hex_mesh_handle),
                    MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                    Wireframe,
                    CustomUV,
                ))
                .observe(clicked_map);
        }
    }
}
