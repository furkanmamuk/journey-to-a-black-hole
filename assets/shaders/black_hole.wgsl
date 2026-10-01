#import "shaders/spectrum.wgsl"::thermal_rgb
#import "shaders/kerr.wgsl"::{State, metric, initial_momentum, derivative, rk4, constraint}
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Settings {
    camera:vec4<f32>, right:vec4<f32>, up:vec4<f32>, forward:vec4<f32>,
    geometry:vec4<f32>, integration:vec4<f32>, features:vec4<f32>, relativity:vec4<f32>,
    disk:vec4<f32>, center:vec4<f32>, sky:vec4<u32>, model:vec4<f32>, axes_x:vec4<f32>, axes_y:vec4<f32>, axes_z:vec4<f32>,
}
@group(0) @binding(0) var scene:texture_2d<f32>;
@group(0) @binding(1) var depth:texture_depth_2d;
@group(0) @binding(2) var<uniform> s:Settings;

fn hash(p:vec3<f32>)->f32 {
    let bits=bitcast<vec3<u32>>(p);
    var h=bits.x*747796405u ^ bits.y*2891336453u ^ bits.z*277803737u ^ s.sky.x;
    h=(h^(h>>16u))*2246822519u;h=(h^(h>>13u))*3266489917u;h=h^(h>>16u);
    return f32(h>>8u)/16777216.0;
}
// Directional cube charts: no sky mesh, no screen-space random stars.
// Gaussian stars have an energy-conserving pixel footprint and no time noise.
fn star_chart(uv:vec2<f32>,face:f32,footprint:f32)->vec3<f32> {
    var color=vec3<f32>(0.0);
    for(var layer=0u;layer<2u;layer++) {
        let cells=select(52.0,109.0,layer==1u);let p=uv*cells;let base=floor(p);
        let pixel=clamp(footprint*cells,0.015,1.2);
        for(var y=-1;y<=1;y++){for(var x=-1;x<=1;x++){
            let cell=base+vec2<f32>(f32(x),f32(y));let seed=vec3<f32>(cell,face+13.0*f32(layer));
            let h=hash(seed);
            if h>0.978 {
                let center=cell+vec2<f32>(hash(seed+1.7),hash(seed+7.9));
                // Fade by the fixed star's position, not the query direction.
                // Overlapping charts therefore cannot pop at dominant-axis seams.
                let center_uv=abs(center/cells);
                let chart_weight=1.0-smoothstep(0.92,1.12,max(center_uv.x,center_uv.y));
                let radius=0.055+0.045*hash(seed+19.0);let width2=radius*radius+pixel*pixel*0.18;
                let delta=p-center;let light=exp(-dot(delta,delta)/(2.0*width2))*radius*radius/width2;
                let tint=mix(vec3<f32>(1.0,0.73,0.48),vec3<f32>(0.65,0.79,1.0),hash(seed+5.0));
                color+=tint*light*chart_weight*(80.0+900.0*pow(hash(seed+11.0),5.0));
            }
        }}
    }
    return color;
}
fn stars(direction:vec3<f32>,footprint:f32)->vec3<f32> {
    let d=normalize(direction);let a=abs(d);let dominant=max(a.x,max(a.y,a.z));
    var color=vec3<f32>(0.0);
    // Three potentially visible hemisphere charts, each bounded to 18 cells.
    if a.x>dominant/1.16 {color+=star_chart(d.yz/a.x,select(0.0,1.0,d.x>0.0),footprint/a.x);}
    if a.y>dominant/1.16 {color+=star_chart(d.xz/a.y,select(2.0,3.0,d.y>0.0),footprint/a.y);}
    if a.z>dominant/1.16 {color+=star_chart(d.xy/a.z,select(4.0,5.0,d.z>0.0),footprint/a.z);}
    let angular2=dot(d.xy,d.xy);let width2=0.000008+footprint*footprint*0.18;
    if d.z<0.0 { color+=vec3<f32>(1.0,0.91,0.8)*2400.0*exp(-angular2/(2.0*width2))*0.000008/width2; }
    return color;
}

