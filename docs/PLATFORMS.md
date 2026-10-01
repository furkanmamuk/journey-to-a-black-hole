# Platform and distribution status

| Target | Status |
|---|---|
| Windows x64 / Vulkan | Native renderer and extracted portable package tested on RTX 4060 |
| Other Windows graphics backends / GPUs | Unverified |
| Linux and macOS source builds | Unverified; no platform binaries published |
| WASM + WebGPU | Planned; no working browser build published |
| WebGL2 | No compatibility target currently implemented |

The Windows release contains the executable, shaders, sample configurations, documentation and license notices. It can be distributed directly through a GitHub Release or uploaded to itch.io as a Windows download. The source repository contains the renderer and reproducible build tools; compiled artifacts and local experiment history do not belong in Git.

## Browser port requirements

Bevy supports WASM/WebGPU, which can retain the GPU geodesic and PBR paths. The current application still uses native CLI startup, filesystem asset lookup/configuration/captures and native clocks. A port needs browser asset paths, a canvas/HTML loader, appropriate time APIs, configuration export/storage, pointer-lock/fullscreen handling and capability-aware diagnostics.

Browser shader validation, HDR/post-processing support, device limits, loading size and performance need actual tests. Native timing is not a browser performance promise. A first web target should detect WebGPU availability and offer the Windows download when unsupported. Mobile/touch support is separate work.

An itch.io browser build must be a web ZIP with `index.html` at its root, alongside JavaScript, WASM and assets. The native ZIP cannot run as an HTML upload. Test the resulting build inside the actual itch.io embed as well as locally before describing it as playable in a browser.

References: [Bevy WebGPU examples](https://bevy.org/examples-webgpu/), [itch.io HTML uploads](https://itch.io/docs/creators/html5).
