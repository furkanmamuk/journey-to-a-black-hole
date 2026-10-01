#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
struct Settings { parameters:vec4<f32>, flags:vec4<f32>, }
@group(0) @binding(0) var source:texture_2d<f32>;
@group(0) @binding(1) var<uniform> s:Settings;

fn bayer(p:vec2<u32>)->f32 {
    // Stable 4x4 ordered dither, no per-frame noise.
    let x=p.x&3u;let y=p.y&3u;
    let low=((x&1u)^(y&1u))*2u+(y&1u);
    let high=(((x>>1u)^(y>>1u))&1u)*2u+((y>>1u)&1u);
    return (f32(low*4u+high)+0.5)/16.0-0.5;
}
@fragment fn reduce(in:FullscreenVertexOutput)->@location(0) vec4<f32> {
    let size=vec2<f32>(textureDimensions(source));
    let grid=select(size,s.parameters.xy,s.flags.x>0.5);
    let cell=floor(in.position.xy);let lo=cell*size/grid;let hi=(cell+1.0)*size/grid;
    // Exact box reduction for usual 2x/3x/4x ratios; bounded 9x9 support for ratios <=8 (fractional edges included).
    let first=vec2<i32>(floor(lo));let last=vec2<i32>(ceil(hi));
    var color=vec3<f32>(0.0);var weight=0.0;
    for(var y=0;y<9;y++){for(var x=0;x<9;x++){
        let p=first+vec2<i32>(x,y);
        if all(p<last) && all(p<vec2<i32>(size)) {
            let overlap=max(vec2<f32>(0.0),min(vec2<f32>(p)+1.0,hi)-max(vec2<f32>(p),lo));
            let w=overlap.x*overlap.y;color+=textureLoad(source,p,0).rgb*w;weight+=w;
        }
    }}
    color/=max(weight,0.0001);
    if s.flags.y>0.5 {
        // Quantize perceptual channels; 48+ levels, no fixed tiny palette.
        var perceptual=pow(max(color,vec3<f32>(0.0)),vec3<f32>(1.0/2.2));
        let n=bayer(vec2<u32>(cell))*s.parameters.w;
        perceptual=clamp(floor(perceptual*(s.parameters.z-1.0)+0.5+n)/(s.parameters.z-1.0),vec3<f32>(0.0),vec3<f32>(1.0));
        color=pow(perceptual,vec3<f32>(2.2));
    }
    return vec4<f32>(color,1.0);
}
@fragment fn expand(in:FullscreenVertexOutput)->@location(0) vec4<f32> {
    let size=textureDimensions(source);
    let p=min(vec2<u32>(in.uv*vec2<f32>(size)),size-1u);
    return textureLoad(source,vec2<i32>(p),0);
}
