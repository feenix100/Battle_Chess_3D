# Blender source scripts

These are the scripts used to create the exported STL models and the skinned
meshes embedded by the game. Run them with Blender's Python environment, which
provides `bpy`, NumPy, and `mathutils`. Blender is not required to compile or play
the Rust game.

## Files

| Script | Purpose |
| --- | --- |
| [build_pawn.py](build_pawn.py) | Soldier pawn sculpt, skin, and STL export |
| [build_knight.py](build_knight.py) | Adapt and skin the original warhorse STL |
| [build_bishop.py](build_bishop.py) | Robed bishop, mitre, and crozier |
| [build_rook.py](build_rook.py) | Parapet, hinged battlement, and limb bricks |
| [build_queen.py](build_queen.py) | Queen, gown, crown, and rapier |
| [build_king.py](build_king.py) | King, crown, scepter, and cape |
| [figure_mesh.py](figure_mesh.py) | Shared sculpting and skinned-mesh export helpers |

## Running the scripts

From the repository root, run the script for the piece you want to regenerate:

```sh
blender --background --factory-startup --python docs/blender/build_pawn.py
blender --background --factory-startup --python docs/blender/build_bishop.py
blender --background --factory-startup --python docs/blender/build_rook.py
blender --background --factory-startup --python docs/blender/build_queen.py
blender --background --factory-startup --python docs/blender/build_king.py
```

The scripts write game meshes into the corresponding `art/<piece>` folder and
export STLs into [`docs/stl`](../stl). `figure_mesh.py` is a shared helper used by
the bishop, rook, queen, and king scripts. Running a script replaces that piece's
generated mesh and STL files.

## Knight source

The knight script adapts MZ4250's original
[Warhorse Mount w and w/o Mini Slot](https://www.printables.com/model/772400-warhorse-mount-w-and-wo-mini-slot/files)
under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
To regenerate the knight, download the original **Warhorse Armored no stand.stl**
and pass its location after Blender's `--` separator:

```sh
blender --background --factory-startup --python docs/blender/build_knight.py -- --source "/path/to/Warhorse Armored no stand.stl"
```

On Windows, replace the example path with the downloaded STL's full path.
Use the original download as input, rather than the already adapted export in
`docs/stl`. The script writes `art/knight/warhorse.mesh` and
`docs/stl/warhorse.stl`. The game build uses the included mesh and does not require
the original download. See the [asset credits](../../art/README.md#knight-attribution).
