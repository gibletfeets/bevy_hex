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
};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, OpaqueRendererMethod};
use bevy::picking::pointer::PointerInteraction;
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, VertexFormat};
//use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use crate::hexgrid::{HexGrid, OffsetCoordinate};

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with `With`, they're usually not queried directly since they don't
// contain information within them.

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            EguiPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, HexTerrainExtension>,>::default()
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (create_map, input_handler, ui_system))
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
    //mut materials2: ResMut<Assets<ExtendedMaterial<StandardMaterial, HexTerrainExtension>>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    //mut images: ResMut<Assets<Image>>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //grid: Res<HexGrid>
) {
    // let test_grid = grid;
    // let hex_mesh_handle: Handle<Mesh> = meshes.add(test_grid.triangulate_grid());
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        handle: asset_server.load("textures/array_texture.png"),
    });

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


    // commands
    //     .spawn((
    //         Mesh3d(hex_mesh_handle),
    //         MeshMaterial3d(materials.add(StandardMaterial {
    //             base_color: Color::srgb(1.0, 1.0, 1.0),
    //             reflectance: 0.1,
    //             perceptual_roughness: 0.9,
    //             ..default()
    //         })),
    //         CustomUV,
    //     ))
    //     .observe(clicked_map);

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_transform =
        Transform::from_xyz(0.0, 100.0, 160.0).looking_at(Vec3::ZERO, Vec3::Y);

    let light_transform =
        Transform::from_xyz(40.0, 5.0, -40.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // 6 world units per pixel of window height.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 100.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        camera_transform
    ));

    // Light up the scene.
    commands.spawn((DirectionalLight::default(), light_transform));
}


fn create_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_texture: ResMut<LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, HexTerrainExtension>>>,
    grid: Res<HexGrid>
) {
    if loading_texture.is_loaded
        || !asset_server
        .load_state(loading_texture.handle.id())
        .is_loaded()
    {
        return;
    }
    loading_texture.is_loaded = true;
    let image = images.get_mut(&loading_texture.handle).unwrap();

    // Create a new array texture asset from the loaded texture.
    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);

    let test_grid = grid;
    let hex_mesh_handle: Handle<Mesh> = meshes.add(test_grid.triangulate_grid());

    let material_handle: Handle<ExtendedMaterial<StandardMaterial, HexTerrainExtension>> = materials.add({
        ExtendedMaterial{
            base: StandardMaterial {
                base_color: WHITE.into(),
                opaque_render_method: OpaqueRendererMethod::Auto,
                reflectance: 0.1,
                perceptual_roughness: 0.9,
                ..Default::default()
            },
            extension: HexTerrainExtension {
                array_texture: loading_texture.handle.clone(),
            }
        }
    });

    commands.spawn((
        Mesh3d(hex_mesh_handle.clone()),
        MeshMaterial3d(material_handle.clone()),

    ))
    .observe(clicked_map);
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
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, HexTerrainExtension>>>,
    mut loading_texture: ResMut<LoadingTexture>,
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
        for entity in query.iter() {
            commands.entity(entity).despawn();
            let hex_mesh_handle: Handle<Mesh> = meshes.add(grid.triangulate_grid());

            let material_handle: Handle<ExtendedMaterial<StandardMaterial, HexTerrainExtension>> = materials.add({
                ExtendedMaterial{
                    base: StandardMaterial {
                        base_color: WHITE.into(),
                        opaque_render_method: OpaqueRendererMethod::Auto,
                        reflectance: 0.1,
                        perceptual_roughness: 0.9,
                        ..Default::default()
                    },
                    extension: HexTerrainExtension {
                        array_texture: loading_texture.handle.clone(),
                    }
                }
            });

            commands.spawn((
                Mesh3d(hex_mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
            ))
                .observe(clicked_map);
        }
    }
}

const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 988540917, VertexFormat::Uint32x3 );

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct HexTerrainExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    array_texture: Handle<Image>,
}

impl MaterialExtension for HexTerrainExtension {
    fn vertex_shader() -> ShaderRef {
        "shaders/hex_terrain_vertex.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/hex_terrain_fragment.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/hex_terrain_fragment.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // layout.0.get_layout(attribute_descriptors);
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            // Mesh::ATTRIBUTE_TANGENT.at_shader_location(3),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
            ATTRIBUTE_TEXTURE_INDEX.at_shader_location(8),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}
