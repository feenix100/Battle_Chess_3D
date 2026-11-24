//! Opt-in visual check for the mansion's deployed legs and walking gait.
use crate::*;
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, save_to_disk},
    time::TimeUpdateStrategy,
};
use shakmaty::{CastlingMode, Chess, fen::Fen};
use std::{path::PathBuf, time::Duration};
#[derive(Resource)]
struct MansionWalkPreview {
    frame: u32,
    output: PathBuf,
}
pub(super) fn configure(app: &mut App) -> bool {
    let args: Vec<_> = std::env::args().collect();
    let Some(i) = args
        .iter()
        .position(|a| a == "--capture-mansion-walk-preview")
    else {
        return false;
    };
    let output = PathBuf::from(
        args.get(i + 1)
            .expect("mansion walk preview output directory"),
    );
    std::fs::create_dir_all(&output).unwrap();
    app.insert_resource(MansionWalkPreview { frame: 0, output })
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 30.0,
        )))
        .add_systems(Last, step);
    true
}
fn shot(world: &mut World, path: PathBuf) {
    world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
fn step(world: &mut World) {
    let (frame, output) = {
        let mut s = world.resource_mut::<MansionWalkPreview>();
        s.frame += 1;
        (s.frame, s.output.clone())
    };
    if frame == 1 {
        world.resource_mut::<Appearance>().piece_style = PieceStyle::GtaStreet;
        world.resource_mut::<CpuSettings>().enabled = false;
    }
    if frame == 110 {
        let position = "7k/7p/8/2r5/8/8/P7/K7 b - - 0 1"
            .parse::<Fen>()
            .unwrap()
            .into_position::<Chess>(CastlingMode::Standard)
            .unwrap();
        *world.resource_mut::<BattleGame>() = BattleGame::from_position(position);
        world
            .resource_mut::<SceneSync>()
            .cancel_capture(&mut FxPause::default());
        let mut camera = world.resource_mut::<CameraRig>();
        camera.distance = 4.8;
        camera.orbit = -0.60;
        camera.elevation = 18_f32.to_radians();
        camera.pan_offset = Vec3::new(0.0, 0.08, -0.5);
    }
    if frame == 160 {
        let mut game = world.remove_resource::<BattleGame>().unwrap();
        let mut sync = world.remove_resource::<SceneSync>().unwrap();
        let mut pause = world.remove_resource::<FxPause>().unwrap();
        let movement = game.legal_moves_between(Square::C5, Square::F5)[0];
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
        shot(world, output.join(format!("walk-{:03}.png", frame - 145)));
    }
    if frame == 175 {
        shot(world, output.join("gta-rook-walking-in-game.png"));
    }
    if frame == 220 {
        shot(world, output.join("gta-rook-retracted-in-game.png"));
    }
    if frame == 330 {
        world.write_message(AppExit::Success);
    }
}
