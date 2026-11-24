//! The lowrider's own drone exits its trunk, banks above the board, and dives.
use super::{Effects, Flight, Projectile, material};
use crate::characters::{BOARD_SCALE, LowriderArm, LowriderDroneAssets, LowriderRotor};
use crate::{CaptureFxRequest, Debris, square_world};
use bevy::{light::NotShadowCaster, prelude::*};
use std::f32::consts::{PI, TAU};

pub(crate) const LAUNCH: f32 = 0.36;
pub(crate) const FLIGHT: f32 = 1.38;
const LIFT_END: f32 = 0.28;
const CRUISE_END: f32 = 0.88;
const DIVE_START: f32 = 1.08;

#[derive(Default)]
pub(super) struct LowriderFx {
    drone: LowriderDroneAssets,
    shard: Handle<Mesh>,
    puff: Handle<Mesh>,
    metal: Handle<StandardMaterial>,
    flash: Handle<StandardMaterial>,
    smoke: Handle<StandardMaterial>,
}
impl LowriderFx {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            drone: LowriderDroneAssets::new(meshes, materials),
            shard: meshes.add(Cuboid::new(0.023, 0.010, 0.045)),
            puff: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            metal: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(116, 132, 144),
                metallic: 0.65,
                ..default()
            }),
            flash: materials.add(material(0x9cffff, 0.90)),
            smoke: materials.add(material(0x687a84, 0.31)),
        }
    }
}

impl Effects {
    pub(super) fn spawn_lowrider_drone(
        &self,
        commands: &mut Commands,
        request: CaptureFxRequest,
        hit_height: f32,
    ) {
        let from = square_world(request.from);
        let target = square_world(request.target);
        let forward = (target - from).normalize();
        let sideways = forward.cross(Vec3::Y);
        let anchor = crate::characters::lowrider_launch(LAUNCH);
        let start = from + sideways * anchor.x + Vec3::Y * anchor.y - forward * anchor.z;
        let shot = Projectile {
            start,
            end: target + Vec3::Y * hit_height,
            elapsed: -LAUNCH,
            duration: FLIGHT,
            role: shakmaty::Role::Knight,
            index: 0,
            sideways,
            lane: 0.0,
            flight: Flight::LowriderDrone,
        };
        let root = commands
            .spawn((
                Transform::from_translation(start)
                    .with_rotation(rotation(&shot, 0.0))
                    .with_scale(Vec3::splat(BOARD_SCALE)),
                Visibility::Hidden,
                shot,
            ))
            .id();
        self.lowrider.drone.spawn(commands, root);
    }

    pub(crate) fn spawn_lowrider_impact(&self, commands: &mut Commands, shot: &Projectile) {
        for i in 0..12 {
            let angle = TAU * i as f32 / 12.0;
            let velocity = shot.sideways * angle.cos() * 0.90
                + Vec3::Y * (0.5 + angle.sin().abs() * 0.65)
                - (shot.end - shot.start).with_y(0.0).normalize() * 0.12;
            commands.spawn((
                Mesh3d(self.lowrider.shard.clone()),
                MeshMaterial3d(if i % 3 == 0 {
                    self.lowrider.flash.clone()
                } else {
                    self.lowrider.metal.clone()
                }),
                Transform::from_translation(shot.end).with_rotation(Quat::from_rotation_y(angle)),
                Debris {
                    velocity,
                    remaining: 0.30 + (i % 4) as f32 * 0.055,
                },
                NotShadowCaster,
            ));
        }
        commands.spawn((
            Mesh3d(self.lowrider.puff.clone()),
            MeshMaterial3d(self.lowrider.flash.clone()),
            Transform::from_translation(shot.end).with_scale(Vec3::splat(0.105)),
            Debris {
                velocity: Vec3::Y * 0.12,
                remaining: 0.075,
            },
            NotShadowCaster,
        ));
        for i in 0..4 {
            commands.spawn((
                Mesh3d(self.lowrider.puff.clone()),
                MeshMaterial3d(self.lowrider.smoke.clone()),
                Transform::from_translation(shot.end + shot.sideways * (i as f32 - 1.5) * 0.035)
                    .with_scale(Vec3::splat(0.07 + i as f32 * 0.015)),
                Debris {
                    velocity: Vec3::Y * (0.6 + i as f32 * 0.08),
                    remaining: 0.40 + i as f32 * 0.03,
                },
                NotShadowCaster,
            ));
        }
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub(crate) fn trunk_fraction(seconds: f32) -> f32 {
    ease(seconds / 0.30) * (1.0 - ease((seconds - 0.78) / 0.26))
}
pub(super) fn position(shot: &Projectile, t: f32) -> Vec3 {
    let seconds = t.clamp(0.0, 1.0) * FLIGHT;
    let roof = shot.start + Vec3::Y * 0.64;
    let high = shot.end + Vec3::Y * 0.76;
    if seconds < LIFT_END {
        shot.start.lerp(roof, ease(seconds / LIFT_END))
    } else if seconds < CRUISE_END {
        let u = ease((seconds - LIFT_END) / (CRUISE_END - LIFT_END));
        roof.lerp(high, u)
            + shot.sideways * (0.23 * (PI * u).sin())
            + Vec3::Y * (0.10 * (PI * u).sin())
    } else if seconds < DIVE_START {
        high + shot.sideways
            * (0.025 * (PI * (seconds - CRUISE_END) / (DIVE_START - CRUISE_END)).sin())
    } else {
        high.lerp(
            shot.end,
            ease((seconds - DIVE_START) / (FLIGHT - DIVE_START)),
        )
    }
}
pub(super) fn rotation(shot: &Projectile, t: f32) -> Quat {
    let seconds = t.clamp(0.0, 1.0) * FLIGHT;
    let forward = (shot.end - shot.start).with_y(0.0).normalize();
    let yaw = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
    if seconds < LIFT_END {
        yaw
    } else if seconds < CRUISE_END {
        let u = (seconds - LIFT_END) / (CRUISE_END - LIFT_END);
        yaw * Quat::from_euler(
            EulerRot::YXZ,
            0.0,
            -0.14 * (PI * u).sin(),
            0.20 * (PI * u).sin(),
        )
    } else if seconds < DIVE_START {
        yaw
    } else {
        yaw * Quat::from_rotation_x(-1.20 * ease((seconds - DIVE_START) / 0.15))
    }
}

pub(crate) fn animate_lowrider_drones(
    shots: Query<&Projectile>,
    mut arms: Query<(&LowriderArm, &mut Transform), Without<LowriderRotor>>,
    mut rotors: Query<(&LowriderRotor, &mut Transform), Without<LowriderArm>>,
) {
    for (arm, mut transform) in &mut arms {
        if let Ok(shot) = shots.get(arm.root) {
            transform.rotation =
                Quat::from_rotation_y(arm.fold * 0.82 * (1.0 - ease(shot.elapsed / 0.26)));
        }
    }
    for (rotor, mut transform) in &mut rotors {
        if let Ok(shot) = shots.get(rotor.root) {
            transform.rotation =
                Quat::from_rotation_y(rotor.phase + shot.elapsed.max(0.0) * TAU * 34.0);
        }
    }
}
