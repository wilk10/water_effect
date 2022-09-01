// #import bevy_sprite::mesh2d_types
// #import bevy_sprite::mesh2d_view_bindings

// @group(1) @binding(0)
// var<uniform> mesh: Mesh2d;

// #import bevy_sprite::mesh2d_functions

// struct Vertex {
//     @location(0) position: vec3<f32>,
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     // @location(0) uv: vec2<f32>,
// };

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
//     var out: VertexOutput;
//     // out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
//     out.clip_position = mesh2d_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
//     // out.uv = vertex.uv;
//     return out;
// }

// struct Time {
//     time_since_startup: f32,
// };

// @group(2) @binding(0)
// var<uniform> time: Time;

@group(1) @binding(0)
var water_texture: texture_2d<f32>;
@group(1) @binding(1)
var water_sampler: sampler;

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {

    var input_colour: vec4<f32> = textureSample(water_texture, water_sampler, uv);


    // TODO: temporarily removed !!!!!!

    // if input_colour.a > 0. {
    //     input_colour = vec4<f32>(input_colour, 1.);
    // } else {
    //     input_colour = vec4<f32>(0., 0., 0., 0.);
    // }

    var result: vec4<f32> = vec4<f32>(1., input_colour.g, input_colour.b, 1.);
    return result;
}
