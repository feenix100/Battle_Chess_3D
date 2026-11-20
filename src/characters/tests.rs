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
