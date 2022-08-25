// Mask generation shader.

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

@group(1) @binding(0)
var<uniform> mesh: Mesh;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    return out;
}

@group(1) @binding(0)
var water_texture: texture_2d<f32>;
@group(1) @binding(1)
var water_sampler: sampler;

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {

    var input_colour: vec4<f32> = textureSample(water_texture, water_sampler, uv);

    // NOTE: it's a bit dumb, but basically it flips the result from the water_sprites shader
    // so that the water is the one that will have the SDFs calculated for, instead of being the outline of the mesh,
    // like in Bevy JFA
    //
    // Overall, if i could manage to pass the water_texture to the JFA Init node directly, 
    // I wouldn't need to do dumb stuff here
    //
    // It would also allow me to skip the stencil test that Bevy JFA has (in the original MeshMask node), 
    // because i know already the fragments that i want the SDFs calculated for

    if input_colour.a > 0. {
        input_colour = vec4<f32>(0., 0., 0., 0.);
    } else {
        input_colour = vec4<f32>(1., 1., 1., 1.);
    }

    var result: vec4<f32> = input_colour;
    return result;
}
