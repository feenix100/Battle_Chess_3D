use shakmaty::{CastlingMode, Chess, fen::Fen};

use super::*;

fn from_fen(fen: &str) -> BattleGame {
    BattleGame::from_position(
        fen.parse::<Fen>()
            .unwrap()
            .into_position::<Chess>(CastlingMode::Standard)
            .unwrap(),
    )
}

fn draw(
    ctx: &egui::Context,
    game: &BattleGame,
    size: egui::Vec2,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, Option<(egui::Rect, egui::Rect, bool)>) {
    let mut menu = None;
    let time = ctx.input(|input| input.time) + 0.05;
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            time: Some(time),
            events,
            ..default()
        },
        |_| {
            menu = checkmate_menu(ctx, game, sidebar_dimensions(size).0)
                .map(|menu| (menu.response.rect, menu.inner.rect, menu.inner.clicked()));
        },
    );
    (output, menu)
}

fn text(shapes: &[egui::epaint::Shape], result: &mut Vec<String>) {
    for shape in shapes {
        match shape {
            egui::epaint::Shape::Text(label) => result.push(label.galley.text().to_owned()),
            egui::epaint::Shape::Vec(children) => text(children, result),
            _ => {}
        }
    }
}

#[test]
fn menu_announces_either_winner_and_leaves_most_of_the_board_visible() {
    for (fen, winner) in [
        ("7k/6Q1/5K2/8/8/8/pnbrq3/8 b - - 0 1", ChessColor::White),
        ("RNBQ4/8/8/8/8/5k2/P5q1/7K w - - 0 1", ChessColor::Black),
    ] {
        let game = from_fen(fen);
        assert_eq!(
            game.outcome(),
            Some(game::GameOutcome::Checkmate { winner })
        );
        for size in [
            egui::vec2(900.0, 620.0),
            egui::vec2(1320.0, 860.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let ctx = egui::Context::default();
            for pass in 0..4 {
                let (output, menu) = draw(&ctx, &game, size, Vec::new());
                if pass == 3 {
                    let (card, button, clicked) = menu.unwrap();
                    let board = egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(size.x - sidebar_dimensions(size).0, size.y),
                    );
                    assert!(
                        board.contains_rect(card),
                        "result card overlaps sidebar: {size:?}"
                    );
                    assert!(card.contains_rect(button));
                    assert!(
                        card.area() < board.area() * 0.20,
                        "board must remain visible"
                    );
                    assert!(!clicked);
                    let mut words = Vec::new();
                    for shape in &output.shapes {
                        text(std::slice::from_ref(&shape.shape), &mut words);
                    }
                    assert!(words.iter().any(|word| word == "Checkmate"), "{words:?}");
                    assert!(
                        words
                            .iter()
                            .any(|word| word == &format!("{} wins", color_name(winner))),
                        "{words:?}"
                    );
                    assert!(words.iter().any(|word| word == "New game"), "{words:?}");
                }
                output.drop_without_applying_deltas();
            }
        }
    }
}

