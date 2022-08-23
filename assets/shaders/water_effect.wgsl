#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var<uniform> mesh: Mesh2d;

#import bevy_sprite::mesh2d_functions

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    out.clip_position = mesh2d_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
    // out.uv = vertex.uv;
    return out;
}

// struct Time {
//     time_since_startup: f32,
// };

// @group(2) @binding(0)
// var<uniform> time: Time;

@fragment
fn fragment() -> @location(0) vec4<f32> {

    // let speed = 2.0;
    // let t = cos(time.time_since_startup * speed);

    return vec4<f32>(0.3, 0.7, 1.0, 1.0);
}
