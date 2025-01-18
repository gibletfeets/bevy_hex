#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions as fns,
    pbr_bindings,
}

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
//@group(2) @binding(0) var material_color_texture: texture_2d_array<f32>;;
@group(2) @binding(1) var material_color_sampler: sampler;



@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return textureSample(material_color_texture, material_color_sampler, fract(mesh.uv))*mesh.color.r;
}