import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};
struct WaterParameters { values: vec4<f32>, }
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> water: WaterParameters;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in, front);
    let p = in.world_position.xz;
    let t = water.values.x;
    // Same beach crest and linear underwater profile as the CPU height field.
    let crest = 8.7 + 1.4*sin(p.x*0.075) + 0.5*sin(p.x*0.29);
    let depth = max(0.0, (p.y-crest-1.2)*0.29);
    let a = dot(p,vec2<f32>(0.71,0.35))*3.1-t*1.3+0.8*sin(p.y*1.3+t*0.2);
    let b = dot(p,vec2<f32>(-0.28,0.93))*6.7+t*1.8+0.65*sin(p.x*0.93);
    let c = dot(p,vec2<f32>(0.87,-0.31))*14.0-t*2.1+0.8*sin(p.y*3.1);
    pbr.N = normalize(vec3<f32>(0.055*cos(a)+0.018*cos(c),1.0,0.045*cos(b)+0.013*cos(c)));
    let fresnel = 0.04+0.96*pow(1.0-clamp(dot(pbr.N,pbr.V),0.0,1.0),5.0);
    let shallow = exp(-depth*0.9);
    let foam = (1.0-smoothstep(0.04,0.36,depth))*smoothstep(0.25,0.85,sin(a*0.42)+0.18*sin(b));
    pbr.material.base_color = vec4<f32>(mix(vec3<f32>(0.095,0.14,0.17),vec3<f32>(0.17,0.24,0.21),shallow)+vec3<f32>(foam*0.2),clamp(0.45+depth*0.28,0.45,0.97));
    pbr.material.perceptual_roughness = 0.19+foam*0.3;
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    let sky = mix(vec3<f32>(0.045,0.065,0.10),vec3<f32>(0.33,0.43,0.50),water.values.y);
    out.color = vec4<f32>(out.color.rgb+sky*(0.18+fresnel*0.7)*(1.0-water.values.z),out.color.a);
    out.color = main_pass_post_lighting_processing(pbr,out.color);
    return out;
}
