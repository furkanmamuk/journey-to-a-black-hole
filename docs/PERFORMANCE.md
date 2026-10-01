# Performance and verification

## Development reference measurements

Measured 30 September 2026 on **RTX 4060 / NVIDIA driver 617.14 / Vulkan / Windows**, release build, **Rust 1.97.0 / Bevy 0.19.1**. Output 1920×1080, HDR 960×540 and grid 480×270 unless specified. Manual EV100 9.7 and bloom enabled. Static cases ran 14 seconds and retained four rolling samples at 6–12 seconds. GPU runs were serial, with no concurrent demo or compilation.

These measurements come from the same renderer before its standalone public naming/documentation cleanup. They are practical short-run observations, not controlled hardware benchmarks, browser timings or a guarantee for other machines. Public-release validation is recorded separately below.

| Case | FPS | Frame ms | Main CPU ms | Black-hole GPU ms | Grid GPU ms | Queued GPU ms |
|---|---:|---:|---:|---:|---:|---:|
| Schwarzschild balanced, probe / 6 | 199.0 | 5.06 | 0.67 | 0.49 | 0.09 | 1.80 |
| Kerr +0.8 balanced, probe / 6 | 198.3 | 5.05 | 0.64 | 1.46 | 0.07 | 2.62 |
| Kerr cheap, 128 steps | 201.5 | 4.97 | 0.60 | 1.46 | 0.07 | 2.64 |
| Kerr high, refined 768-step preset | 202.4 | 4.95 | 0.71 | 2.86 | 0.07 | 4.09 |
| Kerr quantization bypass | 202.2 | 4.96 | 0.66 | 1.47 | 0.00 | 2.57 |
| Kerr foreground TAA + auto exposure | 188.0 | 5.33 | 0.64 | 1.47 | 0.06 | 2.75 |
| Kerr moving seven-view sweep | 202.9 | 4.94 | 0.64 | 1.75 | 0.06 | 2.90 |
| Kerr full 60-second Journey | 199.9 | 5.01 | 0.66 | 1.81 | 0.06 | 2.97 |
| Kerr close / 4 + sky grid + 9-ray inspector | 175.7 | 5.71 | 0.64 | 2.50 | 0.06 | 3.77 |
| Kerr 1333×751 HDR → 167×94 grid | 201.2 | 4.98 | 0.59 | 2.90 | 0.05 | 4.04 |

The moving sweep retained 26 samples over 58 seconds; the full Journey retained 29 over 64 seconds, reached camera distance 1.1632rs for chi=0.8 and continued rendering at the supported static-observer cutoff. The inspector case uses a different camera/sky, so its delta is not pure inspector overhead. The fractional case increases physical pixel count and reduction footprint together.

Kerr's four analytic RHS evaluations per RK4 step and divergent critical rays add meaningful GPU cost. Tightening high-quality caps increased its pass from 2.21 to 2.86 ms in the probe pose. Cheap mode cost almost as much as balanced in this pose while leaving visible unfinished rays, so it is primarily a comparison setting. Observer position and physical pixel count are major scaling factors.

## Public build measurements

The standalone public build was measured again on 1 October 2026 on the same hardware, backend and dimensions. Each frozen Kerr probe view ran 14 seconds; all four rolling samples at 6–12 seconds were retained. These short runs measure different schedules and do not establish a meaningful FPS advantage for high quality.

| Public build case | FPS | Frame ms | Main CPU ms | Black-hole GPU ms | Grid GPU ms | Queued GPU ms |
|---|---:|---:|---:|---:|---:|---:|
| Kerr balanced, 256 steps | 184.6 | 5.47 | 0.68 | 1.57 | 0.06 | 2.74 |
| Kerr high, 768 steps | 189.7 | 5.29 | 0.67 | 2.86 | 0.07 | 4.10 |

The public copy passed formatting, locked compilation and all 18 tests. Its native 86-second validation completed through stage 43 without renderer errors. Both quality captures were visually reviewed at 1920×1080: black vacuum, disk lensing and foreground material highlights remained intact. The similar appearance of these two probe views does not imply equal convergence near critical rays.

## Metric boundaries

Queued GPU time includes shadows, PBR, astronomical tracing, post-processing, presentation-image rendering and UI. It excludes query resolution, screenshot readback, display presentation and compositor latency. Main CPU measures the main ECS schedule rather than all rendering/waiting. Individual pass time is not whole-frame latency. Readback is asynchronous; unavailable timestamp backends show `unavailable`.

Nearly 200 FPS with a shorter queued GPU span reflects other scheduling/presentation work. The values are not interchangeable and should not be extrapolated to browser or mobile hardware.

## Validation coverage

- 18 CPU tests cover independent Schwarzschild capture/weak-field references, Kerr gradients, null initialization, axial momentum, spin/reflection symmetry, RK4 convergence, known ISCO values, transfer, configuration and camera/Journey behavior.
- Native 86-second validation exercises features, resizing with TAA, auto exposure, tilted disks, full-u32 seeds, 9/25-ray inspection, guided/free motion, both spin signs through ±0.98 and fractional resolutions.
- Clean 1920×1080 quantized captures have constant 4×4 output cells at 480×270 presentation. A continuous-output comparison has nonconstant cells while retaining the same physical scene.
- The visual review includes every standard view, moving captures and unquantized error/budget checks. CPU f64 tests do not certify arbitrary f32 GPU rays; critical images can remain unfinished at 768 steps.

The public release must additionally be built from its tracked source, packaged with shaders/licenses, extracted and executed independently. The GitHub Release notes record the checks actually performed for that artifact. Hosted CI checks formatting/build/tests, not GPU visual execution.

## Reproduce

```powershell
cargo build --release --locked
.\scripts\measure.ps1 -Seconds 14
.\scripts\review.ps1 -Iteration comparison -ExtraArgs @('--kerr')
.\target\release\journey-to-a-black-hole.exe --validate --fullscreen --seconds 86
```

Measurement logs and captures are generated locally and excluded from source control. Record hardware, backend, exact source revision, quality, exposure, pose and dimensions with new results. See [visual findings](VISUALS.md) and [model limits](MATH.md).
