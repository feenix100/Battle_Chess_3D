use std::time::Duration;

use shakmaty::{CastlingMode, Chess, fen::Fen};

use super::*;
use crate::{
    BattleSettings, ChessClock, CpuSettings, FxPause, InteractionState, capture_fx_system,
    commit_move, sync_scene_system,
};

const CASES: [(Role, &str, Square, Square, Square); 6] = [
    (
        Role::Pawn,
        "7k/8/8/3p4/4P3/8/8/K7 w - - 0 1",
        Square::E4,
        Square::E5,
        Square::D5,
    ),
    (
        Role::Knight,
        "7k/8/8/8/3N4/5n2/8/K7 w - - 0 1",
        Square::D4,
        Square::F5,
        Square::F3,
    ),
    (
        Role::Bishop,
        "8/7k/7p/4b3/8/2B5/K7/8 w - - 0 1",
        Square::C3,
        Square::D2,
        Square::E5,
    ),
    (
        Role::Rook,
        "7k/8/2r5/8/8/2R5/8/K7 w - - 0 1",
        Square::C3,
        Square::C4,
        Square::C6,
    ),
    (
        Role::Queen,
        "7k/8/5q2/8/3Q4/8/K7/8 w - - 0 1",
        Square::D4,
        Square::D5,
        Square::F6,
    ),
    (
        Role::King,
        "7k/8/8/3p4/4K3/8/8/8 w - - 0 1",
        Square::E4,
        Square::F4,
        Square::D5,
    ),
];

fn fixture(fen: &str) -> App {
    let position = fen
        .parse::<Fen>()
        .unwrap()
        .into_position::<Chess>(CastlingMode::Standard)
        .unwrap();
    let mut app = App::new();
    app.insert_resource(BattleGame::from_position(position))
        .insert_resource(SceneSync {
            dirty: true,
            ..default()
        })
        .insert_resource(SceneHandles {
            cube: default(),
            light_square: default(),
            dark_square: default(),
            board_base: default(),
            white_piece: default(),
            black_piece: default(),
            selected: default(),
            legal: default(),
            capture: default(),
            cursor: default(),
            fx: default(),
        })
        .init_resource::<Time>()
        .init_resource::<FxPause>()
        .init_resource::<InteractionState>()
        .init_resource::<Appearance>()
        .init_resource::<CharacterScene>()
        .init_resource::<CharacterAssets>()
        .add_systems(
            Update,
            (capture_fx_system, sync_scene_system, update_characters).chain(),
        );
    app.update();
    app
}

fn play(app: &mut App, from: Square, to: Square) {
    let world = app.world_mut();
    let mut game = world.remove_resource::<BattleGame>().unwrap();
    let mut scene = world.remove_resource::<SceneSync>().unwrap();
    let movement = game.legal_moves_between(from, to)[0];
    commit_move(
        &mut game,
        movement,
        &mut InteractionState::default(),
        &mut CpuSettings::default(),
        &ChessClock::default(),
        &BattleSettings::default(),
        &mut scene,
        &mut world.resource_mut::<FxPause>(),
    );
    world.insert_resource(game);
    world.insert_resource(scene);
    step(app, 0.0);
}

