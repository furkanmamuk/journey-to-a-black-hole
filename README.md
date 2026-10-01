# Journey to a Black Hole

An interactive **Rust / Bevy 0.19.1** rendering experiment: fly around a black hole, see stars and an accretion disk bend with your viewpoint, switch between Schwarzschild and spinning Kerr geometry, and inspect the rays behind the image.

Continuous physical lighting is rendered underneath an optional, deliberately quantized presentation. Smooth engineering geometry, metallic/ceramic/coated surfaces, HDR emission and deeply black vacuum share the same scene. There is no ambient space fog, painted nebula skybox or baked lensing animation.

![Kerr lensing and the foreground material probe](docs/images/probe.png)

This is an exploration demo and rendering laboratory. Its simulation follows bounded numerical null rays and an analytic disk; it does not simulate plasma dynamics or black-hole interiors. The guided flight moves the same freely controllable camera used in exploration.

## Download and play

Download the **Windows x64 ZIP** from [Releases](https://github.com/furkanmamuk/journey-to-a-black-hole/releases). Extract the complete folder and run:

- **Start Journey.cmd** — guided flight; movement or mouse-look immediately takes over.
- **Free Exploration.cmd** — free flight from the distant view.
- **Spinning Exploration.cmd** — high-quality Kerr comparison.

Keep `assets/` beside the executable. Rust is not needed for the downloadable version. The tested graphics backend is Windows/Vulkan on an NVIDIA RTX 4060; other hardware/backends are unverified. Browser/WASM support is planned and **is not a playable build in this release**. See [platform status](docs/PLATFORMS.md).

## First flight

| Input | Action |
|---|---|
| Tab or right mouse | Capture/release mouse; Esc releases |
| WASD, Space / Ctrl | Translate in camera-local axes |
| Mouse, Q / E | Look and roll |
| Shift / Alt, mouse wheel | Fast/precision movement, adjust speed |
| 1–7 | Distant, edge-on, above, close, aligned, probe and near-limit views |
| J / Enter | Start/pause Journey / restart |
| T | Compare Schwarzschild and Kerr |
| F7 | Compare continuous and quantized presentation |
| P / H | Parameters/help / clean display |
| I / G | Ray inspector / directional celestial grid |
| M / F11 | Numerical constraint errors / unfinished ray budgets |

The nearby probe is a material/scale reference. It uses ordinary foreground PBR projection and is **not gravitationally lensed or occluded**. Near-limit inward-facing views can become entirely black because all visible rays are captured. The camera stays outside its supported static-observer boundary.

See [all controls and configurations](docs/CONTROLS.md). Try view 2, move sideways, disable the disk with F2, and enable the sky grid with G to isolate dynamic lensing. F7 changes presentation while preserving physical geometry and lighting.

## How the image is produced

Each uncovered astronomical pixel reconstructs a world-space observer ray and follows a bounded GPU trajectory. The ray can escape to deterministic directional stars, intersect the world-space disk, be captured, or exhaust its budget. Camera preset names and Journey progress do not reach the shader.

Schwarzschild uses the spatial null-orbit equation and a local static-observer initialization. Optional Kerr uses a separate Kerr–Schild Hamiltonian with analytic metric derivatives and RK4 integration. Spin affects frame dragging, the horizon, stable inner disk radius and emitter frequency transfer.

The disk uses a radial temperature profile, a sampled Planck/CIE colour integral and circular-emitter/static-observer redshift and bolometric beaming. It is a thin analytic emitter rather than a fluid simulation. The probe uses Bevy's standard PBR path and modest procedural material maps.

```text
HDR PBR + depth → optional foreground TAA → depth-masked geodesic background
→ bloom / exposure → ACES → area reduction → tonal quantization / dither
→ nearest expansion → display and UI
```

Defaults are **960×540 HDR → 480×270 presentation**, in a requested 1920×1080 window. Resolutions, exposure, radius, disk orientation, signed spin, quality and individual effects are adjustable. Quality presets allow 128 / 256 / 768 steps; the loop always remains bounded.

- [Rendering architecture and source map](docs/HOW_IT_WORKS.md)
- [Equations, physical assumptions and approximations](docs/MATH.md)
- [Measured performance and verification boundaries](docs/PERFORMANCE.md)
- [Visual findings and remaining limitations](docs/VISUALS.md)

## Build from source

Install Rust **1.95 or newer** and a graphics driver compatible with Bevy's native renderer. Run from the repository root:

```sh
cargo run --release --locked -- --journey
cargo run --release --locked -- --kerr --preset 6
cargo run --release --locked -- --config examples/kerr.cfg
```

For a full-sized native display, add `--fullscreen`. The window/grid adapt with integer-scale letterboxing; Windows may constrain a decorated window to the available desktop area. Source builds on Linux/macOS are not yet verified.

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
```

On Windows, `scripts/package.ps1` builds portable and source ZIPs with licenses and documentation. `scripts/measure.ps1` measures the release renderer; `scripts/review.ps1` captures repeatable views. See [contributor and release notes](CONTRIBUTING.md).

## Scope and license

The project focuses on dynamic lensing, HDR material response and stable quantized presentation. It contains no gameplay, networking, procedural planets or general-purpose simulation framework. Numerical accuracy near critical rays, lens-aware temporal accumulation and curved-ray foreground visibility remain research work.

Original code and documentation are **MIT OR Apache-2.0**. The sampled CIE colour-matching table has a separate **CC BY-SA 4.0** notice in [assets/CIE-NOTICE.txt](assets/CIE-NOTICE.txt). Preserve that attribution when reusing the data. Distributed native builds include dependency license notices. The primary physics/data references are linked in the math documentation.
