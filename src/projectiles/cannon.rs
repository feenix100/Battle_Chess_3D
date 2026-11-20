//! The knight launches a single iron shell. It opens near the top of its arc
//! and sends smaller rounds toward the target on separate paths.

use std::f32::consts::TAU;

use bevy::{light::NotShadowCaster, prelude::*};
use shakmaty::Role;

use super::{Effects, Flight, Projectile, lathe, ribbon, shot_layer};
use crate::Debris;

pub(super) const RADIUS: f32 = 0.170;
const SPLIT_FRACTION: f32 = 0.46;
const FLASH_SECONDS: f32 = 0.15;

#[derive(Default)]
pub(super) struct CannonMeshes {
    seam: Handle<Mesh>,
    spark: Handle<Mesh>,
}

impl CannonMeshes {
    pub(super) fn new(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            seam: meshes.add(ribbon(40, 0.0, TAU, |_| {
                (RADIUS - 0.012, RADIUS + 0.007, 0.0)
            })),
            spark: meshes.add(lathe(
                4,
                &[
                    (-0.056, 0.002),
                    (-0.009, 0.012),
                    (0.021, 0.007),
                    (0.058, 0.002),
                ],
            )),
        }
    }
}

pub(super) fn iron() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(49, 58, 75),
        metallic: 0.48,
        perceptual_roughness: 0.36,
        ..default()
    }
}

impl Effects {
    pub(super) fn decorate_cannon(
        &self,
        commands: &mut Commands,
        entity: Entity,
        trail_length: f32,
    ) {
        let visual = &self.shots[super::role_index(Role::Knight)];
        shot_layer(
            commands,
            entity,
            &self.cannon.seam,
            &visual.accent,
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.22, 0.48, 0.0)),
        );
        shot_layer(
            commands,
            entity,
            &self.trail,
            &visual.glow,
            Transform::from_scale(Vec3::new(0.82, 0.82, trail_length)),
        );
    }

    pub(crate) fn split_knight(&self, commands: &mut Commands, shell: &Projectile) -> bool {
        let split_time = shell.duration * SPLIT_FRACTION;
        if !matches!(shell.flight, Flight::KnightShell) || shell.elapsed < split_time {
            return false;
        }
        let origin = shell.position(SPLIT_FRACTION);
        let age = shell.elapsed - split_time;
        let forward = Vec3::Y.cross(shell.sideways);
        let visual = &self.shots[super::role_index(Role::Knight)];
        let remaining = shell.duration - split_time;

        // Sideways, vertical, and forward kicks separate the balls in three dimensions.
        let spreads = [
            (Vec3::new(-0.46, 0.11, 0.00), -0.10, 0.030),
            (Vec3::new(0.48, 0.16, 0.05), 0.11, 0.047),
            (Vec3::new(-0.05, 0.37, -0.17), -0.025, 0.086),
            (Vec3::new(0.10, -0.15, 0.21), 0.027, 0.0),
        ];
        for (index, (kick, landing, delay)) in spreads.into_iter().enumerate() {
            let control = shell.sideways * kick.x + Vec3::Y * kick.y + forward * kick.z;
            let small_ball = Projectile {
                start: origin,
                end: shell.end + shell.sideways * landing,
                elapsed: age,
                duration: remaining + delay,
                role: Role::Knight,
                index,
                sideways: shell.sideways,
                lane: kick.x,
                flight: Flight::KnightFragment { control },
            };
            let entity = self.spawn_shot(commands, small_ball, Vec3::splat(0.61));
            self.decorate_cannon(commands, entity, 0.52);
        }

        if age < FLASH_SECONDS {
            for i in 0..7 {
                let angle = TAU * i as f32 / 7.0;
                let direction =
                    (shell.sideways * angle.cos() + Vec3::Y * angle.sin() + forward * 0.22)
                        .normalize();
                let velocity = direction * (1.65 + (i % 3) as f32 * 0.30);
                commands.spawn((
                    Mesh3d(self.cannon.spark.clone()),
                    MeshMaterial3d(visual.accent.clone()),
                    Transform::from_translation(origin + velocity * age)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, direction)),
                    NotShadowCaster,
                    Debris {
                        velocity,
                        remaining: FLASH_SECONDS - age,
                    },
                ));
            }
        }
        true
    }
}