fn step(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

fn actor(app: &App, square: Square) -> &Visual {
    app.world().resource::<CharacterScene>().board[square.to_usize()]
        .as_ref()
        .unwrap()
}

fn body(app: &App, square: Square) -> Transform {
    let root = actor(app, square).rig.root;
    let joint = app.world().get::<Children>(root).unwrap()[0];
    *app.world().get::<Transform>(joint).unwrap()
}

#[test]
fn street_queen_moves_on_files_ranks_and_diagonals_with_a_skirt_sway() {
    for to in [Square::D6, Square::G3, Square::G6] {
        let mut app = fixture("7k/7p/8/8/8/3Q4/8/K7 w - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, Square::D3).rig.root;
        let rest = body(&app, Square::D3);
        play(&mut app, Square::D3, to);
        step(&mut app, 0.17);
        let p = app.world().get::<Transform>(root).unwrap().translation;
        let from = square_world(Square::D3);
        let end = square_world(to);
        assert!((p - from).cross(end - from).length() < 0.0001);
        assert!(p.distance(from) > 0.02 && p.distance(end) > 0.02);
        assert_ne!(body(&app, to), rest);
        assert!(
            animation::street_queen_moving(0.34, from.distance(end))
                .cloth
                .abs()
                > 0.01
        );
        step(&mut app, 1.0);
        assert_eq!(app.world().get::<Transform>(root).unwrap().translation, end);
        assert_eq!(body(&app, to), rest);
    }
}
#[test]
fn street_queen_smg_sprays_ten_rounds_then_commits_once_for_both_colors_and_lowriders() {
    for (fen, from, to, low) in [
        (
            "7k/7p/8/8/2Q2p2/8/8/K7 w - - 0 1",
            Square::C4,
            Square::F4,
            false,
        ),
        (
            "7k/7p/8/2P2q2/8/8/8/K7 b - - 0 1",
            Square::F5,
            Square::C5,
            false,
        ),
        (
            "7k/7p/8/8/2Q2n2/8/8/K7 w - - 0 1",
            Square::C4,
            Square::F4,
            true,
        ),
    ] {
        let mut app = fixture(fen);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let attacker = actor(&app, from).rig.root;
        let victim = actor(&app, to).rig.root;
        let upright = body(&app, to);
        play(&mut app, from, to);
        let world = app.world_mut();
        let mut shots: Vec<_> = world.query::<&crate::Projectile>().iter(world).collect();
        shots.sort_by_key(|p| p.index);
        assert_eq!(shots.len(), 10);
        let forward = (square_world(to) - square_world(from)).normalize();
        let distance = square_world(from).distance(square_world(to));
        let aim = crate::projectiles::queen_smg_aim(distance, if low { 0.28 } else { 0.56 });
        for (i, shot) in shots.iter().enumerate() {
            assert!(shot.is_queen_bullet());
            let seconds = 0.46 + i as f32 * 0.07;
            assert!((shot.elapsed + seconds).abs() < 0.0001);
            let m = queen_muzzle(seconds, distance, aim);
            let expected =
                square_world(from) + forward.cross(Vec3::Y) * m.x + Vec3::Y * m.y - forward * m.z;
            assert!(shot.start.distance(expected) < 0.0001);
            assert!(shot.position(0.5).distance(shot.start.lerp(shot.end, 0.5)) < 0.0001);
            if low {
                assert!(shot.end.y < 0.40);
            }
        }
        assert!(
            shots[2].end.distance(shots[7].end) > 0.12,
            "spray sweeps across the defender"
        );
        let first = 0.46 + crate::projectiles::queen_smg_flight(distance);
        while app
            .world()
            .resource::<CharacterScene>()
            .battle
            .as_ref()
            .unwrap()
            .elapsed
            < first + 0.02
        {
            step(&mut app, 0.01);
        }
        assert!(
            !app.world().resource::<SceneSync>().exploding,
            "early rounds flinch, without defeating"
        );
        assert_ne!(body(&app, to), upright);
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
        for _ in 0..100 {
            if app.world().resource::<SceneSync>().exploding {
                break;
            }
            step(&mut app, 0.01);
        }
        assert!(app.world().resource::<SceneSync>().exploding);
        assert_eq!(actor(&app, to).rig.root, victim);
        step(&mut app, 0.16);
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
        for _ in 0..50 {
            step(&mut app, 0.05);
        }
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
        assert_eq!(actor(&app, to).rig.root, attacker);
        assert!(app.world().get_entity(victim).is_err());
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        assert_eq!(world.query::<&crate::SmgEffect>().iter(world).count(), 0);
    }
}
#[test]
fn street_queen_reset_clears_rounds_flashes_casings_and_smoke_during_each_phase() {
    for age in [0.20, 0.54, 1.13, 1.55] {
        let mut app = fixture("7k/7p/8/8/2Q2p2/8/8/K7 w - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        play(&mut app, Square::C4, Square::F4);
        for _ in 0..(age / 0.01) as u32 {
            step(&mut app, 0.01);
        }
        let world = app.world_mut();
        let mut ids: Vec<_> = world
            .query_filtered::<Entity, Or<(With<crate::Projectile>, With<crate::Debris>)>>()
            .iter(world)
            .collect();
        let children: Vec<_> = ids
            .iter()
            .flat_map(|e| {
                world
                    .get::<Children>(*e)
                    .map(|c| c.to_vec())
                    .unwrap_or_default()
            })
            .collect();
        ids.extend(children);
        world
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        step(&mut app, 0.0);
        let world = app.world_mut();
        assert!(ids.into_iter().all(|e| world.get_entity(e).is_err()));
        assert_eq!(world.resource::<BattleGame>().history_len(), 0);
        assert_eq!(world.query::<&crate::SmgEffect>().iter(world).count(), 0);
        assert!(world.resource::<SceneSync>().pending_move.is_none());
    }
}
#[test]
fn street_queen_promotion_and_set_switch_use_the_twelve_joint_skin_in_board_and_tray() {
    fn assets(app: &mut App) {
        let mut meshes = Assets::<Mesh>::default();
        let mut bindposes = Assets::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let kit = CharacterAssets::new(&mut meshes, &mut bindposes, &mut materials);
        app.insert_resource(kit)
            .insert_resource(meshes)
            .insert_resource(bindposes)
            .insert_resource(materials);
        app.world_mut().resource_mut::<SceneSync>().reset_characters = true;
    }
    let mut app = fixture("7k/P6p/8/8/8/8/8/K7 w - - 0 1");
    assets(&mut app);
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let world = app.world_mut();
    let mut game = world.remove_resource::<BattleGame>().unwrap();
    let mut sync = world.remove_resource::<SceneSync>().unwrap();
    let movement = game
        .legal_moves_between(Square::A7, Square::A8)
        .into_iter()
        .find(|mv| mv.promotion() == Some(Role::Queen))
        .unwrap();
    commit_move(
        &mut game,
        movement,
        &mut InteractionState::default(),
        &mut CpuSettings::default(),
        &ChessClock::default(),
        &BattleSettings::default(),
        &mut sync,
        &mut world.resource_mut::<FxPause>(),
    );
    world.insert_resource(game);
    world.insert_resource(sync);
    step(&mut app, 0.0);
    assert_eq!(actor(&app, Square::A8).piece.role, Role::Pawn);
    step(&mut app, 1.0);
    assert_eq!(actor(&app, Square::A8).piece.role, Role::Queen);
    let root = actor(&app, Square::A8).rig.root;
    let body = app.world().get::<Children>(root).unwrap()[0];
    assert_eq!(
        app.world().get::<Children>(body).unwrap().len(),
        6,
        "garment joint is spawned"
    );
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::Original;
    step(&mut app, 0.0);
    assert!(app.world().get_entity(root).is_err());
    let mut app = fixture("7k/7p/2q5/8/8/2R5/8/K7 w - - 0 1");
    assets(&mut app);
    step(&mut app, 0.0);
    play(&mut app, Square::C3, Square::C6);
    for _ in 0..110 {
        step(&mut app, 0.05);
    }
    let original = app.world().resource::<CharacterScene>().tray_entities[0];
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let gta = app.world().resource::<CharacterScene>().tray_entities[0];
    assert_ne!(original, gta);
    assert!(app.world().get_entity(original).is_err());
    let body = app.world().get::<Children>(gta).unwrap()[0];
    assert_eq!(app.world().get::<Children>(body).unwrap().len(), 6);
}
#[test]
fn street_queen_addition_preserves_original_artillery_capture() {
    let mut app = fixture("7k/7p/8/8/2Q2p2/8/8/K7 w - - 0 1");
    play(&mut app, Square::C4, Square::F4);
    let world = app.world_mut();
    let shots: Vec<_> = world.query::<&crate::Projectile>().iter(world).collect();
    assert_eq!(shots.len(), 8);
    assert!(shots.into_iter().all(|p| !p.is_queen_bullet()));
    for _ in 0..95 {
        step(&mut app, 0.05);
    }
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
}

#[test]
fn street_queen_casings_land_beside_her_and_all_opponents_get_fluid_and_smoke() {
    for target in ["p", "n", "r"] {
        let fen = format!("7k/7p/8/8/2Q2{target}2/8/8/K7 w - - 0 1");
        let mut app = fixture(&fen);
        app.add_systems(
            Update,
            crate::projectiles::animate_smg.after(update_characters),
        );
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        play(&mut app, Square::C4, Square::F4);
        for _ in 0..98 {
            step(&mut app, 0.01);
        }
        let world = app.world_mut();
        let mut settled = 0;
        let mut fluid = 0;
        let mut smoke = 0;
        for (effect, t) in world.query::<(&crate::SmgEffect, &Transform)>().iter(world) {
            match effect {
                crate::SmgEffect::Casing {
                    center,
                    ground,
                    landed,
                    ..
                } => {
                    assert!(
                        Vec2::new(t.translation.x - center.x, t.translation.z - center.z).length()
                            <= 0.431
                    );
                    if *landed {
                        settled += 1;
                        assert!((t.translation.y - ground).abs() < 0.0001);
                    }
                }
                crate::SmgEffect::Fluid { .. } => fluid += 1,
                crate::SmgEffect::Smoke { .. } => smoke += 1,
            }
        }
        assert!(settled > 0, "casings reach the board beside the queen");
        assert!(fluid > 0 && smoke > 0, "same hit effect for every opponent");
        for _ in 0..150 {
            if app.world().resource::<SceneSync>().pending_move.is_none() {
                break;
            }
            step(&mut app, 0.01);
        }
        let world = app.world_mut();
        assert_eq!(world.resource::<BattleGame>().history_len(), 1);
        assert!(
            world
                .query::<&crate::SmgEffect>()
                .iter(world)
                .any(|e| e.is_settled_casing()),
            "landed casings may remain as the queen moves"
        );
    }
}

fn king_gun(app: &App, square: Square) -> Transform {
    let root = actor(app, square).rig.root;
    let body = app.world().get::<Children>(root).unwrap()[0];
    let upper = app.world().get::<Children>(body).unwrap()[2];
    let lower = app.world().get::<Children>(upper).unwrap()[0];
    let weapon = app.world().get::<Children>(lower).unwrap()[0];
    *app.world().get::<Transform>(weapon).unwrap()
}
#[test]
fn street_king_walk_and_castle_keep_the_shotgun_carried() {
    for (fen, from, to) in [
        ("7k/7p/8/8/3K4/8/8/8 w - - 0 1", Square::D4, Square::E4),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            Square::E1,
            Square::G1,
        ),
    ] {
        let mut app = fixture(fen);
        let original = actor(&app, from).rig.root;
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, from).rig.root;
        assert_ne!(root, original);
        let gun = king_gun(&app, from);
        play(&mut app, from, to);
        step(&mut app, 0.19);
        assert_eq!(king_gun(&app, to).rotation, gun.rotation);
        assert!(
            app.world()
                .get::<Transform>(root)
                .unwrap()
                .translation
                .distance(square_world(from))
                > 0.02
        );
        step(&mut app, 1.0);
        assert_eq!(
            app.world().get::<Transform>(root).unwrap().translation,
            square_world(to)
        );
        assert_eq!(king_gun(&app, to), gun);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::Original;
        step(&mut app, 0.0);
        assert!(app.world().get_entity(root).is_err());
    }
}
#[test]
fn street_king_spread_blast_commits_once_for_both_colors_and_lowrider_targets() {
    for (fen, from, to, low) in [
        (
            "7k/7p/8/3p4/4K3/8/8/8 w - - 0 1",
            Square::E4,
            Square::D5,
            false,
        ),
        (
            "8/8/8/4k3/3P4/8/P7/7K b - - 0 1",
            Square::E5,
            Square::D4,
            false,
        ),
        (
            "7k/7p/8/3n4/4K3/8/8/8 w - - 0 1",
            Square::E4,
            Square::D5,
            true,
        ),
    ] {
        let mut app = fixture(fen);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, from).rig.root;
        let victim = actor(&app, to).rig.root;
        let carry = king_gun(&app, from);
        play(&mut app, from, to);
        let world = app.world_mut();
        let shots: Vec<_> = world.query::<&crate::Projectile>().iter(world).collect();
        assert_eq!(shots.len(), 9);
        assert!(shots.iter().all(|p| p.is_king_pellet()));
        let forward = (square_world(to) - square_world(from)).normalize();
        let height = if low { 0.28 } else { 0.74 };
        let aim = crate::projectiles::king_shotgun_aim(
            square_world(to).distance(square_world(from)),
            height,
        );
        let muzzle = king_muzzle(0.68, aim);
        let expected = square_world(from) + forward.cross(Vec3::Y) * muzzle.x + Vec3::Y * muzzle.y
            - forward * muzzle.z;
        for shot in &shots {
            assert!(shot.start.distance(expected) < 0.0001);
            assert!(shot.position(0.5).distance(shot.start.lerp(shot.end, 0.5)) < 0.0001);
        }
        if low {
            assert!(shots[0].start.y > shots[0].end.y + 0.05);
        }
        assert!(shots[0].end.distance(shots[1].end) > 0.05);
        step(&mut app, 0.54);
        assert_ne!(king_gun(&app, from).rotation, carry.rotation);
        let world = app.world_mut();
        assert_eq!(
            world.query::<&crate::ShotgunCloud>().iter(world).count(),
            0,
            "no blast before lowering finishes"
        );
        step(&mut app, 0.15);
        let world = app.world_mut();
        assert_eq!(
            world.query::<&crate::ShotgunCloud>().iter(world).count(),
            4,
            "one muzzle flash and three smoke puffs"
        );
        step(&mut app, 0.20);
        assert!(app.world().resource::<SceneSync>().exploding);
        step(&mut app, 0.15);
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
        for _ in 0..45 {
            step(&mut app, 0.05);
        }
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
        assert_eq!(actor(&app, to).rig.root, root);
        assert!(app.world().get_entity(victim).is_err());
        assert_eq!(king_gun(&app, to).rotation, carry.rotation);
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::BombBlast>().iter(world).count(), 0);
        assert_eq!(world.query::<&crate::ShotgunCloud>().iter(world).count(), 0);
    }
}
#[test]
fn street_king_cancellation_clears_pellets_smoke_and_shell_children() {
    for age in [0.22, 0.72, 0.96] {
        let mut app = fixture("8/8/8/4k3/3P4/8/P7/7K b - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        play(&mut app, Square::E5, Square::D4);
        for _ in 0..(age / 0.01) as u32 {
            step(&mut app, 0.01);
        }
        let world = app.world_mut();
        let mut ids: Vec<_> = world
            .query_filtered::<Entity, Or<(With<crate::Projectile>, With<crate::Debris>)>>()
            .iter(world)
            .collect();
        let children: Vec<_> = ids
            .iter()
            .flat_map(|e| {
                world
                    .get::<Children>(*e)
                    .map(|c| c.to_vec())
                    .unwrap_or_default()
            })
            .collect();
        ids.extend(children);
        world
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        step(&mut app, 0.0);
        let world = app.world_mut();
        assert!(ids.into_iter().all(|e| world.get_entity(e).is_err()));
        assert_eq!(world.query::<&crate::ShotgunCloud>().iter(world).count(), 0);
        assert_eq!(world.resource::<BattleGame>().history_len(), 0);
    }
}
#[test]
fn street_king_checkmate_bow_preserves_shoulder_carry() {
    let mut app = fixture("7k/p7/5KQ1/8/8/8/8/8 w - - 0 1");
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let gun = king_gun(&app, Square::H8);
    let upright = body(&app, Square::H8);
    play(&mut app, Square::G6, Square::G7);
    step(&mut app, 1.5);
    assert!(app.world().resource::<CharacterScene>().checkmate.is_some());
    assert_ne!(body(&app, Square::H8), upright);
    assert_eq!(king_gun(&app, Square::H8), gun);
}
#[test]
fn street_king_addition_preserves_original_king_bomb_capture() {
    let mut app = fixture("7k/8/8/3p4/4K3/8/8/8 w - - 0 1");
    play(&mut app, Square::E4, Square::D5);
    let world = app.world_mut();
    assert!(
        world
            .query::<&crate::Projectile>()
            .iter(world)
            .all(|p| !p.is_king_pellet())
    );
    for _ in 0..100 {
        step(&mut app, 0.05);
    }
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
}

#[test]
fn police_bishop_patrol_moves_diagonally_and_set_switch_rebuilds_board_and_tray() {
    let mut app = fixture("8/7k/7p/8/3B4/8/8/K7 w - - 0 1");
    let original = actor(&app, Square::D4).rig.root;
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let root = actor(&app, Square::D4).rig.root;
    assert_ne!(root, original);
    let rest = body(&app, Square::D4);
    play(&mut app, Square::D4, Square::G7);
    step(&mut app, 0.18);
    let p = app.world().get::<Transform>(root).unwrap().translation;
    let from = square_world(Square::D4);
    assert!((p.x - from.x).abs() > 0.02);
    assert!(((p.x - from.x).abs() - (p.z - from.z).abs()).abs() < 0.0001);
    assert_ne!(body(&app, Square::G7), rest);
    step(&mut app, 1.2);
    assert_eq!(
        app.world().get::<Transform>(root).unwrap().translation,
        square_world(Square::G7)
    );
    assert_eq!(body(&app, Square::G7), rest);
    // Capture a bishop so the tray must also rebuild when the style changes.
    let mut app = fixture("7k/7p/2b5/8/8/2R5/8/K7 w - - 0 1");
    play(&mut app, Square::C3, Square::C6);
    for _ in 0..70 {
        step(&mut app, 0.05);
    }
    let tray = app.world().resource::<CharacterScene>().tray_entities[0];
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let gta = app.world().resource::<CharacterScene>().tray_entities[0];
    assert_ne!(tray, gta);
    assert!(app.world().get_entity(tray).is_err());
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::Original;
    step(&mut app, 0.0);
    assert!(app.world().get_entity(gta).is_err());
}

#[test]
fn police_bishop_two_darts_stun_and_commit_once_for_both_colors() {
    for (fen, from, to) in [
        ("7k/7p/5p2/8/3B4/8/8/K7 w - - 0 1", Square::D4, Square::F6),
        ("7k/7p/2b5/8/8/5P2/P7/K7 b - - 0 1", Square::C6, Square::F3),
    ] {
        let mut app = fixture(fen);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, from).rig.root;
        let victim = actor(&app, to).rig.root;
        play(&mut app, from, to);
        let world = app.world_mut();
        let shots: Vec<_> = world.query::<&crate::Projectile>().iter(world).collect();
        assert_eq!(shots.len(), 2);
        let forward = (square_world(to) - square_world(from)).normalize();
        let flight =
            crate::projectiles::taser_flight_seconds(square_world(from).distance(square_world(to)));
        let m = police_muzzle(0.34, flight);
        let muzzle =
            square_world(from) + forward.cross(Vec3::Y) * m.x + Vec3::Y * m.y - forward * m.z;
        for shot in shots {
            assert!(shot.is_taser_dart());
            assert!(shot.start.distance(muzzle) < 0.009);
            assert!(shot.position(0.5).distance(shot.start.lerp(shot.end, 0.5)) < 0.0001);
        }
        for _ in 0..30 {
            if app.world().resource::<SceneSync>().exploding {
                break;
            }
            step(&mut app, 0.025);
        }
        assert!(app.world().resource::<SceneSync>().exploding);
        step(&mut app, 0.15);
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
        assert_eq!(actor(&app, to).rig.root, victim);
        assert!(
            body(&app, to).rotation.angle_between(Quat::IDENTITY) < 0.15,
            "stun stays upright"
        );
        step(&mut app, 0.40);
        assert!(
            body(&app, to).rotation.angle_between(Quat::IDENTITY) > 0.15,
            "fall follows stun"
        );
        for _ in 0..45 {
            step(&mut app, 0.05);
        }
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
        assert_eq!(actor(&app, to).rig.root, root);
        assert!(app.world().get_entity(victim).is_err());
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::TaserCircuit>().iter(world).count(), 0);
    }
}

