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
fn terrain_noise(p: vec2<f32>) -> f32 {
    let i=floor(p); let f=fract(p); let w=f*f*(3.0-2.0*f);
    return mix(mix(hash2(i).x,hash2(i+vec2<f32>(1.0,0.0)).x,w.x),mix(hash2(i+vec2<f32>(0.0,1.0)).x,hash2(i+vec2<f32>(1.0,1.0)).x,w.x),w.y);
}
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in,front);
    let world = in.world_position.xz;
    let broad = terrain_noise(world*0.13)*0.65 + terrain_noise(world*0.37)*0.35;
    let palette = mix(vec3<f32>(0.085,0.12,0.035),vec3<f32>(0.22,0.25,0.10),smoothstep(0.15,0.85,broad));
    let detail = meadow_albedo(world*0.27);
    let meadow = palette * (vec3<f32>(0.65)+detail*1.8) * in.color.rgb * meadow_tint.value.rgb;
    let soil_lightness=dot(pbr.material.base_color.rgb,vec3<f32>(0.2126,0.7152,0.0722));
    let soil=vec3<f32>(soil_lightness)*vec3<f32>(1.20,1.18,1.14);
    let coverage=clamp(in.color.a+(terrain_noise(world*1.7)-0.5)*0.18,0.0,1.0);
    pbr.material.base_color = vec4<f32>(mix(soil*0.80,meadow,coverage),1.0);
    let micro=world*22.0;
    let slope=vec2<f32>(terrain_noise(micro+vec2<f32>(0.15,0.0))-terrain_noise(micro-vec2<f32>(0.15,0.0)),terrain_noise(micro+vec2<f32>(0.0,0.15))-terrain_noise(micro-vec2<f32>(0.0,0.15)));
    let footprint=max(length(dpdx(world)),length(dpdy(world)));
    let detail_fade=1.0-smoothstep(0.025,0.16,footprint);
    pbr.N=normalize(pbr.N+vec3<f32>(-slope.x,0.0,-slope.y)*0.8*detail_fade);
    pbr.material.perceptual_roughness=0.96;
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr,out.color);
    return out;
}