#[test]
fn new_game_menu_click_resets_the_result_clock_and_celebration() {
    let mut game = BattleGame::new();
    let mut latest = None;
    for (from, to) in [
        (Square::F2, Square::F3),
        (Square::E7, Square::E5),
        (Square::G2, Square::G4),
        (Square::D8, Square::H4),
    ] {
        let movement = game.legal_moves_between(from, to)[0];
        game.play(movement).unwrap();
        latest = Some(movement);
    }
    let ctx = egui::Context::default();
    let size = egui::vec2(1320.0, 860.0);
    let mut button = egui::Rect::NOTHING;
    for _ in 0..4 {
        let (output, menu) = draw(&ctx, &game, size, Vec::new());
        button = menu.unwrap().1;
        output.drop_without_applying_deltas();
    }
    let click = button.center();
    let (output, _) = draw(
        &ctx,
        &game,
        size,
        vec![
            egui::Event::PointerMoved(click),
            egui::Event::PointerButton {
                pos: click,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: default(),
            },
        ],
    );
    output.drop_without_applying_deltas();
    let (output, menu) = draw(
        &ctx,
        &game,
        size,
        vec![egui::Event::PointerButton {
            pos: click,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: default(),
        }],
    );
    assert!(
        menu.unwrap().2,
        "the menu's New game button must receive the click"
    );
    output.drop_without_applying_deltas();
    let mut interaction = InteractionState {
        selected: Some(Square::H4),
        ..default()
    };
    let mut clock = ChessClock {
        enabled: true,
        initial_seconds: 180.0,
        white_seconds: 83.0,
        black_seconds: 97.0,
        timed_out: None,
    };
    let mut cpu = CpuSettings {
        enabled: true,
        delay_remaining: 0.0,
        ..default()
    };
    let mut sync = SceneSync {
        undo_used: true,
        character_moves: vec![latest.unwrap()],
        characters_animating: true,
        ..default()
    };
    let mut pause = FxPause::default();
    start_new_game(
        &mut game,
        &mut interaction,
        &mut clock,
        &mut cpu,
        &mut sync,
        &mut pause,
    );
    assert_eq!(game.history_len(), 0);
    assert_eq!(game.side_to_move(), ChessColor::White);
    assert!(game.outcome().is_none());
    assert_eq!(
        Square::ALL
            .into_iter()
            .filter(|&square| game.piece_at(square).is_some())
            .count(),
        32
    );
    assert!(interaction.selected.is_none() && interaction.pending_promotion.is_none());
    assert_eq!((clock.white_seconds, clock.black_seconds), (180.0, 180.0));
    assert!(clock.enabled && clock.timed_out.is_none());
    assert!(cpu.enabled && cpu.color == ChessColor::Black);
    assert_eq!(cpu.delay_remaining, CPU_DELAY_SECONDS);
    assert!(sync.clear_fx && sync.reset_characters && !sync.undo_used);
    assert!(sync.character_moves.is_empty() && !sync.characters_animating);
    assert_eq!(pause.remaining, 0.0);
    let (output, menu) = draw(&ctx, &game, size, Vec::new());
    assert!(menu.is_none());
    output.drop_without_applying_deltas();
}

#[test]
fn ongoing_games_and_draws_have_no_checkmate_menu() {
    for game in [
        BattleGame::new(),
        from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1"),
    ] {
        let (output, menu) = draw(
            &egui::Context::default(),
            &game,
            egui::vec2(1320.0, 860.0),
            Vec::new(),
        );
        assert!(menu.is_none());
        output.drop_without_applying_deltas();
    }
}

fn draw_check(
    ctx: &egui::Context,
    game: &BattleGame,
    size: egui::Vec2,
) -> (egui::FullOutput, Option<egui::Rect>) {
    let mut rect = None;
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events: vec![egui::Event::PointerMoved(egui::pos2(100.0, 100.0))],
            ..default()
        },
        |_| {
            rect = check_banner(ctx, game, sidebar_dimensions(size).0);
        },
    );
    (output, rect)
}

#[test]
fn check_warning_is_centered_over_the_board_for_either_color() {
    for fen in [
        "7k/6Q1/8/8/8/8/P7/K7 b - - 0 1",
        "7k/7p/8/8/8/8/1q6/K7 w - - 0 1",
    ] {
        let game = from_fen(fen);
        assert!(game.is_check() && game.outcome().is_none());
        for size in [
            egui::vec2(900.0, 620.0),
            egui::vec2(1320.0, 860.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let ctx = egui::Context::default();
            let (output, rect) = draw_check(&ctx, &game, size);
            let board = egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size.x - sidebar_dimensions(size).0, size.y),
            );
            let rect = rect.unwrap();
            assert!(board.contains_rect(rect));
            assert!((rect.center() - board.center()).length() < 0.01);
            let mut words = Vec::new();
            for shape in &output.shapes {
                text(std::slice::from_ref(&shape.shape), &mut words);
            }
            assert!(words.iter().any(|word| word == "CHECK!"));
            assert!(
                !ctx.is_pointer_over_egui(),
                "the painted warning must not capture board input"
            );
            output.drop_without_applying_deltas();
        }
    }
}