#[test]
fn police_bishop_cancellation_clears_darts_cables_and_pending_capture() {
    for age in [0.10, 0.46, 0.83] {
        let mut app = fixture("7k/7p/5p2/8/3B4/8/8/K7 w - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        play(&mut app, Square::D4, Square::F6);
        for _ in 0..(age / 0.01) as u32 {
            step(&mut app, 0.01);
        }
        app.world_mut()
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        step(&mut app, 0.0);
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        assert_eq!(world.query::<&crate::TaserCircuit>().iter(world).count(), 0);
        assert_eq!(
            world
                .query::<&crate::projectiles::TaserPart>()
                .iter(world)
                .count(),
            0
        );
        assert_eq!(world.resource::<BattleGame>().history_len(), 0);
        assert!(world.resource::<SceneSync>().pending_move.is_none());
    }
}

#[test]
fn police_bishop_addition_preserves_original_bishop_capture() {
    let mut app = fixture(CASES[2].1);
    play(&mut app, Square::C3, Square::E5);
    let world = app.world_mut();
    assert!(
        world
            .query::<&crate::Projectile>()
            .iter(world)
            .all(|shot| !shot.is_taser_dart())
    );
    for _ in 0..55 {
        step(&mut app, 0.05);
    }
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
}

#[test]
fn mansion_walk_moves_straight_unfolds_legs_and_castles() {
    for to in [Square::D7, Square::D1, Square::A4, Square::G4] {
        let mut app = fixture("7k/7p/8/8/3R4/8/8/K7 w - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, Square::D4).rig.root;
        let base = body(&app, Square::D4);
        let joint = app.world().get::<Children>(root).unwrap()[0];
        assert_eq!(app.world().get::<Children>(joint).unwrap().len(), 3);
        play(&mut app, Square::D4, to);
        step(&mut app, 0.23);
        let p = app.world().get::<Transform>(root).unwrap().translation;
        let from = square_world(Square::D4);
        let end = square_world(to);
        assert!(p.distance(from) > 0.02 && p.distance(end) > 0.02);
        assert!((p.y - from.y).abs() < 0.0001);
        if from.x == end.x {
            assert!((p.x - from.x).abs() < 0.0001);
        } else {
            assert!((p.z - from.z).abs() < 0.0001);
        }
        assert!(
            body(&app, to).translation.y - base.translation.y > 0.30,
            "house rises above its legs"
        );
        let leg_nodes: Vec<_> = app
            .world()
            .get::<Children>(root)
            .unwrap()
            .iter()
            .skip(1)
            .collect();
        assert_eq!(leg_nodes.len(), 6);
        assert!(
            leg_nodes
                .iter()
                .all(|e| app.world().get::<Transform>(*e).unwrap().scale.x > 0.9)
        );
        step(&mut app, 1.2);
        assert_eq!(app.world().get::<Transform>(root).unwrap().translation, end);
        assert_eq!(body(&app, to), base);
    }
    let mut app = fixture("7k/7p/8/8/8/8/8/4K2R w K - 0 1");
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let king = actor(&app, Square::E1).rig.root;
    let rook = actor(&app, Square::H1).rig.root;
    play(&mut app, Square::E1, Square::G1);
    step(&mut app, 1.2);
    assert_eq!(actor(&app, Square::G1).rig.root, king);
    assert_eq!(actor(&app, Square::F1).rig.root, rook);
    assert_eq!(
        app.world().get::<Transform>(rook).unwrap().translation,
        square_world(Square::F1)
    );
}

#[test]
fn mansion_walk_preserves_molotov_arc_fire_and_single_capture_for_both_colors() {
    for (fen, from, to, color) in [
        (
            "7k/7p/2p5/8/8/2R5/8/K7 w - - 0 1",
            Square::C3,
            Square::C6,
            ChessColor::White,
        ),
        (
            "7k/8/5r2/8/8/5P2/P7/K7 b - - 0 1",
            Square::F6,
            Square::F3,
            ChessColor::Black,
        ),
    ] {
        let mut app = fixture(fen);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, from).rig.root;
        play(&mut app, from, to);
        let world = app.world_mut();
        let shot = world.query::<&crate::Projectile>().single(world).unwrap();
        assert!(shot.is_molotov());
        let forward = (square_world(to) - square_world(from)).normalize();
        let anchor = mansion_launch(0.42);
        let expected = square_world(from) + forward.cross(Vec3::Y) * anchor.x + Vec3::Y * anchor.y
            - forward * anchor.z;
        assert!(shot.start.distance(expected) < 0.0001);
        assert!(shot.position(0.5).y > shot.start.lerp(shot.end, 0.5).y + 0.60);
        assert!(
            shot.position(0.0).distance(shot.start) < 0.0001
                && shot.position(1.0).distance(shot.end) < 0.0001
        );
        assert!(shot.rotation(0.25).angle_between(shot.rotation(0.75)) > 0.5);
        step(&mut app, 0.20);
        let body_joint = app.world().get::<Children>(root).unwrap()[0];
        let hatch = app.world().get::<Children>(body_joint).unwrap()[0];
        assert!(
            app.world()
                .get::<Transform>(hatch)
                .unwrap()
                .rotation
                .to_euler(EulerRot::XYZ)
                .0
                > 1.0
        );
        step(&mut app, 1.05);
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
        step(&mut app, 0.08);
        let world = app.world_mut();
        let (fire, transform) = world
            .query_filtered::<(Entity, &Transform), With<crate::MolotovFire>>()
            .single(world)
            .unwrap();
        let ground = *transform;
        assert!((ground.translation.y - 0.122).abs() < 0.0001);
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        step(&mut app, 0.50);
        assert_eq!(*app.world().get::<Transform>(fire).unwrap(), ground);
        assert_eq!(
            app.world().resource::<BattleGame>().history_len(),
            0,
            "wait for fire to expire"
        );
        for _ in 0..80 {
            step(&mut app, 0.025);
        }
        let world = app.world_mut();
        assert_eq!(world.resource::<BattleGame>().history_len(), 1);
        assert_eq!(
            world.resource::<BattleGame>().piece_at(to),
            Some(color.rook())
        );
        assert_eq!(world.query::<&crate::MolotovFire>().iter(world).count(), 0);
        assert_eq!(
            world
                .query::<&crate::projectiles::MolotovPart>()
                .iter(world)
                .count(),
            0
        );
        assert_eq!(world.query::<&crate::Debris>().iter(world).count(), 0);
        assert_eq!(actor(&app, to).rig.root, root);
        assert_eq!(
            app.world().get::<Transform>(hatch).unwrap().rotation,
            Quat::IDENTITY
        );
    }
}

#[test]
fn mansion_new_game_clears_bottle_flames_smoke_and_fire() {
    for elapsed in [0.18, 0.75, 1.50] {
        let mut app = fixture("7k/7p/2p5/8/8/2R5/8/K7 w - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        play(&mut app, Square::C3, Square::C6);
        step(&mut app, elapsed);
        app.world_mut().resource_mut::<BattleGame>().reset();
        app.world_mut()
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        step(&mut app, 0.0);
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        assert_eq!(world.query::<&crate::MolotovFire>().iter(world).count(), 0);
        assert_eq!(
            world
                .query::<&crate::projectiles::MolotovPart>()
                .iter(world)
                .count(),
            0
        );
        assert_eq!(world.query::<&crate::Debris>().iter(world).count(), 0);
        assert_eq!(world.resource::<BattleGame>().history_len(), 0);
    }
}

#[test]
fn mansion_set_switch_refreshes_board_and_captured_tray() {
    let mut app = fixture("7k/8/2r5/8/8/2R5/8/K7 w - - 0 1");
    let original = actor(&app, Square::C3).rig.root;
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    assert_ne!(actor(&app, Square::C3).rig.root, original);
    assert_eq!(body(&app, Square::C3).translation.y, 0.43);
    play(&mut app, Square::C3, Square::C6);
    for _ in 0..160 {
        step(&mut app, 0.025);
    }
    let current = actor(&app, Square::C6).rig.root;
    let tray = app.world().resource::<CharacterScene>().tray_entities[0];
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::Original;
    step(&mut app, 0.0);
    assert_ne!(actor(&app, Square::C6).rig.root, current);
    assert!(app.world().get_entity(tray).is_err());
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
    assert_eq!(
        app.world()
            .resource::<CharacterScene>()
            .tray_view
            .unwrap()
            .style,
        PieceStyle::Original
    );
}

#[test]
fn lowrider_drives_all_eight_knight_routes_with_grounded_wheels() {
    let from = square_world(Square::D4);
    for (dx, dz) in [
        (-2.0, -1.0),
        (-2.0, 1.0),
        (2.0, -1.0),
        (2.0, 1.0),
        (-1.0, -2.0),
        (-1.0, 2.0),
        (1.0, -2.0),
        (1.0, 2.0),
    ] {
        let to = from + Vec3::new(dx, 0.0, dz);
        assert!(animation::lowrider_route(from, to, 0.0).0.distance(from) < 0.0001);
        assert!(animation::lowrider_route(from, to, 1.0).0.distance(to) < 0.0001);
        for i in 0..=100 {
            let (p, q, pose) = animation::lowrider_route(from, to, i as f32 / 100.0);
            assert!((p.y - from.y).abs() < 0.0001 && q.is_finite());
            assert!(pose.offset.y.abs() < 0.04 && pose.body.length() < 0.09);
            assert_eq!(pose.vehicle.trunk, 0.0);
        }
        assert!(
            animation::lowrider_route(from, to, 0.85)
                .2
                .vehicle
                .wheel_roll
                .abs()
                > 10.0
        );
    }
    let mut app = fixture(CASES[1].1);
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    let root = actor(&app, Square::D4).rig.root;
    play(&mut app, Square::D4, Square::F5);
    step(&mut app, 0.32);
    let wheels: Vec<_> = app
        .world()
        .get::<Children>(root)
        .unwrap()
        .iter()
        .skip(1)
        .collect();
    assert_eq!(wheels.len(), 4);
    for wheel in wheels {
        let transform = app.world().get::<Transform>(wheel).unwrap();
        assert!((transform.translation.y - 0.137).abs() < 0.0001);
        let rolling = app.world().get::<Children>(wheel).unwrap()[0];
        assert_ne!(
            app.world().get::<Transform>(rolling).unwrap().rotation,
            Quat::IDENTITY
        );
    }
    step(&mut app, 1.1);
    assert_eq!(
        app.world().get::<Transform>(root).unwrap().translation,
        square_world(Square::F5)
    );
}

#[test]
fn lowrider_trunk_launch_drone_capture_and_cleanup_work_for_both_colors() {
    for (fen, from, to, color) in [
        (
            "7k/8/8/8/3N4/5p2/8/K7 w - - 0 1",
            Square::D4,
            Square::F3,
            ChessColor::White,
        ),
        (
            "7k/8/5P2/3n4/8/8/8/K7 b - - 0 1",
            Square::D5,
            Square::F6,
            ChessColor::Black,
        ),
    ] {
        let mut app = fixture(fen);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, from).rig.root;
        play(&mut app, from, to);
        let world = app.world_mut();
        let drone = world.query::<&crate::Projectile>().single(world).unwrap();
        assert!(drone.is_lowrider_drone());
        let start = drone.start;
        let forward = (square_world(to) - square_world(from)).normalize();
        let local = crate::characters::lowrider_launch(0.36);
        let expected = square_world(from) + forward.cross(Vec3::Y) * local.x + Vec3::Y * local.y
            - forward * local.z;
        assert!(start.distance(expected) < 0.0001);
        assert!((drone.duration - 1.38).abs() < 0.001);
        assert_eq!(world.query::<&LowriderRotor>().iter(world).count(), 4);
        assert_eq!(world.query::<&LowriderArm>().iter(world).count(), 4);
        step(&mut app, 0.20);
        let chassis = app.world().get::<Children>(root).unwrap()[0];
        let trunk = app.world().get::<Children>(chassis).unwrap()[0];
        let angle = app
            .world()
            .get::<Transform>(trunk)
            .unwrap()
            .rotation
            .to_euler(EulerRot::XYZ)
            .0;
        assert!(angle < -1.0, "trunk opens before lift-off");
        step(&mut app, 0.34);
        let world = app.world_mut();
        let drone = world.query::<&crate::Projectile>().single(world).unwrap();
        assert!(drone.position(drone.elapsed / drone.duration).y > start.y + 0.20);
        assert_eq!(world.resource::<BattleGame>().history_len(), 0);
        step(&mut app, 0.92);
        assert_eq!(
            app.world().resource::<BattleGame>().history_len(),
            0,
            "wait for diving impact"
        );
        assert!(!app.world().resource::<SceneSync>().exploding);
        step(&mut app, 0.30);
        assert!(app.world().resource::<SceneSync>().exploding);
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        assert_eq!(world.query::<&LowriderRotor>().iter(world).count(), 0);
        for _ in 0..100 {
            step(&mut app, 0.025);
        }
        let world = app.world_mut();
        assert_eq!(world.resource::<BattleGame>().history_len(), 1);
        assert_eq!(
            world.resource::<BattleGame>().piece_at(to),
            Some(color.knight())
        );
        assert_eq!(world.query::<&crate::Debris>().iter(world).count(), 0);
        assert_eq!(actor(&app, to).rig.root, root);
        let chassis = app.world().get::<Children>(root).unwrap()[0];
        let trunk = app.world().get::<Children>(chassis).unwrap()[0];
        assert_eq!(
            app.world().get::<Transform>(trunk).unwrap().rotation,
            Quat::IDENTITY
        );
    }
}

#[test]
fn lowrider_new_game_clears_deploying_and_diving_drones() {
    for elapsed in [0.20, 0.80, 1.86] {
        let mut app = fixture("7k/8/8/8/3N4/5p2/8/K7 w - - 0 1");
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        play(&mut app, Square::D4, Square::F3);
        step(&mut app, elapsed);
        app.world_mut().resource_mut::<BattleGame>().reset();
        app.world_mut()
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        step(&mut app, 0.0);
        let world = app.world_mut();
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        assert_eq!(world.query::<&crate::Debris>().iter(world).count(), 0);
        assert_eq!(world.query::<&LowriderRotor>().iter(world).count(), 0);
        assert_eq!(world.query::<&LowriderArm>().iter(world).count(), 0);
        assert_eq!(world.resource::<BattleGame>().history_len(), 0);
    }
}

#[test]
fn lowrider_set_switch_and_target_height_match_the_new_vehicle() {
    let mut app = fixture("7k/8/8/3n4/4P3/8/8/K7 w - - 0 1");
    let original = actor(&app, Square::D5).rig.root;
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    assert_ne!(actor(&app, Square::D5).rig.root, original);
    assert_eq!(body(&app, Square::D5).translation.y, 0.245);
    play(&mut app, Square::E4, Square::D5);
    let world = app.world_mut();
    for shot in world.query::<&crate::Projectile>().iter(world) {
        assert!(
            (shot.end.y - square_world(Square::D5).y - 0.28).abs() < 0.011,
            "other projectiles must hit the low car rather than pass above its roof"
        );
    }
    for _ in 0..100 {
        step(&mut app, 0.025);
    }
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::Original;
    step(&mut app, 0.0);
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
    assert_eq!(
        app.world()
            .resource::<CharacterScene>()
            .tray_view
            .unwrap()
            .style,
        PieceStyle::Original
    );
}

#[test]
fn all_roles_travel_between_squares_with_body_motion() {
    for (role, fen, from, to, _) in CASES {
        let mut app = fixture(fen);
        let original = actor(&app, from).rig.root;
        let rest = body(&app, from);
        play(&mut app, from, to);
        assert_eq!(actor(&app, to).rig.root, original, "{role:?}");
        assert_eq!(
            app.world().get::<Transform>(original).unwrap().translation,
            square_world(from)
        );
        step(&mut app, if role == Role::Rook { 0.30 } else { 0.22 });
        let position = app.world().get::<Transform>(original).unwrap().translation;
        assert!(position.distance(square_world(from)) > 0.1, "{role:?}");
        assert!(position.distance(square_world(to)) > 0.1, "{role:?}");
        assert_ne!(body(&app, to), rest, "{role:?}");
        step(&mut app, 1.0);
        assert_eq!(body(&app, to), rest);
        assert_eq!(
            app.world().get::<Transform>(original).unwrap().translation,
            square_world(to)
        );
    }
}

#[test]
fn piece_set_switch_rebuilds_board_and_tray_without_changing_the_game() {
    let mut app = fixture(CASES[0].1);
    let original = actor(&app, Square::E4).rig.root;
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
    step(&mut app, 0.0);
    assert_ne!(actor(&app, Square::E4).rig.root, original);
    assert!(app.world().get_entity(original).is_err());
    assert_eq!(actor(&app, Square::E4).style, PieceStyle::GtaStreet);
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
    play(&mut app, Square::E4, Square::D5);
    for _ in 0..65 {
        step(&mut app, 0.04);
    }
    let street_root = actor(&app, Square::D5).rig.root;
    let tray = app.world().resource::<CharacterScene>().tray_entities[0];
    app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::Original;
    step(&mut app, 0.0);
    assert_ne!(actor(&app, Square::D5).rig.root, street_root);
    assert!(app.world().get_entity(street_root).is_err());
    assert!(app.world().get_entity(tray).is_err());
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
    assert_eq!(
        app.world().resource::<BattleGame>().captured_pieces().len(),
        1
    );
    assert_eq!(
        app.world()
            .resource::<CharacterScene>()
            .tray_view
            .unwrap()
            .style,
        PieceStyle::Original
    );
}

#[test]
fn street_pawn_captures_in_both_directions_including_en_passant() {
    for (fen, from, to, victim, color) in [
        (
            "7k/8/8/3p4/4P3/8/8/K7 w - - 0 1",
            Square::E4,
            Square::D5,
            Square::D5,
            ChessColor::White,
        ),
        (
            "7k/8/8/4p3/3P4/8/8/K7 b - - 0 1",
            Square::E5,
            Square::D4,
            Square::D4,
            ChessColor::Black,
        ),
        (
            "7k/8/8/3pP3/8/8/8/K7 w - d6 0 1",
            Square::E5,
            Square::D6,
            Square::D5,
            ChessColor::White,
        ),
        (
            "7k/8/8/8/3Pp3/8/8/K7 b - d3 0 1",
            Square::E4,
            Square::D3,
            Square::D4,
            ChessColor::Black,
        ),
    ] {
        let mut app = fixture(fen);
        app.world_mut().resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        step(&mut app, 0.0);
        let root = actor(&app, from).rig.root;
        play(&mut app, from, to);
        for _ in 0..100 {
            step(&mut app, 0.025);
        }
        let world = app.world_mut();
        assert_eq!(
            world.resource::<BattleGame>().piece_at(to),
            Some(color.pawn())
        );
        assert!(world.resource::<BattleGame>().piece_at(from).is_none());
        if victim != to {
            assert!(world.resource::<BattleGame>().piece_at(victim).is_none());
        }
        assert_eq!(world.resource::<BattleGame>().history_len(), 1);
        assert_eq!(world.query::<&crate::Projectile>().iter(world).count(), 0);
        assert_eq!(world.query::<&crate::Debris>().iter(world).count(), 0);
        assert_eq!(actor(&app, to).rig.root, root);
        assert_eq!(actor(&app, to).style, PieceStyle::GtaStreet);
    }
}

#[test]
fn rook_extends_before_leaving_and_retracts_on_arrival() {
    let mut app = fixture("7k/8/2r5/8/8/2R5/8/K7 w - - 0 1");
    let from = Square::C3;
    let to = Square::C4;
    let root = actor(&app, from).rig.root;
    let rest = body(&app, from);
    let root_position = |app: &App| app.world().get::<Transform>(root).unwrap().translation;

    play(&mut app, from, to);
    step(&mut app, 0.12);
    assert_eq!(root_position(&app), square_world(from));
    assert!(body(&app, to).translation.y > rest.translation.y + 0.25);

    step(&mut app, 0.22);
    assert!(root_position(&app).distance(square_world(from)) > 0.1);
    assert!(root_position(&app).distance(square_world(to)) > 0.1);
    assert!(body(&app, to).translation.y > rest.translation.y + 0.50);

    let distance = square_world(from).distance(square_world(to));
    step(
        &mut app,
        animation::move_seconds(Role::Rook, distance) - 0.42,
    );
    assert_eq!(root_position(&app), square_world(to));
    assert!(body(&app, to).translation.y > rest.translation.y);
    assert!(body(&app, to).translation.y < rest.translation.y + 0.40);

    step(&mut app, 0.09);
    assert_eq!(root_position(&app), square_world(to));
    assert_eq!(body(&app, to), rest);
}

#[test]
fn white_and_black_knights_face_the_opponent_at_rest_and_after_landing() {
    let mut app = fixture("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let assert_facing = |app: &App, square, expected: Vec3| {
        let root = actor(app, square).rig.root;
        let rotation = app.world().get::<Transform>(root).unwrap().rotation;
        assert!((rotation * Vec3::NEG_Z).dot(expected) > 0.999);
    };
    for square in [Square::B1, Square::G1] {
        assert_facing(&app, square, Vec3::NEG_Z);
    }
    for square in [Square::B8, Square::G8] {
        assert_facing(&app, square, Vec3::Z);
    }
    for (from, to, expected) in [
        (Square::G1, Square::F3, Vec3::NEG_Z),
        (Square::G8, Square::F6, Vec3::Z),
    ] {
        play(&mut app, from, to);
        step(&mut app, 1.0);
        assert_facing(&app, to, expected);
    }
}

#[test]
fn knight_jumps_lands_pivots_and_jumps_again_in_each_direction() {
    for (from, to, corner) in [
        (Square::D4, Square::F5, Square::F4),
        (Square::F5, Square::D4, Square::D5),
        (Square::G1, Square::F3, Square::G3),
        (Square::F3, Square::G1, Square::F1),
    ] {
        let (origin, goal, bend) = (square_world(from), square_world(to), square_world(corner));
        let route = KnightRoute::new(origin, goal);
        let frames: Vec<_> = (0..=100)
            .map(|frame| route.sample(frame as f32 / 100.0))
            .collect();
        assert_eq!(frames[0].0, origin);
        assert_eq!(frames[100].0, goal);
        let distance_to_leg = |point: Vec3, start: Vec3, end: Vec3| {
            let axis = end - start;
            let projection = (point - start).dot(axis) / axis.length_squared();
            point.distance(start + axis * projection.clamp(0.0, 1.0))
        };
        assert!(frames.iter().all(|(position, _, _)| {
            distance_to_leg(*position, origin, bend).min(distance_to_leg(*position, bend, goal))
                < 0.0001
        }));
        assert!(
            frames[47..=65]
                .iter()
                .all(|(position, _, _)| *position == bend)
        );
        assert!(frames[22].2.offset.y > 1.0);
        assert!(frames[47].2.offset.y.abs() < 0.0001);
        assert!(frames[81].2.offset.y > 0.55);
        assert!(frames[100].2.offset.y.abs() < 0.0001);
        assert!((frames[47].1 * Vec3::NEG_Z).dot((bend - origin).normalize()) > 0.99);
        assert!((frames[65].1 * Vec3::NEG_Z).dot((goal - bend).normalize()) > 0.99);
    }
}

#[test]
fn every_role_yields_to_a_knight_and_returns_to_its_square() {
    for (role, symbol) in [
        (Role::Pawn, 'P'),
        (Role::Knight, 'N'),
        (Role::Bishop, 'B'),
        (Role::Rook, 'R'),
        (Role::Queen, 'Q'),
        (Role::King, 'K'),
    ] {
        let first_rank = if role == Role::King { "8" } else { "K7" };
        let fen = format!("7k/p7/8/8/3Np{symbol}2/8/8/{first_rank} w - - 0 1");
        let mut app = fixture(&fen);
        let pass = Square::E4;
        let corner = Square::F4;
        let passing_pawn = actor(&app, pass).rig.root;
        let obstacle = actor(&app, corner).rig.root;
        let original_pose = body(&app, corner);
        let position =
            |app: &App, entity| app.world().get::<Transform>(entity).unwrap().translation;
        play(&mut app, Square::D4, Square::F5);
        step(&mut app, 0.24);
        assert!(position(&app, passing_pawn).distance(square_world(pass)) > 0.48);
        step(&mut app, 0.30);
        assert_eq!(
            position(&app, actor(&app, Square::F5).rig.root),
            square_world(corner)
        );
        assert!(
            position(&app, obstacle).distance(square_world(corner)) > 0.55,
            "{role:?}"
        );
        assert_ne!(body(&app, corner), original_pose, "{role:?} dodge pose");
        let game = app.world().resource::<BattleGame>();
        assert_eq!(game.piece_at(pass), Some(ChessColor::Black.pawn()));
        assert_eq!(game.piece_at(corner).unwrap().role, role);
        assert_eq!(game.piece_at(Square::F5), Some(ChessColor::White.knight()));
        assert_eq!(game.history_len(), 1);
        step(&mut app, 0.55);
        assert_eq!(position(&app, obstacle), square_world(corner));
        assert_eq!(position(&app, passing_pawn), square_world(pass));
        assert_eq!(body(&app, corner), original_pose);
    }
}

#[test]
fn attacks_and_reactions_follow_the_same_projectile_lifecycle_for_all_roles() {
    for (role, fen, from, _, target) in CASES {
        let mut app = fixture(fen);
        let attacker_root = actor(&app, from).rig.root;
        let victim_root = actor(&app, target).rig.root;
        let attacker_rest = body(&app, from);
        let victim_rest = body(&app, target);
        play(&mut app, from, target);
        step(&mut app, 0.16);
        assert_ne!(body(&app, from), attacker_rest, "{role:?} attack");
        assert_eq!(body(&app, target), victim_rest, "{role:?} before impact");
        for _ in 0..40 {
            if app.world().resource::<SceneSync>().exploding {
                break;
            }
            assert!(app.world().resource::<SceneSync>().pending_move.is_some());
            step(&mut app, 0.05);
        }
        assert!(app.world().resource::<SceneSync>().exploding);
        step(&mut app, 0.15);
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 0);
        assert_ne!(body(&app, target), victim_rest, "{role:?} impact reaction");
        assert_eq!(actor(&app, target).rig.root, victim_root);
        for _ in 0..60 {
            if app.world().resource::<SceneSync>().pending_move.is_none() {
                break;
            }
            step(&mut app, 0.05);
        }
        assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
        assert_eq!(actor(&app, target).rig.root, attacker_root);
        assert!(app.world().get_entity(victim_root).is_err());
        assert_eq!(
            app.world().resource::<CharacterScene>().tray_entities.len(),
            1
        );
        assert!(
            !app.world()
                .resource::<SceneSync>()
                .can_undo(app.world().resource::<BattleGame>())
        );
        step(&mut app, 1.1);
        assert!(
            app.world()
                .resource::<SceneSync>()
                .can_undo(app.world().resource::<BattleGame>())
        );
    }
}

