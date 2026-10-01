// Ingoing Cartesian Kerr-Schild, signature -+++, backward p_t = +1.
// Original Hamiltonian implementation; equations documented in MATH.md.
struct Metric {
    r:f32, H:f32, l:vec3<f32>, gr:vec3<f32>, gh:vec3<f32>,
    gx:vec3<f32>, gy:vec3<f32>, gz:vec3<f32>,
}
struct State { x:vec3<f32>, p:vec3<f32>, }
fn metric(x:vec3<f32>,mass:f32,a:f32)->Metric {
    let a2=a*a;let u=dot(x,x)-a2;
    let disc=max(sqrt(u*u+4.0*a2*x.z*x.z),1e-12);
    let r=sqrt(max(0.5*(u+disc),1e-12));let r2=r*r;let b=r2+a2;
    let gr=vec3<f32>(x.x*r,x.y*r,x.z*b/r)/disc;
    let den=r2*r2+a2*x.z*x.z;let H=mass*r*r2/den;
    let l=vec3<f32>((r*x.x+a*x.y)/b,(r*x.y-a*x.x)/b,x.z/r);
    let gh=H*(3.0*gr/r-(4.0*r*r2*gr+vec3<f32>(0.0,0.0,2.0*a2*x.z))/den);
    let gx=(x.x*gr+vec3<f32>(r,a,0.0)-2.0*r*l.x*gr)/b;
    let gy=(x.y*gr+vec3<f32>(-a,r,0.0)-2.0*r*l.y*gr)/b;
    let gz=vec3<f32>(0.0,0.0,1.0)/r-x.z*gr/r2;
    return Metric(r,H,l,gr,gh,gx,gy,gz);
}
fn initial_momentum(x:vec3<f32>,d:vec3<f32>,mass:f32,a:f32)->vec3<f32> {
    let g=metric(x,mass,a);let f=max(1.0-2.0*g.H,1e-5);
    return d/sqrt(f)+g.l*((1.0/f-1.0/sqrt(f))*dot(g.l,d)-2.0*g.H/f);
}
fn derivative(y:State,mass:f32,a:f32)->State {
    let g=metric(y.x,mass,a);let q=-1.0+dot(g.l,y.p);
    return State(y.p-2.0*g.H*g.l*q,
        g.gh*q*q+2.0*g.H*q*(y.p.x*g.gx+y.p.y*g.gy+y.p.z*g.gz));
}
fn add(y:State,d:State,h:f32)->State {return State(y.x+h*d.x,y.p+h*d.p);}
fn rk4(y:State,h:f32,mass:f32,a:f32)->State {
    let d1=derivative(y,mass,a);let d2=derivative(add(y,d1,h*0.5),mass,a);
    let d3=derivative(add(y,d2,h*0.5),mass,a);let d4=derivative(add(y,d3,h),mass,a);
    return State(y.x+h*(d1.x+2.0*d2.x+2.0*d3.x+d4.x)/6.0,
        y.p+h*(d1.p+2.0*d2.p+2.0*d3.p+d4.p)/6.0);
}
fn constraint(y:State,mass:f32,a:f32)->f32 {
    let g=metric(y.x,mass,a);let q=-1.0+dot(g.l,y.p);
    // Scale-normalized null residual; p_t fixed by construction.
    return abs(dot(y.p,y.p)-1.0-2.0*g.H*q*q)/max(1.0,dot(y.p,y.p));
}
