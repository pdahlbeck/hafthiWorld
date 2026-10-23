# Physical coastal rendering review

The rejected PR #23 screenshot is the baseline. The supplied village image is
the target, not evidence that either PR reaches that target. Automated tests
verify behavior and numerical contracts; actual screenshots are required to
assess appearance. This remains a small settlement prototype with one owned
house; it does not reproduce the image's architecture, crowd, assets or density.

## Engine decision

Keep Bevy 0.20, using its existing physical atmosphere and environment filtering.
No missing upstream rendering primitive was identified that would justify an
engine fork. A fork alone would not replace poor scene art. The runtime now uses:

- High: Earth Rayleigh/Mie scattering, sky-view/aerial LUTs, a visible solar disk,
  atmosphere-generated diffuse irradiance and prefiltered environment reflections.
- Raw solar illuminance filtered by the atmosphere, white incident sunlight,
  photographic exposure and ACES tonemapping. High's former distance fog and
  large uniform ambient fill are removed to avoid double haze and flat shadows.
- Temporally filtered cascaded shadows and contact shadows, alongside existing
  HDR, SSAO and TAA. Shadow coverage is tuned to the walkable coastal map.
- One solar frame derived directly from the saved presentation hour. The sun
  crosses below the horizon at night. Exposure has no temporal adaptation state,
  so camera switches and F9 do not require convergence to the saved hour.
  Night adds a 0.25 lux cool directional fill and bounded photographic exposure
  (EV 8–13). Daylight exposure remains EV 13 through the horizon crossing;
  a smooth transition begins below the horizon, avoiding extreme amplification
  of the still-bright twilight sky. Lighting updates precede exposure updates
  in the same frame. The night fill is an artistic full-moon floor, not a lunar
  ephemeris. The bounded gain intentionally leaves night darker than daytime.

Low retains the old light/fog pipeline without atmospheric environment-map
compute passes, contact shadows or TAA. It shares the revised art and terrain
shader. Quality is chosen at launch; there is no mid-session pipeline switch.
No path tracing, scene-object raytraced reflection or global diffuse bounce is
claimed. The atmosphere's sky illumination is not an occlusion-aware GI solver.

## Materials and geometry

The new original reed albedo replaces the flat roof material. Parallel analytic
micro-normals provide fiber response, UVs follow the roof slope at 1.2 m per tile,
and merged cut reeds plus bent wooden battens add actual silhouette/shadow detail.
The house's footprint, door, ownership and collision remain unchanged.

Pines use a new original alpha-cut branch texture on three curved branch ribbons
instead of solid triangular needle fans. Decorative pines have taller bare stems,
and the distant woodland mixes needle crowns and existing broadleaf crowns.
The three harvestable trees keep their original trunk height and tree identities.
Material mip generation filters color with alpha weighting so transparent black
texels do not contaminate the color of needle edges. There is no alpha-coverage
rescaling, so the finest needles can still thin out at distance.

Terrain uses broad low-frequency color variation over grass detail, irregular
layer blending and a screen-footprint-filtered micro-normal. The water uses the
physical sky's environment lighting in High; the old additive sky approximation
is restricted to Low. Domain-warped wave phases reduce regular parallel bands.
The analytical water-depth/shore contract and shared ground height are retained.

Asset provenance and prompts: `tools/art/coastal-material-prompts.md`.
Generated texture outputs are unchanged; the reed normal is separately analytic.
No external game art, models or source code are imported.

## Review

```sh
cargo run --release --locked -- --quality high
cargo run --release --locked -- --quality low
```

F4 hides the HUD. Tab changes camera mode. T stops the sun; Y advances three
hours. Inspect noon, late afternoon, sunset and night, including F5/F9 restoration.
Review needle edges near/far, roof fibers, terrain detail and water in both views.
A visual improvement here does not establish parity with the reference: authored
characters, architecture, stonework and vegetation still need substantial work.
Linux software-Vulkan tests are functional evidence, not native Metal performance.