#[test]
fn check_warning_clears_after_check_and_is_absent_at_checkmate() {
    let ctx = egui::Context::default();
    let size = egui::vec2(1320.0, 860.0);
    let mut game = from_fen("7k/6Q1/8/8/8/8/P7/K7 b - - 0 1");
    let (output, rect) = draw_check(&ctx, &game, size);
    assert!(rect.is_some());
    output.drop_without_applying_deltas();
    game.play(game.legal_moves_between(Square::H8, Square::G7)[0])
        .unwrap();
    for game in [
        game,
        BattleGame::new(),
        from_fen("7k/6Q1/5K2/8/8/8/pnbrq3/8 b - - 0 1"),
    ] {
        let (output, rect) = draw_check(&ctx, &game, size);
        assert!(rect.is_none());
        output.drop_without_applying_deltas();
    }
}

fn draw_confirmation(
    ctx: &egui::Context,
    size: egui::Vec2,
    events: Vec<egui::Event>,
) -> (egui::FullOutput, Option<NewGameAction>, [egui::Rect; 2]) {
    let mut result = None;
    let time = ctx.input(|input| input.time) + 0.05;
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            time: Some(time),
            events,
            ..default()
        },
        |_| {
            let (action, modal) = new_game_confirmation(ctx, sidebar_dimensions(size).0);
            result = Some((action, [modal.inner[0].rect, modal.inner[1].rect]));
        },
    );
    let (action, rects) = result.unwrap();
    (output, action, rects)
}

#[test]
fn confirmation_buttons_reset_or_preserve_the_actual_game_state() {
    for (index, expected) in [(0, NewGameAction::Confirm), (1, NewGameAction::Back)] {
        let ctx = egui::Context::default();
        let size = egui::vec2(1320.0, 860.0);
        let mut buttons = [egui::Rect::NOTHING; 2];
        for _ in 0..4 {
            let (out, action, rects) = draw_confirmation(&ctx, size, vec![]);
            assert_eq!(action, None);
            buttons = rects;
            out.drop_without_applying_deltas();
        }
        let click = buttons[index].center();
        let (out, _, _) = draw_confirmation(
            &ctx,
            size,
            vec![
                egui::Event::PointerMoved(click),
                egui::Event::PointerButton {
                    pos: click,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: default(),
                },
            ],
        );
        out.drop_without_applying_deltas();
        let (out, action, _) = draw_confirmation(
            &ctx,
            size,
            vec![egui::Event::PointerButton {
                pos: click,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: default(),
            }],
        );
        out.drop_without_applying_deltas();
        assert_eq!(action, Some(expected));
        let mut game = BattleGame::new();
        for (from, to) in [(Square::E2, Square::E4), (Square::D7, Square::D5)] {
            game.play(game.legal_moves_between(from, to)[0]).unwrap();
        }
        let previous = game.position().clone();
        let pending = game.legal_moves_between(Square::E4, Square::D5)[0];
        let mut interaction = InteractionState {
            confirm_new_game: true,
            selected: Some(Square::E4),
            ..default()
        };
        let mut clock = ChessClock {
            enabled: true,
            initial_seconds: 300.0,
            white_seconds: 97.0,
            black_seconds: 113.0,
            timed_out: None,
        };
        let mut cpu = CpuSettings {
            enabled: true,
            ..default()
        };
        let mut sync = SceneSync {
            pending_move: Some(pending),
            exploding: true,
            ..default()
        };
        let mut pause = FxPause { remaining: 1.0 };
        resolve_new_game_confirmation(
            action.unwrap(),
            &mut game,
            &mut interaction,
            &mut clock,
            &mut cpu,
            &mut sync,
            &mut pause,
        );
        assert!(!interaction.confirm_new_game);
        match expected {
            NewGameAction::Confirm => {
                assert_eq!(game.history_len(), 0);
                assert_eq!(*game.position(), shakmaty::Chess::default());
                assert_eq!((clock.white_seconds, clock.black_seconds), (300.0, 300.0));
                assert!(sync.pending_move.is_none() && sync.reset_characters && sync.clear_fx);
                assert!(interaction.selected.is_none());
                assert_eq!(pause.remaining, 0.0);
            }
            NewGameAction::Back => {
                assert_eq!(*game.position(), previous);
                assert_eq!(game.history_len(), 2);
                assert_eq!((clock.white_seconds, clock.black_seconds), (97.0, 113.0));
                assert_eq!(sync.pending_move, Some(pending));
                assert!(sync.exploding);
                assert_eq!(interaction.selected, Some(Square::E4));
                assert_eq!(pause.remaining, 1.0);
            }
        }
    }
}

