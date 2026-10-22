import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var meadow_color: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var meadow_sampler: sampler;
struct MeadowTint { value: vec4<f32>, }
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var<uniform> meadow_tint: MeadowTint;
fn hash2(cell: vec2<f32>) -> vec2<f32> {
    return fract(sin(vec2<f32>(dot(cell,vec2<f32>(127.1,311.7)),dot(cell,vec2<f32>(269.5,183.3))))*43758.5453);
}
fn sample_meadow_tile(uv:vec2<f32>, cell:vec2<f32>, dx:vec2<f32>, dy:vec2<f32>) -> vec3<f32> {
    let random=hash2(cell);
    let angle=random.x*6.2831853;
    let rotation=mat2x2<f32>(cos(angle),sin(angle),-sin(angle),cos(angle));
    return textureSampleGrad(meadow_color,meadow_sampler,rotation*uv+random,rotation*dx,rotation*dy).rgb;
}
fn meadow_albedo(uv:vec2<f32>) -> vec3<f32> {
    let cell=floor(uv*0.55);
    let weight=smoothstep(vec2<f32>(0.0),vec2<f32>(1.0),fract(uv*0.55));
    let dx=dpdx(uv);let dy=dpdy(uv);
    let a=sample_meadow_tile(uv,cell,dx,dy);
    let b=sample_meadow_tile(uv,cell+vec2<f32>(1.0,0.0),dx,dy);
    let c=sample_meadow_tile(uv,cell+vec2<f32>(0.0,1.0),dx,dy);
    let d=sample_meadow_tile(uv,cell+vec2<f32>(1.0,1.0),dx,dy);
    return mix(mix(a,b,weight.x),mix(c,d,weight.x),weight.y);
}
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in,front);
    let meadow = meadow_albedo(in.uv) * in.color.rgb * meadow_tint.value.rgb;
    let soil_lightness=dot(pbr.material.base_color.rgb,vec3<f32>(0.2126,0.7152,0.0722));
    let soil=vec3<f32>(soil_lightness)*vec3<f32>(1.20,1.18,1.14);
    pbr.material.base_color = vec4<f32>(mix(soil,meadow,clamp(in.color.a,0.0,1.0)),1.0);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr,out.color);
    return out;
}
