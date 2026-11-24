# GTA Street mansion rook

The mansion now deploys two stone-and-metal legs during movement and castling.
It rises from its foundation, takes alternating steps with planted feet, and
retracts the legs before settling at its destination. Idle and attack poses
keep the legs concealed. The rooftop Molotov retains its enlarged game scale.

`mansion-walking.stl` shows the deployed stance; `mansion-legs.stl` contains the
separate leg parts. The standard rest-pose STL and editable Blender scene keep
the legs within the building volume. Runtime transforms deploy them from the
foundation; the house material loader keeps leg geometry out of the bottle prop.

Focused checks: `cargo test --offline --locked --features asset-preview mansion_walk_`.
The opt-in `--capture-mansion-walk-preview ABSOLUTE_OUTPUT_DIR` records the actual
game's deployment, steps, and retraction.

An original miniature mansion with a rooftop Molotov capture, built for the
existing BattleChess engine. The crenellated parapet and four corner crowns keep
the rook silhouette. The estate styling includes two storeys of arched glazing,
gold mullions, a balcony with turned balusters, tiled portico, entry columns,
double doors, lanterns, stone quoins, planters, chimney, and hinged iron gates.
The major wall surfaces follow the game's selected team material.

## Files

- `mansion-rook.stl`: complete rest-pose building and concealed legs, 39,632 triangles.
- `molotov.stl`: stylized bottle and cloth end, 772 triangles.
- `mansion-rook.blend`: editable original parts, materials and named pivots.
- `mansion.mesh` and `molotov.mesh`: combined runtime-format geometry for inspection.
- Named part/material `.mesh` files and `assets.rs`: the embedded runtime layers.
- `build_assets.py`: original geometry source; reuses the project's geometry
  helpers in `../lowrider/build_assets.py` without using car or drone geometry.
- `build-report.json`: counts, pivots, release point and asset inventory.

STL and Blender use Z up and +Y forward. Runtime BCM2 meshes use Y up and -Z
forward, followed by the game's existing 0.73 board scale. These are in-game
visual meshes with detailed intersecting shells, without a physical print scale.
The building, plinth, gates and planters fit inside the board square at rest.

## Capture performance

| Stage | Motion |
| --- | --- |
| 0.00–0.28 s | Turn toward the defender and open the terracotta rooftop hatch. |
| 0.34–0.42 s | The bottle rises visibly from the recessed roof opening. |
| 0.42 s | Release from the animated rooftop anchor with a flickering wick flame. |
| Following 0.77–1.01 s | Rotate along a high arc; leave short smoke puffs. Flight time and arc height adapt to distance. |
| 0.68–0.95 s | Close the roof hatch after the bottle clears it. |
| Impact | Break into small green fragments and ignite a brief fire beneath the defender. |
| Following 1.00 s | Animate flames, smoke and a temporary scorch ring within the captured square. |
| After the fire expires | Commit the legal capture, deploy the legs, and walk into the destination. |

Fire roots stay fixed at board height. Their timer participates in the existing
capture-effects lifecycle. New Game clears the bottle, wick flames, trail puffs,
fragments, fire and child effects. The fire is part of the capture presentation;
chess legality and captured-piece accounting remain owned by the existing game.

## Movement and set selection

Choose **Piece set → GTA Street**. All six GTA piece types are available.
The Original rook retains its drone/grenade attack and
unfolding brick limbs.

The mansion extends its legs, follows a straight stepping path, and retracts
them at the destination. Its castling path follows the existing king-and-rook move. Captured or
checkmated mansions settle and lean slightly using a building-specific pose.
Set switching refreshes board and captured-tray visuals between moves.

## Rebuild and focused verification

Run Blender 5.2 with `--background --factory-startup --python
art/mansion/build_assets.py`. The compiled game embeds the assets and does not
need Blender or Python. With Cargo's optional `asset-preview` feature, use
`--capture-mansion-preview ABSOLUTE_OUTPUT_DIRECTORY` to record the actual game.

For the walking-leg change, three `mansion_walk_` checks cover hidden legs,
planted feet, straight travel, castling, and the existing Molotov capture for
both colors. One playable preview checks the actual mesh deployment and gait.
