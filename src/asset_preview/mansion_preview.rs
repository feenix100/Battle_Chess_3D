//! Deterministic capture recording using the game's actual mansion systems.
use crate::*;
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, save_to_disk},
    time::TimeUpdateStrategy,
};
use shakmaty::{CastlingMode, Chess, fen::Fen};
use std::{path::PathBuf, time::Duration};
#[derive(Resource)]
struct MansionPreview {
    frame: u32,
    output: PathBuf,
}
pub(super) fn configure(app: &mut App) -> bool {
    let args: Vec<_> = std::env::args().collect();
    let Some(index) = args
        .iter()
        .position(|arg| arg == "--capture-mansion-preview")
    else {
        return false;
    };
    let output = PathBuf::from(
        args.get(index + 1)
            .expect("mansion preview output directory"),
    );
    std::fs::create_dir_all(&output).unwrap();
    app.insert_resource(MansionPreview { frame: 0, output })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 30.0,
        )))
        .add_systems(Last, step);
    true
}
fn screenshot(world: &mut World, path: PathBuf) {
    world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
fn step(world: &mut World) {
    let (frame, output) = {
        let mut preview = world.resource_mut::<MansionPreview>();
        preview.frame += 1;
        (preview.frame, preview.output.clone())
    };
    if frame == 1 {
        world.resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        world.resource_mut::<CpuSettings>().enabled = false;
    }
    if frame == 90 {
        screenshot(world, output.join("gta-mansion-in-game.png"));
    }
    if frame == 110 {
        let position = "7k/7p/2r5/8/8/2P5/P7/K7 b - - 0 1"
            .parse::<Fen>()
            .unwrap()
            .into_position::<Chess>(CastlingMode::Standard)
            .unwrap();
        *world.resource_mut::<BattleGame>() = BattleGame::from_position(position);
        world
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        let mut camera = world.resource_mut::<CameraRig>();
        camera.distance = 6.3;
        camera.orbit = -0.72;
        camera.elevation = 32_f32.to_radians();
        camera.pan_offset = Vec3::new(-1.5, 0.06, 0.0);
    }
    if frame == 160 {
        let mut game = world.remove_resource::<BattleGame>().unwrap();
        let mut sync = world.remove_resource::<SceneSync>().unwrap();
        let mut pause = world.remove_resource::<FxPause>().unwrap();
        let movement = game.legal_moves_between(Square::C6, Square::C3)[0];
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
    if (145..=285).contains(&frame) {
        screenshot(
            world,
            output.join(format!("capture-{:03}.png", frame - 145)),
        );
    }
    if frame == 315 {
        screenshot(world, output.join("gta-mansion-after-capture.png"));
    }
    if frame == 360 {
        world.write_message(AppExit::Success);
    }
}