// Positive disk angular momentum follows its plane normal. p_t=+1 for
// backward rays, so emitted frequency = u^t(1 + Omega Lz).
fn disk_radiation(r:f32,lz:f32,observer_f:f32,spin_a:f32)->vec3<f32> {
    let rs=s.geometry.x;let mass=0.5*rs;let inner=s.geometry.y;let outer=s.geometry.z;
    let flux=pow(inner/r,3.0)*max(0.0,1.0-sqrt(inner/r));
    let omega=sqrt(mass)/(pow(r,1.5)+spin_a*sqrt(mass));
    let ut=(1.0+spin_a*sqrt(mass)/pow(r,1.5)) /
        sqrt(max(0.00001,1.0-3.0*mass/r+2.0*spin_a*sqrt(mass)/pow(r,1.5)));
    let doppler=1.0/max(0.01,1.0+omega*lz);
    let gravity=1.0/(sqrt(max(0.00001,observer_f))*ut);
    var shift=1.0;var gain=1.0;
    if s.features.w>0.5 {shift*=doppler;}
    if s.relativity.x>0.5 {gain*=pow(doppler,4.0);}
    if s.relativity.y>0.5 {shift*=gravity;gain*=pow(gravity,4.0);}
    let temperature=s.model.z*pow(max(flux,1e-12),0.25)*shift;
    let tint=thermal_rgb(temperature);
    let edge=smoothstep(inner,inner+0.15*rs,r)*(1.0-smoothstep(outer-0.5*rs,outer,r));
    return tint*flux*80000.0*s.disk.w*gain*edge;
}
fn disk_emission(p:vec3<f32>,v:vec3<f32>)->vec3<f32> {
    let observer_f=1.0-s.geometry.x/length(s.camera.xyz-s.center.xyz);
    return disk_radiation(length(p),dot(s.disk.xyz,cross(p,v)),observer_f,0.0);
}
// Hermite dense output: refine thin-disk events without extra geodesic steps.
fn curve(p:vec3<f32>,q:vec3<f32>,v:vec3<f32>,w:vec3<f32>,h:f32,t:f32)->vec3<f32> {
    let t2=t*t;let t3=t2*t;
    return (2.0*t3-3.0*t2+1.0)*p+(t3-2.0*t2+t)*h*v+
        (-2.0*t3+3.0*t2)*q+(t3-t2)*h*w;
}
fn curve_t(p:vec3<f32>,q:vec3<f32>,v:vec3<f32>,w:vec3<f32>,h:f32,t:f32)->vec3<f32> {
    return ((6.0*t*t-6.0*t)*p+(-6.0*t*t+6.0*t)*q)/h+
        (3.0*t*t-4.0*t+1.0)*v+(3.0*t*t-2.0*t)*w;
}
fn disk_event(p:vec3<f32>,q:vec3<f32>,v:vec3<f32>,w:vec3<f32>,h:f32)->f32 {
    var lo=0.0;var hi=1.0;let sign=dot(p,s.disk.xyz);
    for(var i=0u;i<10u;i++) {
        let mid=(lo+hi)*0.5;
        if dot(curve(p,q,v,w,h,mid),s.disk.xyz)*sign>0.0 {lo=mid;} else {hi=mid;}
    }
    return (lo+hi)*0.5;
}

fn acceleration(p:vec3<f32>,l2:f32)->vec3<f32> {
    let r2=max(dot(p,p),0.01);return -1.5*s.geometry.x*l2*p/(r2*r2*sqrt(r2));
}

