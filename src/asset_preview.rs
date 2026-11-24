//! Opt-in developer harness: render the real game with deterministic capture frames.
//! cargo run --features asset-preview -- --capture-pawn-preview ABSOLUTE_OUTPUT_DIR
use super::*;
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, save_to_disk},
    time::TimeUpdateStrategy,
};
use shakmaty::{CastlingMode, Chess, fen::Fen};
use std::{path::PathBuf, time::Duration};
mod king_preview;
mod mansion_preview;
mod mansion_walk_preview;
mod police_preview;
mod queen_preview;
mod readme_preview;

#[derive(Resource)]
struct Preview {
    frame: u32,
    output: PathBuf,
    lowrider: bool,
}

pub(super) fn configure(app: &mut App) {
    if readme_preview::configure(app) {
        return;
    }
    if mansion_walk_preview::configure(app) {
        return;
    }
    if queen_preview::configure(app) {
        return;
    }
    if king_preview::configure(app) {
        return;
    }
    if police_preview::configure(app) {
        return;
    }
    if mansion_preview::configure(app) {
        return;
    }
    let args: Vec<_> = std::env::args().collect();
    let Some(i) = args
        .iter()
        .position(|s| s == "--capture-pawn-preview" || s == "--capture-lowrider-preview")
    else {
        return;
    };
    let output = PathBuf::from(args.get(i + 1).expect("preview output directory"));
    std::fs::create_dir_all(&output).unwrap();
    app.insert_resource(Preview {
        frame: 0,
        output,
        lowrider: args[i] == "--capture-lowrider-preview",
    })
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 30.0,
    )))
    .add_systems(Last, step_preview);
}

fn shot(world: &mut World, path: PathBuf) {
    world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

fn step_preview(world: &mut World) {
    let (frame, output, lowrider) = {
        let mut preview = world.resource_mut::<Preview>();
        preview.frame += 1;
        (preview.frame, preview.output.clone(), preview.lowrider)
    };
    if frame == 1 {
        world.resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        world.resource_mut::<CpuSettings>().enabled = false;
    }
    if frame == 90 {
        shot(
            world,
            output.join(if lowrider {
                "gta-lowrider-in-game.png"
            } else {
                "gta-pawn-in-game.png"
            }),
        );
    }
    if frame == 110 {
        let position = (if lowrider {
            "7k/8/8/5p2/3N4/8/8/K7 w - - 0 1"
        } else {
            "7k/8/8/3p4/4P3/8/8/K7 w - - 0 1"
        })
        .parse::<Fen>()
        .unwrap()
        .into_position::<Chess>(CastlingMode::Standard)
        .unwrap();
        *world.resource_mut::<BattleGame>() = BattleGame::from_position(position);
        world
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        let mut camera = world.resource_mut::<CameraRig>();
        camera.distance = if lowrider { 5.0 } else { 4.5 };
        camera.orbit = if lowrider { -0.85 } else { -0.67 };
        camera.elevation = 24_f32.to_radians();
        camera.pan_offset = Vec3::new(if lowrider { 0.50 } else { 0.0 }, 0.02, 0.0);
    }
    if frame == 160 {
        let mut game = world.remove_resource::<BattleGame>().unwrap();
        let mut sync = world.remove_resource::<SceneSync>().unwrap();
        let mut pause = world.remove_resource::<FxPause>().unwrap();
        let movement = if lowrider {
            game.legal_moves_between(Square::D4, Square::F5)[0]
        } else {
            game.legal_moves_between(Square::E4, Square::D5)[0]
        };
        commit_move(
            &mut game,
            movement,
            &mut InteractionState::default(),
            &mut CpuSettings::default(),
            &ChessClock::default(),
            &BattleSettings::default(),
            &mut sync,
            &mut pause,
        );
        world.insert_resource(game);
        world.insert_resource(sync);
        world.insert_resource(pause);
    }
    if (145..=if lowrider { 300 } else { 235 }).contains(&frame) {
        shot(
            world,
            output.join(format!("capture-{:03}.png", frame - 145)),
        );
    }
    if frame == if lowrider { 325 } else { 260 } {
        shot(
            world,
            output.join(if lowrider {
                "gta-lowrider-after-capture.png"
            } else {
                "gta-pawn-after-capture.png"
            }),
        );
    }
    if frame == if lowrider { 360 } else { 300 } {
        world.write_message(AppExit::Success);
    }
}
