# GTA Street pawn

An original streetwear pawn made for this BattleChess project. No downloaded
character geometry, logos, or named game characters are used.

The sculpt has a rounded six-panel cap, sunglasses, sculpted face and goatee,
oversized jersey, rolled collar, baggy denim, pocket piping and seams, chunky
sneakers, wrist bandanna, watch, pistol details, individual chain links, and a
small chess-pawn pendant. The shirt, cap, bandanna, and sneaker accents use the
game's selected team material. Six additional material layers distinguish skin,
denim, shoes, gold, dark equipment, and stitching.

## Files

- `street_pawn.stl`: complete static model, 26,156 triangles.
- `street_pawn.blend`: editable original parts and eleven-joint armature.
- `street_pawn.mesh`: complete BCM1 mesh for geometry and skin validation.
- `team.mesh` and the six named material meshes: actual embedded game geometry.
- `bullet.stl`: small visual projectile matching the game's brass round.
- `build-report.json`: vertex counts, bind positions, muzzle anchor, and axes.
- `build_mesh.py` and `export_bullet.py`: reproducible asset sources.

STL and Blender use Z up and +Y forward. BCM1 uses Y up and -Z forward. STL
coordinates are model units, without the previous miniature's 35x print scale.
The game applies its existing 0.73 board scale. These STLs are in-game visual
assets and consist of detailed intersecting shells; they are not prepared as
single-volume printable sculptures.

## Regenerate

Run Blender 5.2 in background mode with `--python art/street_pawn/build_mesh.py`.
Run `python art/street_pawn/export_bullet.py` for the projectile STL. The runtime
does not need Blender or Python; it embeds the generated meshes in the executable.

## Attack

The pawn raises its right arm over 0.265 seconds. Three small brass bullets launch
at 0.300, 0.415, and 0.530 seconds, each flying for 0.200 seconds on a straight path.
Forward kinematics calculates each launch from the animated pistol muzzle.
Each shot has a brief muzzle flash, casing ejection, and matching recoil. The
first two impacts produce short flinches; the final impact begins defeat.
Compact sparks and smoke expire before the legal capture commits, then the pawn
swaggers into the destination. En passant aims at the actual victim square and
lands on the legal destination square. Existing capture cancellation clears all
projectiles, flashes, casings, sparks, and smoke.

## Set selection

The game menu's **Piece set** selector offers **Original** and **GTA Street**.
Original is the default. GTA Street includes this pawn, the lowrider knight and
the mansion rook; the remaining three roles use their Original assets until built individually.
The selector is temporarily disabled during attacks and movement. Switching sets
refreshes board and captured-tray models while preserving the position and history.

The knight is documented separately in [`../lowrider`](../lowrider), including its
trunk-launched drone capture.