// category: 0 captured, 1 escaped, 2 disk hit (even if subsequently captured), 3 budget exhausted.
struct RayResult { direction:vec3<f32>, emission:vec3<f32>, escaped:f32, exhausted:f32, category:u32, error:f32, }
fn straight(origin:vec3<f32>,direction:vec3<f32>)->RayResult {
    let rs=s.geometry.x;let b=dot(origin,direction);let c=dot(origin,origin)-rs*rs;
    let discriminant=b*b-c;var capture=1e30;
    if discriminant>=0.0 && b<0.0 {capture=-b-sqrt(discriminant);}
    var light=vec3<f32>(0.0);var transmission=1.0;
    var category=select(1u,0u,capture<1e29);
    let denominator=dot(direction,s.disk.xyz);
    if s.features.y>0.5 && abs(denominator)>0.000001 {
        let t=-dot(origin,s.disk.xyz)/denominator;let hit=origin+direction*t;let r=length(hit);
        if t>0.0 && t<capture && r>s.geometry.y && r<s.geometry.z {
            light=disk_emission(hit,direction)*0.96;transmission=0.04;category=2u;
        }
    }
    return RayResult(direction,light,select(transmission,0.0,capture<1e29),0.0,category,0.0);
}
fn trace(origin:vec3<f32>,direction:vec3<f32>)->RayResult {
    if s.features.x<0.5 {return straight(origin,direction);}
    if s.model.y>0.5 {return trace_kerr(origin,direction);}
    let rs=s.geometry.x;var p=origin;var v=direction;var light=vec3<f32>(0.0);
    var error=0.0;var escaped=0.0;var travel=0.0;var transmission=1.0;var exhausted=1.0;var disk_hit=false;
    let r0=length(p);let radial=p/max(r0,0.0001);
    // Static observer tetrad → E=1 affine velocity in areal Cartesian coordinates.
    if s.features.x>0.5 {
        let radial_part=dot(direction,radial);
        v=radial*radial_part+(direction-radial*radial_part)/sqrt(max(0.0001,1.0-rs/r0));
    }
    let angular=cross(p,v);let l2=dot(angular,angular);
    let escape_radius=max(r0*1.15,s.geometry.z*3.0);
    for(var i=0u;i<768u;i++) {
        if i>=u32(s.integration.x) || travel>s.geometry.w {break;}
        let r=length(p);
        if r<=rs*1.015 {exhausted=0.0;break;}
        if r>escape_radius && dot(p,v)>0.0 {escaped=1.0;exhausted=0.0;break;}
        // Velocity Verlet; tighten near the photon sphere and disk, grow in vacuum.
        var h=s.integration.y*min(r*0.16,r*r/(sqrt(l2)+0.01)*0.12);
        // Large distant steps allow astronomical starts; near-horizon local
        // velocities need an additional spatial-displacement bound.
        h=clamp(h,rs*0.018,max(rs*12.0,r*0.15));
        h=min(h,r*0.12/max(length(v),1.0));
        let old=p;let old_v=v;var acc=vec3<f32>(0.0);
        if s.features.x>0.5 {acc=acceleration(p,l2);}
        p+=v*h+0.5*acc*h*h;
        var next_acc=vec3<f32>(0.0);
        if s.features.x>0.5 {next_acc=acceleration(p,l2);}
        v+=0.5*(acc+next_acc)*h;travel+=h;
        let nr=length(p);let vr=dot(p,v)/nr;
        error=max(error,abs(vr*vr+(1.0-rs/nr)*l2/(nr*nr)-1.0)/max(1.0,dot(v,v)));
        let old_plane=dot(old,s.disk.xyz);let new_plane=dot(p,s.disk.xyz);
        if s.features.y>0.5 && old_plane*new_plane<=0.0 && abs(new_plane-old_plane)>0.000001 {
            let t=disk_event(old,p,old_v,v,h);
            let hit=curve(old,p,old_v,v,h,t);let hit_v=curve_t(old,p,old_v,v,h,t);let dr=length(hit);
            if dr>s.geometry.y && dr<s.geometry.z {
                light+=transmission*disk_emission(hit,hit_v)*0.96;transmission*=0.04;
                disk_hit=true;
                if transmission<0.002 {exhausted=0.0;break;}
            }
        }
    }
    let category=select(select(select(0u,1u,escaped>0.5),3u,exhausted>0.5),2u,disk_hit);
    return RayResult(normalize(v),light,escaped*transmission,exhausted,category,error);
}

