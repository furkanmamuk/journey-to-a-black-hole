# Contributing and making a release

Keep this a small rendering experiment. Changes should improve ray transport, physically motivated emission, material response, stability or inspectability. Avoid adding gameplay/simulation frameworks or coupling the black-hole model to foreground material shaders.

## Verify a change

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo build --release --locked
```

Numerical tests include known Schwarzschild limits, Kerr analytic gradients, null initialization, conserved momentum, spin symmetry and convergence. They do not certify arbitrary f32 GPU rays. Windows CI runs CPU/build checks; a GPU-equipped machine is required for visual acceptance.

On Windows, run the real demo and inspect the affected views. `--validate --fullscreen --seconds 86` exercises render toggles, resolutions, disk orientations, inspection and spin signs. Compare F7, F11 and M. Review a moving camera sequence as well as static screenshots; pleasing reduced output can conceal unfinished internal rays.

```powershell
.\scripts\review.ps1 -Iteration comparison -ExtraArgs @('--kerr')
.\scripts\measure.ps1 -Seconds 14
```

Record hardware, build type, physical/grid/output sizes, observer pose, quality and exposure. Distinguish frame interval, main ECS CPU time, individual GPU passes and queued GPU time. Preserve relevant failures and describe the limits; do not claim desktop measurements apply to browser/mobile hardware.

## Package a Windows release

```powershell
.\scripts\package.ps1 -Version 0.1.0
```

The script builds the locked release, collects dependency license files, and writes a portable ZIP, an inspectable source ZIP and SHA256 checksums under `dist/`. By default the source package comes from tracked Git files; commit the intended source before packaging. Compiled outputs, logs, captures, saved personal camera state, caches and experimental backups stay excluded.

Extract the portable ZIP into a fresh folder and launch its executable from a different working directory. Keep `assets/` beside it. Verify the sample Kerr configuration, normal Journey, shader compilation and output dimensions. GPU execution is not established by `cargo check` alone.

Publish tagged source and the verified ZIPs/checksums as a GitHub Release. The same portable ZIP can be used as the Windows upload on itch.io. The release must accurately state platform coverage and known limitations. A browser upload is a separate build, described in [platform status](docs/PLATFORMS.md).

## Licensing and reports

Use the repository's MIT OR Apache-2.0 license for contributed original code. Preserve the separate CIE table attribution/license and applicable third-party notices. Do not add downloaded assets without clear provenance and redistribution terms.

For rendering/physics bugs, include a reproducible configuration or CLI, GPU/backend, quality/resolutions, screenshot and relevant errors. Exclude credentials, personal paths and unrelated logs. A short ray-inspector or M/F11 comparison is especially useful for integration failures.
