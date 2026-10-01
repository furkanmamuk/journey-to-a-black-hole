# Platform and distribution status

| Target | Status |
|---|---|
| Windows x64 / Vulkan | Native renderer and extracted portable package tested on RTX 4060 |
| Other Windows graphics backends / GPUs | Unverified |
| Linux and macOS source builds | Unverified; no platform binaries published |
| WASM + WebGPU | Planned; no working browser build published |
| WebGL2 | No compatibility target currently implemented |
| itch.io | Free Windows v0.1.0, source ZIP and checksums published; signed-out access and download verified |

The Windows release contains the executable, shaders, sample configurations, documentation and license notices. It is available on [itch.io](https://mamuk.itch.io/journey-to-a-black-hole) and [GitHub Releases](https://github.com/furkanmamuk/journey-to-a-black-hole/releases/tag/v0.1.0). The source repository contains the renderer and reproducible build tools; compiled artifacts and local experiment history do not belong in Git.

On 1 October 2026, an unauthenticated itch.io download of the Windows ZIP matched the tested GitHub artifact byte for byte (SHA-256 `85fef593daff9a319137602044c30657efd56e08289d270e8ad2b1816a073137`). The renderer was executed and visually reviewed from an extracted portable package before distribution. This establishes the published download, not acceptance on other users' machines or installation through the itch desktop app.

## Browser port requirements

Bevy supports WASM/WebGPU, which can retain the GPU geodesic and PBR paths. The current application still uses native CLI startup, filesystem asset lookup/configuration/captures and native clocks. A port needs browser asset paths, a canvas/HTML loader, appropriate time APIs, configuration export/storage, pointer-lock/fullscreen handling and capability-aware diagnostics.

Browser shader validation, HDR/post-processing support, device limits, loading size and performance need actual tests. Native timing is not a browser performance promise. A first web target should detect WebGPU availability and offer the Windows download when unsupported. Mobile/touch support is separate work.

An itch.io browser build must be a web ZIP with `index.html` at its root, alongside JavaScript, WASM and assets. The native ZIP cannot run as an HTML upload. Test the resulting build inside the actual itch.io embed as well as locally before describing it as playable in a browser.

References: [Bevy WebGPU examples](https://bevy.org/examples-webgpu/), [itch.io HTML uploads](https://itch.io/docs/creators/html5).
