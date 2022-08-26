// Mask generation shader.

@group(1) @binding(0)
var water_texture: texture_2d<f32>;
@group(1) @binding(1)
var water_sampler: sampler;

struct FragmentIn {
    @location(0) texcoord: vec2<f32>,
};

@fragment
fn fragment(in: FragmentIn) -> @location(0) vec4<f32> {

    var input_colour: vec4<f32> = textureSample(water_texture, water_sampler, in.uv);

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
