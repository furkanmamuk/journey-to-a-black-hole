# Rendering architecture

The lab has three independent concerns: an ordinary scene/camera, astronomical ray transport, and intentional presentation quantization. It uses Bevy's custom rendering facilities without modifying Bevy itself.

## Observer and scene

The 6-DOF camera supplies world position, orientation and projection. Presets and the guided Journey only move that transform. Numerical safety keeps the observer outside the supported static-observer boundary; translational observer velocity is not included in relativistic aberration.

A smooth procedural engineering probe supplies metallic, ceramic, thermal-coating and small emissive surfaces. Light comes from explicit disk quadrature samples, an optional distant stellar directional light and equipment emission. Ambient illumination and clear colour are black. The foreground uses conventional PBR depth/projection, not curved-ray visibility.

## Astronomical shader

For each uncovered physical pixel, reconstruct a camera ray in world space and convert its direction to the local static-observer initialization. The reference Schwarzschild path integrates a spatial null equation with velocity Verlet. Kerr is a separate Cartesian Kerr–Schild Hamiltonian/RK4 implementation with analytic derivatives.

Both use distance-dependent steps, bounded work, early capture and outgoing escape termination. The disk is an oriented world-space annulus. Dense curve/event evaluation detects plane intersections between integration samples. The finite emitting surface supports at most two intersections before transmission becomes negligible.

Escaped directions sample sparse deterministic stars from overlapping directional charts. Sources are tied to celestial direction rather than screen UV. A fixed longitude/latitude grid can replace or accompany stars, making distortions easier to inspect.

The disk temperature follows a simplified no-torque radial flux profile. A finite Planck/CIE integral produces linear RGB; orbital transfer modifies observed temperature and bolometric radiance. These are physically motivated approximations with artistic HDR normalization. [MATH.md](MATH.md) documents equations, sign conventions, coordinate mappings and limits.

## Render ordering

| Stage | Purpose |
|---|---|
| HDR PBR and depth | Shade the ordinary craft and establish foreground coverage |
| Optional TAA | Stabilize foreground PBR geometry |
| Geodesic HDR background | Trace only uncovered celestial pixels; preserve foreground |
| Bloom, exposure, ACES | Process bright disk and faint stars in the same HDR composition |
| Area reduction | Stable weighted downsample into the presentation grid |
| Tonal/dither treatment | Configurable colour levels and stable ordered dither |
| Nearest expansion | Deliberate spatial cells, with native-resolution UI afterward |

PBR runs first so depth can skip covered ray work. The celestial composition is still HDR and happens before exposure/tonemapping. This ordering is an implementation choice; it does not allow foreground objects to be gravitationally occluded. F7 bypasses presentation quantization while preserving the underlying physical scene and framing.

The bounded 9×9 reduction covers fractional ratios up to eight. GPU buffers/textures are reused, and profiling uses asynchronous diagnostic readback. The inspector performs separate f64 CPU traces at 10 Hz only while visible; it does not synchronize the GPU or establish bit-for-bit agreement with f32 rays.

## Source map

| Source | Responsibility |
|---|---|
| `src/main.rs` | Application wiring and portable asset lookup |
| `src/camera.rs`, `src/journey.rs` | Free observer, normal presets, guided camera path |
| `src/settings.rs`, `src/configuration.rs` | Runtime controls, CLI and reproducible scenes |
| `src/scene.rs`, `src/materials.rs` | Procedural probe and explicit illumination |
| `assets/shaders/black_hole.wgsl` | Per-pixel reference/spinning tracing, disk, celestial sources |
| `assets/shaders/kerr.wgsl` | Self-contained spinning metric, Hamiltonian and RK4 |
| `assets/shaders/spectrum.wgsl` | Sampled Planck/CIE spectrum |
| `src/black_hole/pipeline.rs`, `uniforms.rs` | Custom HDR pass and extracted settings |
| `src/black_hole/integration.rs`, `kerr.rs` | CPU inspector/reference paths |
| `src/black_hole/relativity.rs` | Horizon/ISCO and radiation-transfer references |
| `src/black_hole/debug_rays.rs` | Log-radius trajectory diagram and outcomes |
| `src/quantization/`, `assets/shaders/quantize.wgsl` | Independent reduction/presentation pass |
| `src/profiling.rs`, `src/debug_ui.rs` | CPU/GPU metrics, controls and runtime validation |
| `src/physics_reference.rs` | Independent reference physics tests |

## Extending it

Priorities are better local/adaptive error control and reference shadow comparisons, lens-aware temporal history, curved-ray geometry intersections, moving-observer tetrads and a more complete disk radiative model. A thick turbulent or GRMHD disk would also need volumetric/plasma transfer and retarded-time state. These should stay separate from spacecraft materials and presentation quantization.
