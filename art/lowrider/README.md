# GTA Street lowrider knight

An original coupe and quadcopter made for BattleChess. The car is identified as
the knight by its sculpted horse-head hood ornament, matching gold door crests,
and KN1GHT plate. No downloaded vehicle, drone, or game-character meshes are used.

The model has chrome bumpers, four individual headlights and lens ridges,
egg-crate grille bars, wing mirrors, window surrounds, interior seats and dash,
gold pinstripes, wire-spoke wheels with whitewalls and tire tread, exhaust tips,
rear lamps, and a recessed carpeted drone bay with rails and decorative equipment.
The sloped trunk lid, four wheel assemblies, steering pivots, four drone arms,
and four rotors are independently animated. Paint uses the existing team material.

## Assets

- `lowrider-knight.stl`: complete static car, 37,200 triangles.
- `trunk-drone.stl`: deployed drone, 4,172 triangles.
- `lowrider-knight.blend`: editable original parts, materials, and named pivots;
  the drone assembly is parked in the bay and the trunk is closed.
- `lowrider.mesh` and `drone.mesh`: combined BCM2 geometry for inspection.
- Named part/material `.mesh` files and `assets.rs`: embedded runtime geometry.
- `build_assets.py`: reproducible Blender source.
- `build-report.json`: counts, part pivots, bay position, and asset inventory.

These are in-game meshes, with detailed intersecting shells. STL and Blender use
Z up and +Y forward. Runtime meshes use Y up and -Z forward. Model units are
scaled by BattleChess's existing 0.73 board scale; there is no miniature print
scale. The complete car fits within a board square's inscribed circle at rest,
including its ornament, bumpers, mirrors, and exhausts.

## Capture timeline

| Time after attack starts | Action |
| --- | --- |
| 0.00–0.30 s | Turn toward the defender, small hydraulic bounce, open trunk. |
| 0.19 s | Folded drone becomes visible inside the open bay. |
| 0.36 s | Drone begins lifting from the animated bay anchor. |
| 0.36–0.64 s | Vertical lift; arms unfold and rotors spin up. |
| 0.64–1.24 s | Drone banks along a raised curve toward the target. |
| 0.78–1.04 s | Trunk closes after the drone clears the car. |
| 1.24–1.44 s | Brief alignment above the target. |
| 1.44–1.74 s | Nose-down dive into the defender. |
| 1.74 s | One impact: cyan flash, metallic fragments, and short smoke puffs. |
| After effects expire | Commit the existing legal capture, then drive into the square. |

The drone itself makes the capture. It does not drop another projectile or return
to the trunk. Subsequent attacks deploy a fresh drone. Hit height adapts when the
defender is another lowrider so bullets and other attacks hit its cabin/body.

## Movement and game integration

The lowrider drives the two legs of the knight's L, turns its front wheels at the
corner, spins its wire wheels, and rocks on its suspension. Tires remain at board
height. The existing clearance animation makes intermediate pieces yield; chess
legality and the knight's ability to move past occupied squares stay authoritative
in the existing rules system. A captured lowrider slumps on its suspension rather
than adopting the Original horse's fall or kneeling pose.

Choose **Piece set → GTA Street**. The street pawn and lowrider knight are now
available together. The Original set retains its horse and splitting-cannonball
capture. New Game cancels deployed drones and recursively clears arms, rotors,
fragments, and smoke. Set changes remain disabled during attacks and travel.

## Rebuild

Run Blender 5.2 with `--background --factory-startup --python
art/lowrider/build_assets.py`. The game does not need Blender or Python to run.
The optional `asset-preview` Cargo feature provides
`--capture-lowrider-preview ABSOLUTE_OUTPUT_DIRECTORY` for a repeatable in-game
recording of the actual capture systems.

Focused verification covers geometry/part integrity, all eight routes, grounded
wheels, trunk opening before lift-off, launch position, captures for both colors,
one capture commit, cancellation at different attack stages, set switching, and
hit height against a lowrider. The Original-knight route test is also retained.
