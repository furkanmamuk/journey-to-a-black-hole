# Relativistic model and numerical boundaries

This lab uses G=c=1, rs=2M, and distances in scene units. The shader has no camera-preset or journey-stage input. Choosing a view changes only the observer transform. Schwarzschild remains the default reference; T switches to the optional Kerr calculation. This is numerical null-ray tracing plus an analytic emitting disk, not a fluid or GRMHD simulation.

## Schwarzschild reference

The spatial null equation in areal Cartesian coordinates, with conserved energy normalized to E=1, is

```
x'' = -(3/2) rs L² x / r⁵
L = x × x'
r = |x|
```

The initial direction is a local static observer's unit direction d. Set er=x/r and μ=d·er:

```
x' = μ er + (d - μ er) / sqrt(1-rs/r_observer)
```

This is a tetrad conversion, not arbitrary Euclidean initialization. The null constraint is `v_radial² + (1-rs/r)L²/r² = 1`. Velocity Verlet preserves angular momentum well, but variable steps do not guarantee energy conservation. The M overlay shows the maximum normalized null residual encountered along each pixel ray, with explicit green (<1e-4), amber (1e-4..1e-3), red (>=1e-3, brighter >=1e-2) and magenta (exhausted) bands. Captured/escaped/disk/budget classification remains available separately with B; F11 exposes exhausted rays.

## Optional Kerr–Schild Hamiltonian

The spin axis follows the disk normal. `chi=a/M` is signed relative to positive disk angular momentum, bounded to [-0.98,0.98]. Positive chi is a prograde disk; negative chi is retrograde. Disk tilt therefore also tilts the hole's spin axis. Misaligned/precessing disks are outside this first Kerr experiment.