fn local(p:vec3<f32>)->vec3<f32> {return vec3<f32>(dot(s.axes_x.xyz,p),dot(s.axes_y.xyz,p),dot(s.axes_z.xyz,p));}
fn world(p:vec3<f32>)->vec3<f32> {return s.axes_x.xyz*p.x+s.axes_y.xyz*p.y+s.axes_z.xyz*p.z;}
fn trace_kerr(origin:vec3<f32>,direction:vec3<f32>)->RayResult {
    let mass=0.5*s.geometry.x;let a=s.model.x*mass;
    let horizon=mass*(1.0+sqrt(1.0-s.model.x*s.model.x));
    let x0=local(origin);let d=local(direction);
    var y=State(x0,initial_momentum(x0,d,mass,a));
    let observer_f=1.0-2.0*metric(x0,mass,a).H;
    let escape=max(length(origin)*1.15,s.geometry.z*3.0);
    let lz0=y.x.x*y.p.y-y.x.y*y.p.x;
    var travel=0.0;var exhausted=1.0;var escaped=0.0;var error=0.0;
    var light=vec3<f32>(0.0);var transmission=1.0;var hit_disk=false;
    for(var i=0u;i<768u;i++) {
        if i>=u32(s.integration.x) || travel>s.geometry.w {break;}
        let g=metric(y.x,mass,a);let vel=derivative(y,mass,a).x;
        if g.r<=1.015*horizon {exhausted=0.0;break;}
        if length(y.x)>escape && dot(y.x,vel)>0.0 {exhausted=0.0;escaped=1.0;break;}
        var h=s.integration.y*min(g.r*0.16,g.r*g.r/(length(cross(y.x,vel))+0.01)*0.12);
        h=clamp(h,s.geometry.x*0.008,max(12.0*s.geometry.x,g.r*0.15));
        let refinement=min(1.0,s.integration.y/0.85);
        h=min(h,0.10*refinement*g.r/max(1.0,length(vel)));
        // Avoid RK sub-stages leaping through the ring/horizon singularity.
        // High quality also refines the safety caps. Near-extremal horizons
        // require smaller inward steps; a fixed cap defeated quality scaling.
        let horizon_refinement=refinement*select(1.0,max(0.5,sqrt(1.0-s.model.x*s.model.x)),refinement<1.0);
        if dot(g.gr,vel)<0.0 {h=min(h,0.12*horizon_refinement*max(g.r-horizon,0.002*s.geometry.x)/max(-dot(g.gr,vel),0.01));}
        let old=y;let old_v=vel;y=rk4(y,h,mass,a);travel+=h;
        if !all(abs(y.x)<vec3<f32>(1e20)) || !all(abs(y.p)<vec3<f32>(1e20)) {break;}
        error=max(error,max(constraint(y,mass,a),abs(y.x.x*y.p.y-y.x.y*y.p.x-lz0)/max(1.0,abs(lz0))));
        if s.features.y>0.5 && old.x.z*y.x.z<0.0 && abs(y.x.z-old.x.z)>1e-7 {
            var lo=0.0;var hi=1.0;let next_v=derivative(y,mass,a).x;
            for(var j=0u;j<10u;j++) {
                let t=(lo+hi)*0.5;
                if curve(old.x,y.x,old_v,next_v,h,t).z*old.x.z>0.0 {lo=t;} else {hi=t;}
            }
            let event=rk4(old,h*(lo+hi)*0.5,mass,a);let r=metric(event.x,mass,a).r;
            if r>s.geometry.y && r<s.geometry.z {
                let lz=event.x.x*event.p.y-event.x.y*event.p.x;
                light+=transmission*disk_radiation(r,lz,observer_f,a)*0.96;
                transmission*=0.04;hit_disk=true;
                if transmission<0.002 {exhausted=0.0;break;}
            }
        }
    }
    let category=select(select(select(0u,1u,escaped>0.5),3u,exhausted>0.5),2u,hit_disk);
    return RayResult(normalize(world(derivative(y,mass,a).x)),light,escaped*transmission,exhausted,category,error);
}

