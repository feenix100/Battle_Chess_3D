# GTA Street police bishop

Original procedural game geometry with a tall, diagonally notched bishop helmet,
shield badge, aviators, uniform pockets and tie, epaulettes, shoulder patches,
radio and microphone cable, duty belt, holster, handcuffs, pressed trousers,
tall boots, and a yellow fictional taser. The selected team piece color applies
to the shirt, pants, and helmet; seven additional material layers preserve
uniform and equipment detail.

`police-bishop.stl` is the full rest-pose piece; `taser.stl` is the separate held
prop; `taser-dart.stl` matches the small runtime projectile. These use visual game
units, Z up, and +Y as the bishop's forward direction. They are in-game source
assets, without a physical printing scale. The editable Blender scene contains
the mesh parts, surface assignments, and eleven-joint biped rig.

Regenerate with Blender 5.2:

```powershell
blender --background --factory-startup --python art/police_bishop/build_assets.py
python art/police_bishop/export_dart.py
```

The game embeds the BCM mesh layers and uses the generated `anchors.rs` for its
weapon kinematics. `src/characters/model/police_bishop.rs` loads the skin.
`src/characters/animation.rs` supplies the patrol stride, aiming, recoil, and
stun reaction; `src/projectiles/taser.rs` owns the tethered capture effect.

The bishop turns to face the defender and aims over 0.29 seconds. At 0.34 seconds
two darts leave the actual posed muzzle. Flight time varies from 0.25 to 0.48
seconds with square distance. Two segmented cables sag between the moving
weapon and darts. On contact, two cyan pulses travel along the cables while
short electrical arcs flicker between the contact points for 0.42 seconds.
The defender stiffens, then falls over 0.46 seconds. Vehicles and buildings use
their own disabled poses. The circuit root gates the existing capture lifecycle,
so the legal move commits once after the effect and fall finish.

All cable segments, pulses, embedded dart proxies, and sparks belong to the
circuit root and clear with the existing New Game/reset cancellation. Switching
Original/GTA Street rebuilds board pieces and captured trays between moves.
The Original bishop retains its three ball-lightning projectiles.

Focused checks:

```powershell
cargo test --offline --locked --features asset-preview police_
cargo fmt --check
```

The opt-in `asset-preview` feature records the actual game capture:

```powershell
cargo run --features asset-preview -- --capture-police-preview ABSOLUTE_OUTPUT_DIR
```

Geometry and source code are covered by the repository's MIT license.
