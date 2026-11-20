//! Articulated character visuals. Movement follows committed chess moves. Battle
//! poses observe capture start and impact while the effects system owns timing.

mod animation;
mod model;
#[cfg(test)]
mod tests;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Move, Piece, Role, Square};

use self::animation::{KnightRoute, Pose};
use self::model::{Joint, Rig};
use crate::game::GameOutcome;
use crate::{
    Appearance, BattleGame, SceneHandles, SceneSync, capture_square, display_move_to, move_from,
    square_from_world, square_world,
};

pub(crate) use model::CharacterAssets;

pub(crate) const BOARD_SCALE: f32 = 0.73;
const TRAY_SCALE: f32 = 0.42;

#[derive(Clone, Copy)]
struct Travel {
    from: Vec3,
    role: Role,
    elapsed: f32,
    duration: f32,
    distance: f32,
    lane_offset: f32,
}

#[derive(Clone, Copy)]
struct YieldSpace {
    direction: Vec3,
    amount: f32,
}

struct Battle {
    movement: Move,
    attacker: Square,
    victim: Square,
    elapsed: f32,
    since_hit: Option<f32>,
}

pub(crate) struct CheckmateCelebration {
    pub loser: ChessColor,
    pub elapsed: f32,
}

struct Visual {
    piece: Piece,
    rig: Rig,
    rest: Quat,
    animated: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct TrayView {
    visible: bool,
    upright: bool,
    knight_orientation: i32,
}

#[derive(Resource)]
pub(crate) struct CharacterScene {
    board: [Option<Visual>; 64],
    travel: [Option<Travel>; 64],
    battle: Option<Battle>,
    pub checkmate: Option<CheckmateCelebration>,
    tray_entities: Vec<Entity>,
    tray_pieces: Vec<(Piece, ChessColor)>,
    tray_view: Option<TrayView>,
}

impl Default for CharacterScene {
    fn default() -> Self {
        Self {
            board: std::array::from_fn(|_| None),
            travel: [None; 64],
            battle: None,
            checkmate: None,
            tray_entities: Vec::new(),
            tray_pieces: Vec::new(),
            tray_view: None,
        }
    }
}

impl CharacterScene {
    fn start_travel(
        &mut self,
        commands: &mut Commands,
        from: Square,
        to: Square,
        role: Role,
        lane_offset: f32,
    ) {
        if from == to {
            return;
        }
        let from_index = from.to_usize();
        let to_index = to.to_usize();
        self.remove(commands, to);
        self.board[to_index] = self.board[from_index].take();
        self.travel[from_index] = None;
        let origin = square_world(from);
        let distance = origin.distance(square_world(to));
        self.travel[to_index] = Some(Travel {
            from: origin,
            role,
            elapsed: 0.0,
            duration: animation::move_seconds(role, distance),
            distance,
            lane_offset,
        });
    }

    fn remove(&mut self, commands: &mut Commands, square: Square) {
        let index = square.to_usize();
        if let Some(visual) = self.board[index].take() {
            commands.entity(visual.rig.root).despawn();
        }
        self.travel[index] = None;
    }

    fn committed(&mut self, commands: &mut Commands, movement: Move) {
        if let Some(victim) = capture_square(&movement) {
            self.remove(commands, victim);
        }
        if let (Some(from), Some(to)) = (move_from(&movement), display_move_to(&movement)) {
            self.start_travel(commands, from, to, movement.role(), 0.0);
        }
        if let Move::Castle { rook, .. } = movement {
            let side = movement.castling_side().expect("castle has a side");
            let to = Square::from_coords(side.rook_to_file(), rook.rank());
            // Give the rook room to pass the king as their paths cross.
            let lane = if square_world(rook).z > 0.0 {
                0.60
            } else {
                -0.60
            };
            self.start_travel(commands, rook, to, Role::Rook, lane);
        }
    }

