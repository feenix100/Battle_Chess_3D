# GTA Street shotgun king

An original barrel-chested adult man with a handlebar mustache, tied bandanna,
gold crown emblems, linked necklace, crown pendant, wallet chain, watch, crown
tattoo, tank top, baggy cargo trousers, and heavy laced boots. Team colors apply
to the tank top, trousers, and bandanna. Seven material layers retain the skin,
hair, metal, wood, stitches, and eye detail.

His wood-stock shotgun rests diagonally against the right shoulder, tilted up
and back. The bind pose includes this carry and a bent right arm. The gun has a
hollow muzzle, barrel, magazine tube, ribbed pump, sights, receiver, ejection
port, trigger guard, carved wood stock, and rubber butt pad. The gripping hand
shares the gun joint so it stays attached as the weapon turns.

Files:

- `street-king.stl`: complete king in the shoulder-carry rest pose.
- `king-shotgun.stl`: separate shotgun prop in its carried orientation.
- `shotgun-pellet.stl`: small spherical projectile.
- `spent-shotgun-shell.stl`: separate spent-shell prop.
- `street-king.blend`: editable parts, materials, and eleven-joint rig.
- `*.mesh`: embedded BCM1 material layers and combined validation mesh.
- `anchors.rs`: generated joint, muzzle, ejection, and support-hand coordinates.

STLs use visual game units, Z up and +Y forward. They are in-game source assets
and have no assigned physical miniature scale. Geometry and code use the
repository's MIT license; no downloaded character geometry is used.

Regenerate the sources:

```powershell
blender --background --factory-startup --python art/street_king/build_assets.py
python art/street_king/export_props.py
```

The game uses `src/characters/model/street_king.rs` to load the layered skin and
calculate posed muzzle/ejection positions. Its two-bone support-hand solver
keeps the free hand on the fore-end during aim and recoil. Shoulder yaw and
weapon yaw default to zero for the existing figures.

The capture turns toward the target, lowers the gun from 0.08 to 0.56 seconds,
and fires one nine-pellet spread at 0.68 seconds. A brief muzzle flash, expanding
smoke, and one red/brass shell accompany the blast. Pellets travel directly to
a compact set of contact points; shorter targets such as the lowrider receive
a downward aim. One compact impact runs after the simultaneous pellets arrive.
The existing legal move commits once after the effect and defender's fall finish.
The king returns the gun to his shoulder from 1.04 to 1.44 seconds, then walks
into the captured square. Quiet movement, castling, yielding to knights, and
the checkmate bow retain the carried gun.

`src/projectiles/shotgun.rs` owns the visual blast. Cloud components use the
existing debris lifetime but bypass its gravity and tumbling; the smoke system
expands and fades them. Reset/New Game removes pellet roots and trails, smoke,
the flash, shell and brass rim. The Original king retains its bomb attack.

Focused verification:

```powershell
cargo test --offline --locked --features asset-preview street_king_
cargo test --offline --locked --features asset-preview police_bishop_two_darts_stun_and_commit_once_for_both_colors
cargo test --offline --locked --features asset-preview mansion_walk_preserves_molotov_arc_fire_and_single_capture_for_both_colors
cargo fmt --check
```

Record the actual game sequence with the opt-in developer harness:

```powershell
cargo run --features asset-preview -- --capture-king-preview ABSOLUTE_OUTPUT_DIR
```
