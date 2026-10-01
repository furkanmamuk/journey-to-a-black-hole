# Distribution and release checklist

## Current release

[itch.io](https://mamuk.itch.io/journey-to-a-black-hole) and [GitHub v0.1.0](https://github.com/furkanmamuk/journey-to-a-black-hole/releases/tag/v0.1.0) distribute the same tested Windows x64 ZIP. Both also offer the tracked source ZIP and SHA-256 checksums. The itch.io page is free with payments disabled and clearly classified as a prototype.

The initial source archive is the immutable v0.1.0 tag. The main branch can contain newer documentation; use the tag or SOURCE-COMMIT.txt to match the executable. Do not silently replace old versioned artifacts when rendering or packaging changes.

## Before publishing an update

1. Update Cargo.toml and Cargo.lock to the release version. Run formatting, locked compilation and numerical tests. Check hosted CI separately.
2. Run the release renderer at 1920×1080 output. Review real captures and moving views. Record hardware, quality, dimensions, exposure and the boundaries of CPU/GPU timings.
3. Commit the public source. From a clean checkout run `scripts/package.ps1 -Version VERSION`; the source ZIP is created from Git HEAD. Only use `-SkipBuild` if the release executable was already built from that exact source.
4. Extract the ZIP to a fresh directory. Launch both guided and Kerr exploration from a different working directory. Verify shader/assets lookup, controls and explicit missing-asset errors. Include dependency and CIE attribution notices.
5. Tag and push the tested commit. Publish the Windows ZIP, source ZIP and SHA256SUMS through a GitHub Release, with checks actually performed and known numerical/platform limits.
6. Upload those same files through the itch.io project's Edit game page. Mark only the Windows ZIP as Executable / Windows, the source ZIP as Source code, and checksums as Documentation or Instructions. Update launch guidance and version references.
7. Save and review the page before publication. Verify public access while signed out. Download the Windows ZIP using the public UI and compare its SHA-256 with the tested artifact.

The current page uses actual renderer captures for the cover and screenshots. Keep screenshots representative of the released renderer; do not substitute generated illustrations for gameplay evidence. The page discloses AI-assisted code and documentation under Code and Text & Dialog. It does not claim generated bitmap artwork or audio.

## Page maintenance

Keep the Windows download first. Include extraction instructions, cursor release controls, performance context, known limits, the public source repository and an issue-report link. Use the native download project type until a browser artifact has passed acceptance inside the actual embed.

No itch.io tokens, browser sessions or account credentials belong in the repository. The current release was uploaded through the creator UI; butler automation is optional future tooling. Windows ZIP launchers were tested as native command equivalents; installation and launch through the itch desktop app remain unverified.

## Browser acceptance is a separate gate

Bevy's WebGPU backend can retain the ray solver and PBR path, but this project is not currently a browser build. Port native startup, clocks, assets, configuration persistence and diagnostics first. Browser keyboard conflicts, pointer lock and fullscreen need explicit handling. Timestamp queries must show unavailable when unsupported, rather than converting CPU time into a GPU measurement.

An HTML ZIP must contain index.html at its root with the generated JavaScript, WASM and shader assets. Detect unavailable WebGPU and present the Windows download link. Test loading, resizing, camera motion, all presentation/physics toggles and recovery from unsupported/device-lost cases. Capture and profile both Schwarzschild and Kerr in a supported browser, then repeat inside the itch.io iframe before enabling HTML playback.

References: [itch.io project setup](https://itch.io/docs/creators/getting-started), [HTML uploads](https://itch.io/docs/creators/html5), [Bevy WebGPU examples](https://bevy.org/examples-webgpu/).