    fn update_battle(&mut self, pending: Option<Move>, impact: bool, dt: f32) {
        let Some(movement) = pending else {
            self.battle = None;
            return;
        };
        if let Some(battle) = &mut self.battle
            && battle.movement == movement
        {
            battle.elapsed += dt;
            if let Some(since_hit) = &mut battle.since_hit {
                *since_hit += dt;
            } else if impact {
                battle.since_hit = Some(0.0);
            }
            return;
        }
        if let (Some(attacker), Some(victim)) = (move_from(&movement), capture_square(&movement)) {
            self.travel[attacker.to_usize()] = None;
            self.travel[victim.to_usize()] = None;
            self.battle = Some(Battle {
                movement,
                attacker,
                victim,
                elapsed: 0.0,
                since_hit: impact.then_some(0.0),
            });
        }
    }
}

fn rest_heading(piece: Piece, knight_orientation: i32) -> Quat {
    let forward = if piece.color == ChessColor::White {
        0.0
    } else {
        PI
    };
    let offset = if piece.role == Role::Knight {
        (knight_orientation as f32).to_radians()
    } else {
        0.0
    };
    Quat::from_rotation_y(forward + offset)
}

fn team_material(piece: Piece, handles: &SceneHandles) -> Handle<StandardMaterial> {
    match piece.color {
        ChessColor::White => handles.white_piece.clone(),
        ChessColor::Black => handles.black_piece.clone(),
    }
}

fn battle_pose(
    battle: Option<&Battle>,
    square: Square,
    role: Role,
    rest: Quat,
) -> Option<(Quat, Pose)> {
    let battle = battle?;
    let here = square_world(square);
    if square == battle.attacker {
        let facing = animation::heading(here, square_world(battle.victim));
        let elapsed = battle.elapsed;
        let duration = match role {
            Role::Pawn => 0.47,
            Role::Knight => 0.71,
            Role::Rook => crate::projectiles::ROOK_SEQUENCE_SECONDS,
            Role::Bishop | Role::Queen => 0.80,
            Role::King => 0.96,
        };
        Some((
            rest.slerp(facing, (elapsed / 0.12).clamp(0.0, 1.0)),
            animation::attacking(role, (elapsed / duration).min(1.0)),
        ))
    } else if square == battle.victim {
        let facing = animation::heading(here, square_world(battle.attacker));
        let turn = rest.slerp(facing, (battle.elapsed / 0.21).clamp(0.0, 1.0));
        let pose = battle
            .since_hit
            .map(|t| animation::captured(role, (t / 0.46).min(1.0)))
            .unwrap_or_default();
        Some((turn, pose))
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_characters(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<BattleGame>,
    appearance: Res<Appearance>,
    handles: Option<Res<SceneHandles>>,
    kit: Option<Res<CharacterAssets>>,
    mut sync: ResMut<SceneSync>,
    mut scene: ResMut<CharacterScene>,
    mut roots: Query<&mut Transform, Without<Joint>>,
    mut joints: Query<&mut Transform, With<Joint>>,
) {
    let (Some(handles), Some(kit)) = (handles, kit) else {
        return;
    };
    let dt = time.delta_secs();
    if std::mem::take(&mut sync.reset_characters) {
        for square in Square::ALL {
            scene.remove(&mut commands, square);
        }
        scene.battle = None;
        scene.checkmate = None;
    }

    if let Some(defeat) = &mut scene.checkmate {
        defeat.elapsed += dt;
    }

    for travel in &mut scene.travel {
        if let Some(active) = travel {
            active.elapsed += dt;
            if active.elapsed >= active.duration {
                *travel = None;
            }
        }
    }
    for movement in sync.character_moves.drain(..) {
        scene.committed(&mut commands, movement);
    }
    if let Some(GameOutcome::Checkmate { winner }) = game.outcome() {
        if scene
            .checkmate
            .as_ref()
            .is_none_or(|state| state.loser == winner)
        {
            // The final move lands before every defeated figure bows together.
            let delay = scene
                .travel
                .iter()
                .flatten()
                .map(|travel| travel.duration - travel.elapsed)
                .fold(0.0_f32, f32::max);
            scene.checkmate = Some(CheckmateCelebration {
                loser: !winner,
                elapsed: -delay,
            });
        }
    } else {
        scene.checkmate = None;
    }
    sync.characters_animating = scene.travel.iter().any(Option::is_some);
    scene.update_battle(sync.pending_move, sync.exploding, dt);

    let mut yielding: [Option<YieldSpace>; 64] = [None; 64];
    for (travel, to) in scene.travel.iter().zip(Square::ALL) {
        if let Some(travel) = travel.filter(|travel| travel.role == Role::Knight) {
            let route = KnightRoute::new(travel.from, square_world(to));
            for (waypoint, direction, amount) in route.clearances(travel.elapsed / travel.duration)
            {
                let Some(square) = square_from_world(waypoint) else {
                    continue;
                };
                let slot = &mut yielding[square.to_usize()];
                if amount > 0.0 && slot.is_none_or(|existing| existing.amount < amount) {
                    *slot = Some(YieldSpace { direction, amount });
                }
            }
        }
    }

    let scene = &mut *scene;
    for square in Square::ALL {
        let index = square.to_usize();
        let Some(mut piece) = game.piece_at(square) else {
            scene.remove(&mut commands, square);
            continue;
        };
        let mut position = square_world(square);
        let travel = scene.travel[index];
        if let Some(travel) = travel {
            // A promoted pawn finishes its stride before the new role appears.
            piece.role = travel.role;
        }
        let rest = rest_heading(piece, appearance.knight_orientation);
        let action =
            if let Some(action) = battle_pose(scene.battle.as_ref(), square, piece.role, rest) {
                Some(action)
            } else if let Some(travel) = travel {
                let (t, extension) = if piece.role == Role::Rook {
                    animation::rook_walk_phase(travel.elapsed, travel.duration)
                } else {
                    ((travel.elapsed / travel.duration).clamp(0.0, 1.0), 0.0)
                };
                let (path_position, facing, mut pose) = if piece.role == Role::Knight {
                    KnightRoute::new(travel.from, position).sample(t)
                } else {
                    (
                        animation::walk_position(travel.from, position, t),
                        animation::heading(travel.from, position),
                        animation::moving(piece.role, t, travel.distance),
                    )
                };
                pose.extension = extension;
                position = path_position;
                position.z += travel.lane_offset * (PI * t).sin();
                Some((rest.slerp(facing, animation::turn_fraction(t)), pose))
            } else if let Some(defeat) = scene
                .checkmate
                .as_ref()
                .filter(|defeat| defeat.loser == piece.color && defeat.elapsed >= 0.0)
            {
                Some((
                    rest,
                    animation::kneeling(piece.role, (defeat.elapsed / 0.85).min(1.0)),
                ))
            } else if let Some(YieldSpace { direction, amount }) = yielding[index] {
                let crowded = (direction.x == 0.0 || direction.z == 0.0)
                    && square_from_world(position + direction)
                        .is_none_or(|neighbor| game.piece_at(neighbor).is_some());
                let distance = if crowded {
                    0.44
                } else if matches!(piece.role, Role::Knight | Role::Rook) {
                    0.70
                } else {
                    0.63
                };
                position += direction * amount * distance;
                Some((
                    rest,
                    animation::yielding(piece.role, rest.inverse() * direction, amount),
                ))
            } else {
                None
            };

        let slot = &mut scene.board[index];
        let (heading, pose) = action.unwrap_or((rest, Pose::default()));
        let transform = Transform::from_translation(position)
            .with_rotation(heading)
            .with_scale(Vec3::splat(BOARD_SCALE));
        if let Some(visual) = slot.as_mut().filter(|visual| visual.piece == piece) {
            if action.is_some() || visual.animated || visual.rest != rest {
                if let Ok(mut root) = roots.get_mut(visual.rig.root) {
                    *root = transform;
                }
                visual.rig.pose(pose, &mut joints);
                visual.animated = action.is_some();
                visual.rest = rest;
            }
        } else {
            if let Some(visual) = slot.take() {
                commands.entity(visual.rig.root).despawn();
            }
            let rig = model::spawn(
                &mut commands,
                &kit,
                piece.role,
                team_material(piece, &handles),
                transform,
            );
            *slot = Some(Visual {
                piece,
                rig,
                rest,
                animated: action.is_some(),
            });
        }
    }

    let tray_view = TrayView {
        visible: appearance.show_captured,
        upright: appearance.captured_upright,
        knight_orientation: appearance.knight_orientation,
    };
    if game.is_changed() || scene.tray_view != Some(tray_view) {
        update_tray(&mut commands, &game, &kit, &handles, scene, tray_view);
    }
}

fn update_tray(
    commands: &mut Commands,
    game: &BattleGame,
    kit: &CharacterAssets,
    handles: &SceneHandles,
    scene: &mut CharacterScene,
    view: TrayView,
) {
    let pieces = if view.visible {
        game.captured_pieces()
            .into_iter()
            .map(|capture| (capture.piece, capture.captured_by))
            .collect()
    } else {
        Vec::new()
    };
    if scene.tray_view == Some(view) && scene.tray_pieces == pieces {
        return;
    }
    for entity in scene.tray_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    let mut counts = [0, 0];
    for &(piece, captor) in &pieces {
        let side = usize::from(captor == ChessColor::Black);
        let index = counts[side];
        counts[side] += 1;
        let x = -2.75 + (index % 8) as f32 * 0.78;
        let z = (5.10 + (index / 8) as f32 * 0.72) * if side == 0 { 1.0 } else { -1.0 };
        let lay = if view.upright {
            Quat::IDENTITY
        } else {
            Quat::from_rotation_x(-FRAC_PI_2)
        };
        let rotation = lay * rest_heading(piece, view.knight_orientation);
        let transform = Transform::from_xyz(x, if view.upright { 0.04 } else { 0.13 }, z)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(TRAY_SCALE));
        let rig = model::spawn(
            commands,
            kit,
            piece.role,
            team_material(piece, handles),
            transform,
        );
        scene.tray_entities.push(rig.root);
    }
    scene.tray_pieces = pieces;
    scene.tray_view = Some(view);
}
