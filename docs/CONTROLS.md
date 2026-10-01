# Controls and reproducible views

## Journey and camera

The default guided duration is 150 seconds, with a fixed 60-degree FOV. It holds at **160 rs**, physically approaches, moves sideways, changes observer inclination, makes a close orbit at **6 rs**, then approaches **1.12 rs** in Schwarzschild (about **1.163 rs** for chi=0.8 Kerr, outside the static-observer limit). Motion comes from smooth interpolation of ordinary spherical camera coordinates; radius interpolation is logarithmic. The disk and solver keep evaluating continuously.

**J** pauses/resumes. **Enter** restarts. Any WASD/up/down/roll input or captured mouse motion immediately takes over from the current transform, without a camera jump or a different model. After manual takeover, J starts a new journey. Selecting a preset also exits guidance. At the terminal boundary the Journey stops and labels the external-observer cutoff. No interior is drawn. Free flight is still available, clamped outside **1.08 rs** for numerical safety. This is a sequence of static external observers, not a freely falling relativistic observer with aberration.

| Input | Action |
|---|---|
| WASD / Space / left Ctrl | Camera-local translation, including above and below the disk |
| Mouse | Yaw/pitch after capture |
| Q / E | Roll |
| Left Shift / left Alt | 4× speed / 1⁄8 precision speed |
| Mouse wheel | Adjust movement speed (0.02–2000 scene units/s) |
| Tab / right mouse | Capture or release mouse |
| Esc / focus loss | Release mouse |
| J / Enter | Journey start/pause/resume / restart |
| 1 | Distant telescope view, 160 rs |
| 2 | Nearly edge-on disk |
| 3 | Above-disk view |
| 4 | Close lateral view |
| 5 | Reference-star alignment / Einstein-ring experiment |
| 6 | Engineering probe foreground |
| 7 | Near supported limit, about 1.12 rs |

Presets are only transforms; normal movement works immediately afterward. Preset 6 is positioned near the unchanged probe. The astronomical presets scale with rs; changing rs afterward leaves a free camera stationary so the model change is observable.

## Comparison and parameter controls

| Input | Action |
|---|---|
| P / H | Show full parameters/help / hide all UI including inspector |
| F1 / F2 / F3 / F4 | Lensing / disk / celestial background / probe |
| F5 / F6 / F7 / F8 | Bloom / foreground TAA / all quantization / dither |
| F9 / F10 | Histogram automatic exposure / explicit distant stellar light |
| F11 | Exhausted integration budgets in magenta |
| B | GPU ray classes: dark captured, green escaped, amber disk hit, magenta exhausted |
| G | Deterministic stars → celestial grid → both |
| T / U / M | Kerr model / ISCO following / constraint error |
| Z / X / C | Doppler temperature shift / D⁴ bolometric beaming / gravitational redshift |
| V | Tonal quantization only; spatial grid remains |
| Up / Down, Left / Right | Select a parameter, then adjust it |
| + / − | Brighten / darken by 0.25 EV |
| F12 / L | Save `journey.cfg` / reload it (or initial CLI configuration if no saved file exists) |
| I | Open/close the separate ray inspector |
| K / O | 9 or 25 sample rays / inspector orbit, top or side view |
| , / . | Select a sample ray, highlighted white |
| [ / ] | Narrow/widen the screen-ray sampling spread |

Adjustable rows include quality, steps, HDR resolution, presentation-grid resolution, exposure, rs, disk inner/outer radii, maximum affine distance, tonal levels, **camera r/rs**, disk tilt, disk azimuth, emissivity and star seed, signed spin and temperature scale. T switches Schwarzschild/Kerr, U follows the ISCO, and M displays accumulated null/axial constraint error. Camera-distance adjustment takes over from guidance. The disk ISCO constraint is spin-dependent (3rs for Schwarzschild); U defaults to following it, while manual inner-radius edits disable following; dimensions and the maximum-distance budget are stored in scene units, with disk/camera distances displayed as ratios to rs. Changing rs updates an ISCO-following inner edge; with U off, the safety constraint only expands an invalid inner radius.

Useful CLI options: `--quality 0|1|2`, `--steps N`, `--seed U32`, `--disk-tilt DEGREES`, `--disk-azimuth DEGREES`, `--emissivity SCALE`, `--sky-grid`, `--sky-both`, `--ray-classes`, `--inspect-rays`, `--taa`, `--auto-exposure`, `--budget-debug`, `--hide-hud`, `--no-lensing`, `--no-disk`, `--no-stars`, `--no-craft`, `--no-stellar-light`, `--no-bloom`, `--no-quantization`, `--no-tonal`. See `--help`.

For repeatable tests, `--seconds N` exits normally, `--capture` saves a PNG after ten seconds, `--journey-seconds N` changes tour duration, and `--journey-at 0..1` starts at a chosen progress. These only change normal camera motion. `--benchmark` cycles all seven camera transforms with a lateral sweep; it is a test harness, not an alternative visual model. `--validate` runs an 86-second GPU feature/resize/parameter/inspector smoke sequence, including both spin signs and fractional grids.
