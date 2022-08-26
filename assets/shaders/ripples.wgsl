#import water_effect::fullscreen
#import water_effect::dimensions

struct Params {
    water_color: vec4<f32>,
    ripples_color: vec4<f32>,
    distance_from_coast: f32,
    frequency: f32, // https://itscai.us/blog/post/jfa/
    speed: f32,
};

struct Time {
    time_since_startup: f32,
};

@group(1)  @binding(0)
var jfa_buffer: texture_2d<f32>;
@group(1) @binding(1)
var mask_buffer: texture_2d<f32>;
@group(1) @binding(2)
var nearest_sampler: sampler;

@group(2) @binding(0)
var<uniform> params: Params;

@group(3) @binding(0)
var<uniform> time: Time;

struct FragmentIn {
    @location(0) texcoord: vec2<f32>,
};

@fragment
fn fragment(in: FragmentIn) -> @location(0) vec4<f32> {
    let fb_jfa_pos = textureSample(jfa_buffer, nearest_sampler, in.texcoord).xy;
    let fb_to_pix = vec2<f32>(dims.width, dims.height);

    let mask_value = textureSample(mask_buffer, nearest_sampler, in.texcoord).r;

    // Fragment position in pixel space.
    let pix_coord = in.texcoord * fb_to_pix;
    // Closest initial fragment in pixel space.
    let pix_jfa_pos = fb_jfa_pos * fb_to_pix;

    let delta = pix_coord - pix_jfa_pos;
    let mag = sqrt(dot(delta, delta));

    // Computed texcoord and stored texcoord are likely to differ even if they
    // represent the same position due to storage as fp16, so an epsilon is
    // needed.
    if (mask_value < 1.0) {
        if (mask_value > 0.0) {
            return vec4<f32>(params.water_color.rgb, 1.0 - mask_value);  // TODO: this makes no sense right now
        } else {
            let fade = clamp(params.frequency - mag, 0.0, 1.0);          // TODO: this makes no sense right now
            return vec4<f32>(params.ripples_color.rgb, fade);            // TODO: this makes no sense right now
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}