Use a right-handed Cartesian frame with spin along local Z. Ingoing Kerr–Schild coordinates avoid the Boyer–Lindquist coordinate singularity at the horizon. The metric form and radius relation follow the equations given in [Pelle et al., Skylight (2022), section 4.1](https://arxiv.org/abs/2206.06429). The code below is an original Hamiltonian derivation from that metric, not copied from their renderer.

```
g_mu_nu = eta_mu_nu + 2H l_mu l_nu       signature (-,+,+,+)
r² = (u + sqrt(u²+4a²z²))/2             u = x²+y²+z²-a²
H = M r³/(r⁴+a²z²)
l_mu = (1, (rx+ay)/(r²+a²), (ry-ax)/(r²+a²), z/r)
```

The backward ray has constant canonical `p_t=+1`. With spatial covector p and `q=-1+l·p`, the null Hamiltonian and six integrated variables are

```
Hamiltonian = (|p|²-1)/2 - H q² = 0
x' = p - 2H l q
p' = grad(H) q² + 2H q sum_i[p_i grad(l_i)]
```

All spatial derivatives are analytic. Four right-hand-side evaluations per RK4 step advance x and p together. There is no finite-difference metric sampling in the GPU loop, no geodesic texture, no rotation of a Schwarzschild shadow to imitate frame dragging. In particular, total spatial angular momentum is not conserved by Kerr. The axial canonical `Lz=x p_y-y p_x` is conserved and is the inspector's momentum diagnostic.

For reproducibility, the derivatives used are

```
D = sqrt(u²+4a²z²), B=r²+a², A=r⁴+a²z²
grad(r) = (xr, yr, zB/r) / D
grad(H) = H [3 grad(r)/r - (4r³ grad(r)+2a²z e_z)/A]
grad(l_x) = [x grad(r)+(r,a,0)-2r l_x grad(r)]/B
grad(l_y) = [y grad(r)+(-a,r,0)-2r l_y grad(r)]/B
grad(l_z) = e_z/r-z grad(r)/r²
```

A static observer requires `f=1-2H>0`. Given local sky direction d, its canonical initialization is

```
p = d/sqrt(f) + l [(1/f-1/sqrt(f))(l·d)-2H/f]
```

The tests check that this is null, and that its raised spatial velocity at a=0 reduces to the Schwarzschild tetrad above. The camera is conservatively kept outside the equatorial stationary-limit surface: world distance at least `sqrt((1.08rs)²+a²)`. Journey termination is slightly farther outside this bound. This is a supported external static observer, not a freely falling observer or a simulation of surviving near a horizon. The moving camera currently changes position/orientation without including its translational velocity in an observer Lorentz boost.

The outer horizon is `r+=M(1+sqrt(1-chi²))`. Capture terminates at 1.015 r+. A Kerr horizon is oblate in these Cartesian coordinates; the inspector draws that ellipsoid. Its blue rings are the equatorial prograde/retrograde photon-orbit references, not a spherical photon shell. The full Kerr photon region is more complicated.

U toggles following the circular-orbit ISCO. The signed-spin ISCO calculation is the standard expression associated with [Bardeen, Press & Teukolsky (1972)](https://www.osti.gov/biblio/4585183). Known values at chi=0,+0.8,-0.8 are tested. In equatorial Kerr–Schild coordinates a disk radius r has Cartesian in-plane radius `sqrt(r²+a²)`. The disk, inspector and illumination sample positions account for that mapping. Editing the inner radius disables ISCO following and still enforces its lower stability bound. Old version-1 scene files without `isco_lock` retain their manually specified inner radius.

## Radiation transfer and spectrum

Positive disk angular momentum points along the normal. For equatorial circular emitters,

```
Omega = sqrt(M)/(r^(3/2)+a sqrt(M))
u^t = [1+a sqrt(M)/r^(3/2)] / sqrt[1-3M/r+2a sqrt(M)/r^(3/2)]
g = observed/emitted frequency = 1/[sqrt(f_observer) u^t (1+Omega Lz)]
```

The plus sign uses the backward ray's p_t=+1 convention. Factor the transfer into `D=1/(1+Omega Lz)` and `R=1/(sqrt(f_observer) u^t)`. Z applies D to the temperature, X applies D^4 to bolometric intensity, C applies R to temperature and R^4 to intensity. With all enabled the transfer is the complete circular-emitter/static-observer g and g^4 for this model. R includes gravitational **and transverse Doppler** shift. The separated switches are comparison experiments; disabling one breaks the complete physical transfer on purpose. The old D^3 intensity approximation was replaced: D^3 describes monochromatic specific intensity, whereas this disk's flux is bolometric.

The radial emissivity still uses the simplified Newtonian no-torque form `F ∝ (r_in/r)^3 (1-sqrt(r_in/r))`, with `T=T_scale F^(1/4)` and a configurable default scale of 18000 K. It is not the full relativistic Novikov–Thorne flux, an inferred accretion rate, or a mass-calibrated disk temperature. Artificial sine brightness bands have been removed.

Observed temperature `gT` feeds a 41-sample Planck integral at 10 nm spacing from 380 to 780 nm. XYZ uses the [CIE 1931 2-degree colour matching dataset](https://cie.co.at/datatable/cie-1931-colour-matching-functions-2-degree-observer), then converts to linear sRGB. Dividing the colour integral by T^4 retains the change in visible fraction per bolometric flux; g^4 intensity and shifted colour therefore avoid counting the spectral energy shift twice. Negative sRGB components are gamut-clipped. Quadrature is finite, not a spectral renderer. CIE data are separately attributed/licensed in assets/CIE-NOTICE.txt. HDR normalization remains an artistic exposure calibration.

Thin-disk intersections use Hermite dense output with ten bisection iterations. Kerr re-integrates one fractional RK4 step to evaluate the canonical state at the event. Work is bounded, with at most two emitting intersections before the approximate surface transmission becomes negligible. The finite transmission (0.04), radial edge taper and zero thickness are approximations. Exact coplanar rays are singular for this model; slightly elevate the observer.

## Integration quality and failure visibility

Both paths have a hard maximum of 768 steps, configurable affine-distance and step budgets, early capture and outgoing escape termination. Cheap/balanced/high select 128/256/768 steps and different local step scales. The step bound includes displacement and distance to the Kerr horizon. A failed conservation test on the initial horizon step bound led to reducing its allowed approach from 0.35 to 0.12 of the remaining horizon distance. A later GPU diagnostic exposed that fixed safety caps prevented high quality from refining the strong-field error. Kerr now scales its displacement cap by `min(1, step_scale/0.85)`. High quality multiplies its horizon cap by this factor and `max(0.5,sqrt(1-chi²))`; the latter follows shrinking horizon separation but has a bounded practical floor. The initial unfloored variant exhausted many near-extremal captured rays and was rejected. Raising the high preset from 512 to 768 preserves a bounded budget while allowing these smaller steps to finish. No momentum renormalization conceals integration error.

The GPU uses f32. The independent Kerr inspector/reference uses f64, identical equations and step bounds; it is not claimed to reproduce GPU rounding bit-for-bit. Tests cover analytic gradients against finite differences, null tetrads, the Schwarzschild capture limit, axial momentum, spin-reflection symmetry, RK4 convergence and known ISCO values. M measures GPU constraint error; CPU residual tests alone cannot certify GPU pixels.

An escaped ray samples a finite-radius approximation to an infinitely distant sky. Increasing maximum distance/quality can change narrow higher-order images. Budget exhaustion is black in the normal image and explicit magenta under F11. Near the critical capture boundary, subpixel topology can change even when local residuals are small. The reference alignment star has finite angular width. Stars have empirical colours/radiance and do not yet receive observer gravitational spectral transfer. No volumetric plasma, synchrotron transfer, polarization, retarded emission, turbulence dynamics or returning disk irradiation is simulated.

## Whole render path and remaining architecture

The pipeline remains PBR+depth -> optional foreground TAA -> geodesic HDR background -> bloom/auto histogram -> ACES -> box reduction -> tonal/dither -> nearest expansion -> UI/display. A 9x9 bounded reduction covers ratios up to 8 even when fractional edges span nine texels. Overlapping directional star charts fade each fixed star by its own chart position, avoiding dominant-axis chart changes. There is no time-dependent star noise.

The foreground craft is ordinary Bevy PBR. It is neither curved-ray lensed nor gravitationally occluded; quadrature disk lights are not a GR light-transport solution. These are the largest architectural limitations for flying through/behind the central object. A proper next renderer would intersect bounded foreground geometry along curved ray segments and share visibility/radiometric transport. A depth-only distortion trick cannot provide missing geometry. TAA currently stabilizes foreground geometry only; the astronomical background is filtered spatially. Motion accumulation needs separate geodesic direction/event reprojection and explicit disocclusion rules.

A research-quality Kerr extension now needs tighter/adaptive error control, Carter-constant/reference shadow comparisons, moving-observer tetrads, Novikov–Thorne or GRMHD emission, thickness/plasma transfer and retarded-time state. The current Hamiltonian module supplies the spinning geometry; those refinements remain separate and explicit.
