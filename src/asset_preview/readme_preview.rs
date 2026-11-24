//! Reproducible README screenshots rendered by the real game.
use crate::*;
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::path::PathBuf;

#[derive(Resource)]
struct ReadmePreview {
    frame: u32,
    output: PathBuf,
}

pub(super) fn configure(app: &mut App) -> bool {
    let args: Vec<_> = std::env::args().collect();
    let Some(index) = args.iter().position(|arg| arg == "--readme-screenshots") else {
        return false;
    };
    let output = PathBuf::from(args.get(index + 1).expect("README screenshot directory"));
    std::fs::create_dir_all(&output).unwrap();
    app.insert_resource(bevy::winit::WinitSettings::continuous())
        .insert_resource(ReadmePreview { frame: 0, output })
        .insert_resource(Appearance {
            piece_style: PieceStyle::GtaStreet,
            light_square: hex_rgb(0x5b1915),
            dark_square: hex_rgb(0x111024),
            background: hex_rgb(0x070215),
            ..Default::default()
        })
        .insert_resource(CameraRig {
            orbit: -1.05,
            distance: 11.2,
            elevation: 24.0_f32.to_radians(),
            ..Default::default()
        })
        .add_systems(Last, step);
    true
}

fn step(world: &mut World) {
    let (frame, output) = {
        let mut preview = world.resource_mut::<ReadmePreview>();
        preview.frame += 1;
        (preview.frame, preview.output.clone())
    };
    if frame == 1 {
        for mut window in world.query::<&mut Window>().iter_mut(world) {
            window.mode = bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Current,
            );
        }
    }
    for (shot_frame, distance, name) in [
        (90, 11.2, "gta-readme-close.png"),
        (150, 11.8, "gta-readme-medium.png"),
        (210, 12.4, "gta-readme-wide.png"),
    ] {
        if frame == shot_frame - 20 {
            world.resource_mut::<CameraRig>().distance = distance;
        }
        if frame == shot_frame {
            world
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(output.join(name)));
        }
    }
    if frame == 240 {
        world.write_message(AppExit::Success);
    }
}