#[test]
fn checkmate_makes_every_losing_role_kneel_together_and_hold_for_both_colors() {
    for (fen, from, to, winner, loser_squares, winner_king) in [
        (
            "7k/p7/5KQ1/8/8/8/8/rnbq4 w - - 0 1",
            Square::G6,
            Square::G7,
            ChessColor::White,
            [
                Square::A7,
                Square::B1,
                Square::C1,
                Square::A1,
                Square::D1,
                Square::H8,
            ],
            Square::F6,
        ),
        (
            "RNBQ4/8/8/8/8/5kq1/P7/7K b - - 0 1",
            Square::G3,
            Square::G2,
            ChessColor::Black,
            [
                Square::A2,
                Square::B8,
                Square::C8,
                Square::A8,
                Square::D8,
                Square::H1,
            ],
            Square::F3,
        ),
    ] {
        let mut app = fixture(fen);
        let resting =
            loser_squares.map(|square| (actor(&app, square).rig.root, body(&app, square)));
        let winning_rest = [body(&app, from), body(&app, winner_king)];
        play(&mut app, from, to);
        step(&mut app, 0.20);
        for (square, (_, rest)) in loser_squares.into_iter().zip(resting) {
            assert_eq!(
                body(&app, square),
                rest,
                "wait for the winning move to land"
            );
        }
        step(&mut app, 0.30);
        for (square, (root, rest)) in loser_squares.into_iter().zip(resting) {
            assert_ne!(body(&app, square), rest, "{square}: simultaneous surrender");
            assert_eq!(actor(&app, square).rig.root, root);
        }
        step(&mut app, 1.0);
        let held = loser_squares.map(|square| body(&app, square));
        step(&mut app, 4.0);
        for (square, pose) in loser_squares.into_iter().zip(held) {
            assert_eq!(body(&app, square), pose);
        }
        assert_eq!(body(&app, to), winning_rest[0]);
        assert_eq!(body(&app, winner_king), winning_rest[1]);
        let game = app.world().resource::<BattleGame>();
        assert_eq!(game.outcome(), Some(GameOutcome::Checkmate { winner }));
        assert!(game.captured_pieces().is_empty());
        assert!(!app.world().resource::<SceneSync>().can_undo(game));
        // Starting a new game restores upright poses and clears the result clock.
        app.world_mut().resource_mut::<BattleGame>().reset();
        app.world_mut()
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        step(&mut app, 0.0);
        assert!(app.world().resource::<CharacterScene>().checkmate.is_none());
        assert!(
            app.world()
                .resource::<CharacterScene>()
                .board
                .iter()
                .flatten()
                .all(|visual| !visual.animated)
        );
    }
}

