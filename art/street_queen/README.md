# GTA Street mini-Uzi queen

Original adult woman with a cropped shirt, lined pleated skirt, long waist-length
ponytail, five-point tiara, hoop earrings, necklace, bangles, hip bag, manicure,
and laced street boots. Team piece colors apply to the shirt, skirt, hair tie,
and boot cuffs. Seven extra layers preserve hair, skin, jewelry, makeup, leather,
metal, and sole detail.

The compact mini Uzi has an original receiver, short hollow barrel, cooling
slots, raised sights, charging handle, stick magazine, folding wire stock,
trigger guard, grip texture, ejection port, and queen crest. The gripping hand
shares its joint and stays attached as the gun raises, sweeps, and recoils.

Assets:

- `street-queen.stl`: complete queen in the resting pose, gun held low.
- `queen-mini-uzi.stl`: separate mini Uzi in its rest orientation.
- `smg-round.stl` and `smg-spent-case.stl`: small projectile and casing props.
- `street-queen.blend`: editable original parts, materials, and twelve-joint rig.
- `*.mesh`: embedded BCM2 material layers and combined validation mesh.
- `anchors.rs`: generated rest joints, muzzle and ejection coordinates.

Character STLs use Z up and +Y forward. The projectile props follow their local
flight axis along -Y in STL coordinates. All use visual game units with no
assigned physical miniature scale. The source and original geometry use the
repository's MIT license.

Regenerate:

```powershell
blender --background --factory-startup --python art/street_queen/build_assets.py
python art/street_queen/export_props.py
```

The shared biped uses joints 0–10. Joint 11 is the skirt pivot at the waistband;
the pleated shell and opaque lining sway from it during movement. The long
ponytail follows the head and reaches the waist. The BCM2 header records all
twelve joints; the existing biped builder creates the extra garment joint.

`src/characters/model/street_queen.rs` loads the skin and calculates posed gun
anchors. `src/characters/animation.rs` supplies the hip-carried walk, aiming,
spray sweep, recoil, and recovery. `src/projectiles/smg.rs` owns the burst.

The queen raises the gun over 0.38 seconds. Ten small rounds launch from the
actual posed muzzle, starting at 0.46 seconds and spaced 0.07 seconds apart.
The gun sweeps gently from side to side, with short per-shot recoil. Each round
has a brief two-layer flash and brass casing. Casings fall onto the board within
the queen's square and settle briefly beside her feet. Settled casings remain
cosmetic while she begins moving. Intermittent smoke expands and fades.
Flight time varies with board distance.
The queen aims down for the lowrider. Early impacts produce compact sparks and
brief defender flinches. Every opponent receives the same stylized red-fluid
spray and gray impact smoke. Only the final round starts the normal fall and capture
commit lifecycle. The queen lowers the gun and then walks into the square.

Reset/New Game clears waiting rounds, trails, flashes and their cores, casings,
smoke, fluid, and impact fragments. Promotion, board/tray style switching, and the
Original queen's eight-shell artillery attack remain supported.

Focused checks:

```powershell
cargo test --offline --locked --features asset-preview street_queen_
cargo test --offline --locked --features asset-preview street_pawn_captures_in_both_directions_including_en_passant
cargo test --offline --locked --features asset-preview street_king_spread_blast_commits_once_for_both_colors_and_lowrider_targets
cargo test --offline --locked --features asset-preview mansion_walk_preserves_molotov_arc_fire_and_single_capture_for_both_colors
cargo fmt --check
```

Record the actual game capture with the opt-in developer harness:

```powershell
cargo run --features asset-preview -- --capture-queen-preview ABSOLUTE_OUTPUT_DIR
```
