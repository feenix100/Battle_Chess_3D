# Game assets

These nine meshes are embedded in the executable by the Rust source. They contain
geometry, surface normals, and, where applicable, joint indices and weights for
the character rigs.

| Piece | Required meshes |
| --- | --- |
| Pawn | `pawn/pawn.mesh` |
| Knight | `knight/warhorse.mesh` |
| Bishop | `bishop/bishop.mesh` |
| Rook | `rook/rook.mesh`, `rook/brick.mesh`, `rook/hatch-left.mesh`, `rook/hatch-right.mesh` |
| Queen | `queen/queen.mesh` |
| King | `king/king.mesh` |

The pawn, bishop, rook, queen, and king are original BattleChess sculpts. The
project license is [MIT](../LICENSE). The knight asset retains its CC BY 4.0 license
and attribution below.

## Knight attribution

The knight is adapted from **MZ4250**'s *Warhorse Mount w and w/o Mini Slot*.

- Original model: https://www.printables.com/model/772400-warhorse-mount-w-and-wo-mini-slot/files
- License: [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
- Changes: leveled the stride, raised the head, shortened the tail, reduced the
  triangle count to 22,652, and added smooth normals and an eleven-joint skin for
  animation in Bevy. The game applies one side material across the horse.