// Fixed celestial longitude/latitude grid. Sample the escaping direction,
// never the screen UV: the same numerical solver lenses it.
fn celestial_grid(direction:vec3<f32>,footprint:f32)->vec3<f32> {
    let d=normalize(direction);let spacing=0.261799388;
    let longitude=atan2(d.x,-d.z);let latitude=asin(clamp(d.y,-1.0,1.0));
    let lon_distance=abs(fract(longitude/spacing+0.5)-0.5)*spacing*sqrt(max(0.01,1.0-d.y*d.y));
    let lat_distance=abs(fract(latitude/spacing+0.5)-0.5)*spacing;
    let width=max(0.001,footprint*0.55);
    let line=1.0-smoothstep(width*0.35,width*1.35,min(lon_distance,lat_distance));
    return vec3<f32>(0.23,0.66,1.0)*line*180.0;
}

@fragment fn fragment(in:FullscreenVertexOutput)->@location(0) vec4<f32> {
    let pixel=vec2<i32>(in.position.xy);let foreground=textureLoad(scene,pixel,0);
    let ndc=in.uv*vec2<f32>(2.0,-2.0)+vec2<f32>(-1.0,1.0);
    let ray=normalize(s.forward.xyz+s.right.xyz*ndc.x*s.right.w+s.up.xyz*ndc.y*s.up.w);
    // All lanes trace consistently so directional derivatives remain usable.
    // Foreground rays are short-circuited without sampling a fake backdrop.
    var result=RayResult(ray,vec3<f32>(0.0),0.0,0.0,1u,0.0);
    let z=textureLoad(depth,pixel,0);
    if z==0.0 {result=trace(s.camera.xyz-s.center.xyz,ray);}
    let footprint=max(length(dpdx(result.direction)),length(dpdy(result.direction)));
    var background=result.emission;
    if s.features.z>0.5 {
        if s.sky.y!=1u {background+=result.escaped*stars(result.direction,footprint);}
        if s.sky.y!=0u {background+=result.escaped*celestial_grid(result.direction,footprint);}
    }
    if z>0.0 {return foreground;}
    if s.model.w>0.5 {
        // Explicit error bands, so ordinary f32 rounding stays legibly green.
        var color=vec3<f32>(0.015,0.22,0.05);
        if result.error>=1e-4 {color=vec3<f32>(0.4,0.22,0.005);}
        if result.error>=1e-3 {color=vec3<f32>(1.0,0.07,0.005);}
        if result.error>=1e-2 {color=vec3<f32>(4.0,0.005,0.005);}
        if result.exhausted>0.5 {color=vec3<f32>(4.0,0.0,3.0);}
        return vec4<f32>(color,1.0);
    }
    if s.sky.z>0u {
        var color=vec3<f32>(0.015);
        if result.category==1u {color=vec3<f32>(0.06,0.5,0.22);}
        if result.category==2u {color=vec3<f32>(2.0,0.55,0.04);}
        if result.category==3u {color=vec3<f32>(4.0,0.0,3.0);}
        return vec4<f32>(color,1.0);
    }
    if s.relativity.z>0.5 && result.exhausted>0.5 {return vec4<f32>(4.0,0.0,3.0,1.0);}
    return vec4<f32>(background*s.integration.z,1.0);
}