#[test]
fn confirmation_fits_the_board_and_escape_returns_to_the_game() {
    for size in [
        egui::vec2(900.0, 620.0),
        egui::vec2(1320.0, 860.0),
        egui::vec2(1920.0, 1080.0),
    ] {
        let ctx = egui::Context::default();
        for _ in 0..4 {
            let (out, action, buttons) = draw_confirmation(&ctx, size, vec![]);
            let board = egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size.x - sidebar_dimensions(size).0, size.y),
            );
            assert!(buttons.iter().all(|button| board.contains_rect(*button)));
            assert!(buttons[0].right() < buttons[1].left());
            assert_eq!(action, None);
            out.drop_without_applying_deltas();
        }
        let (out, action, _) = draw_confirmation(
            &ctx,
            size,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: default(),
            }],
        );
        assert_eq!(action, Some(NewGameAction::Back));
        out.drop_without_applying_deltas();
    }
}

#[test]
fn confirmation_pauses_cpu_and_clock_and_blocks_board_moves() {
    let mut game = BattleGame::new();
    game.play(game.legal_moves_between(Square::E2, Square::E4)[0])
        .unwrap();
    let mut interaction = InteractionState {
        selected: Some(Square::E7),
        confirm_new_game: true,
        ..default()
    };
    let mut cpu = CpuSettings {
        enabled: true,
        delay_remaining: 0.0,
        ..default()
    };
    let clock = ChessClock {
        enabled: true,
        white_seconds: 102.0,
        black_seconds: 110.0,
        ..default()
    };
    let battle = BattleSettings::default();
    let mut sync = SceneSync::default();
    let mut pause = FxPause::default();
    handle_square_action(
        Square::E5,
        &mut game,
        &mut interaction,
        &mut cpu,
        &clock,
        &battle,
        &mut sync,
        &mut pause,
    );
    assert_eq!(game.history_len(), 1);
    assert_eq!(interaction.selected, Some(Square::E7));
    let mut app = App::new();
    app.init_resource::<Time>()
        .insert_resource(game)
        .insert_resource(interaction)
        .insert_resource(cpu)
        .insert_resource(clock)
        .insert_resource(battle)
        .insert_resource(sync)
        .insert_resource(pause)
        .add_systems(Update, (cpu_turn_system, clock_system).chain());
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.6));
    app.update();
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 1);
    let clock = app.world().resource::<ChessClock>();
    assert_eq!((clock.white_seconds, clock.black_seconds), (102.0, 110.0));
    app.world_mut()
        .resource_mut::<InteractionState>()
        .confirm_new_game = false;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.6));
    app.update();
    assert_eq!(app.world().resource::<BattleGame>().history_len(), 2);
    assert!(app.world().resource::<ChessClock>().white_seconds < 102.0);
}
