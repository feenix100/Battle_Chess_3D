//! Small brass rounds, synchronized muzzle flashes, casings and compact impacts.
use super::{Effects, Projectile, ShotVisual, lathe, material};
use crate::Debris;
use bevy::{light::NotShadowCaster, prelude::*};

#[derive(Default)]
pub(super) struct StreetMeshes {
    pub visual: ShotVisual,
    flash: Handle<Mesh>,
    puff: Handle<Mesh>,
    smoke: Handle<StandardMaterial>,
}

impl StreetMeshes {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            visual: ShotVisual {
                shape: meshes.add(lathe(
                    12,
                    &[
                        (-0.035, 0.008),
                        (-0.030, 0.010),
                        (0.010, 0.010),
                        (0.025, 0.007),
                        (0.036, 0.001),
                    ],
                )),
                body: materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(225, 172, 75),
                    metallic: 0.70,
                    perceptual_roughness: 0.28,
                    ..default()
                }),
                accent: materials.add(material(0xffefbf, 1.0)),
                glow: materials.add(material(0xffbe57, 0.75)),
                impact: materials.add(material(0xffd18c, 0.92)),
            },
            flash: meshes.add(lathe(4, &[(-0.018, 0.001), (0.012, 0.035), (0.095, 0.001)])),
            puff: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            smoke: materials.add(material(0x8a8175, 0.23)),
        }
    }
}

impl Effects {
    pub(crate) fn spawn_street_flash(&self, commands: &mut Commands, shot: &Projectile) {
        let direction = (shot.end - shot.start).normalize();
        commands.spawn((
            Mesh3d(self.street.flash.clone()),
            MeshMaterial3d(self.street.visual.accent.clone()),
            Transform::from_translation(shot.start)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, direction)),
            NotShadowCaster,
            Debris {
                velocity: Vec3::ZERO,
                remaining: 0.055,
            },
        ));
        commands.spawn((
            Mesh3d(self.street.visual.shape.clone()),
            MeshMaterial3d(self.street.visual.body.clone()),
            Transform::from_translation(shot.start - direction * 0.14)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, shot.sideways))
                .with_scale(Vec3::new(0.7, 0.7, 0.6)),
            NotShadowCaster,
            Debris {
                velocity: shot.sideways * 0.55 + Vec3::Y * 0.75,
                remaining: 0.31,
            },
        ));
    }

    pub(crate) fn spawn_street_impact(&self, commands: &mut Commands, shot: &Projectile) {
        let incoming = (shot.end - shot.start).normalize();
        for i in 0..7 {
            let angle = std::f32::consts::TAU * i as f32 / 7.0;
            let velocity = shot.sideways * angle.cos() * 0.52
                + Vec3::Y * (0.40 + angle.sin().abs() * 0.42)
                - incoming * 0.18;
            commands.spawn((
                Mesh3d(self.street.flash.clone()),
                MeshMaterial3d(self.street.visual.impact.clone()),
                Transform::from_translation(shot.end)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, velocity.normalize()))
                    .with_scale(Vec3::new(0.22, 0.22, 0.42)),
                NotShadowCaster,
                Debris {
                    velocity,
                    remaining: 0.20 + (i % 3) as f32 * 0.055,
                },
            ));
        }
        for i in 0..2 {
            commands.spawn((
                Mesh3d(self.street.puff.clone()),
                MeshMaterial3d(self.street.smoke.clone()),
                Transform::from_translation(shot.end + Vec3::Y * i as f32 * 0.035)
                    .with_scale(Vec3::splat(0.055 + i as f32 * 0.020)),
                NotShadowCaster,
                Debris {
                    velocity: Vec3::Y * (0.45 + i as f32 * 0.12),
                    remaining: 0.36,
                },
            ));
        }
    }
}