#[test]
fn undo_waits_for_every_roles_movement_and_rewinds_only_one_completed_move() {
    for (role, fen, from, to, _) in CASES {
        let mut app = fixture(fen);
        play(&mut app, from, to);
        step(&mut app, 0.20);
        let root = actor(&app, to).rig.root;
        let pose = body(&app, to);
        let world = app.world_mut();
        let mut game = world.remove_resource::<BattleGame>().unwrap();
        let mut scene = world.remove_resource::<SceneSync>().unwrap();
        assert!(!scene.can_undo(&game), "{role:?} still moving");
        crate::perform_undo(
            &mut game,
            &mut InteractionState::default(),
            &mut ChessClock::default(),
            &mut CpuSettings::default(),
            &mut scene,
            &mut world.resource_mut::<FxPause>(),
        );
        assert_eq!(game.history_len(), 1);
        assert!(!scene.reset_characters);
        world.insert_resource(game);
        world.insert_resource(scene);
        assert_eq!(actor(&app, to).rig.root, root);
        assert_eq!(body(&app, to), pose);
        step(&mut app, 1.0);
        let world = app.world_mut();
        let mut game = world.remove_resource::<BattleGame>().unwrap();
        let mut scene = world.remove_resource::<SceneSync>().unwrap();
        assert!(scene.can_undo(&game), "{role:?} finished moving");
        crate::perform_undo(
            &mut game,
            &mut InteractionState::default(),
            &mut ChessClock::default(),
            &mut CpuSettings::default(),
            &mut scene,
            &mut world.resource_mut::<FxPause>(),
        );
        assert_eq!(game.history_len(), 0);
        assert_eq!(game.piece_at(from).unwrap().role, role);
        assert!(!scene.can_undo(&game));
    }
}

#[test]
fn castling_moves_both_figures_and_promotion_finishes_as_a_pawn() {
    let mut app = fixture("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    let king = actor(&app, Square::E1).rig.root;
    let rook = actor(&app, Square::H1).rig.root;
    play(&mut app, Square::E1, Square::G1);
    assert_eq!(actor(&app, Square::G1).rig.root, king);
    assert_eq!(actor(&app, Square::F1).rig.root, rook);
    step(&mut app, 0.8);
    assert_eq!(
        app.world().get::<Transform>(rook).unwrap().translation,
        square_world(Square::F1)
    );

    let mut app = fixture("7k/P7/8/8/8/8/8/K7 w - - 0 1");
    play(&mut app, Square::A7, Square::A8);
    assert_eq!(actor(&app, Square::A8).piece.role, Role::Pawn);
    let promoted = app
        .world()
        .resource::<BattleGame>()
        .piece_at(Square::A8)
        .unwrap()
        .role;
    assert_ne!(promoted, Role::Pawn);
    step(&mut app, 0.8);
    assert_eq!(actor(&app, Square::A8).piece.role, promoted);
}
