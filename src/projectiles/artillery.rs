//! The queen's staggered artillery barrage and its expanding impact smoke.

use bevy::{light::NotShadowCaster, prelude::*};
use shakmaty::Role;

use super::{Effects, lathe, role_index, shot_layer};
use crate::{Debris, Projectile, SceneHandles};

const SMOKE_STEPS: usize = 16;

#[derive(Default)]
pub(super) struct ArtilleryMeshes {
    band: Handle<Mesh>,
    puff: Handle<Mesh>,
    smoke: [Handle<StandardMaterial>; SMOKE_STEPS],
}

pub(super) fn shell() -> Mesh {
    lathe(
        24,
        &[
            (-0.235, 0.069),
            (-0.212, 0.089),
            (-0.151, 0.089),
            (-0.128, 0.082),
            (0.070, 0.082),
            (0.118, 0.075),
            (0.192, 0.051),
            (0.246, 0.020),
            (0.264, 0.004),
        ],
    )
}

pub(super) fn steel() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(76, 87, 65),
        metallic: 0.42,
        perceptual_roughness: 0.38,
        ..default()
    }
}

impl ArtilleryMeshes {
    pub(super) fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            band: meshes.add(lathe(24, &[(-0.173, 0.092), (-0.143, 0.092)])),
            puff: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            smoke: std::array::from_fn(|index| {
                let t = index as f32 / (SMOKE_STEPS - 1) as f32;
                let heat = (1.0 - t * 3.0).max(0.0);
                let tint = Vec3::new(0.24, 0.22, 0.20).lerp(Vec3::new(4.2, 0.90, 0.10), heat);
                materials.add(StandardMaterial {
                    base_color: Color::linear_rgba(tint.x, tint.y, tint.z, 0.83 * (1.0 - t).sqrt()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })
            }),
        }
    }
}

impl Effects {
    pub(super) fn decorate_artillery(&self, commands: &mut Commands, entity: Entity) {
        let visual = &self.shots[role_index(Role::Queen)];
        shot_layer(
            commands,
            entity,
            &self.artillery.band,
            &visual.accent,
            Transform::IDENTITY,
        );
        shot_layer(
            commands,
            entity,
            &self.trail,
            &visual.glow,
            Transform::from_scale(Vec3::new(0.80, 0.80, 0.77)),
        );
    }

    pub(crate) fn spawn_artillery_impact(&self, commands: &mut Commands, impact: Vec3) {
        let visual = &self.shots[role_index(Role::Queen)];
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::TAU / 6.0;
            let direction = Vec3::new(angle.cos(), 1.5 + (i % 3) as f32 * 0.16, angle.sin());
            let lifetime = 0.48 + (i % 3) as f32 * 0.08;
            commands.spawn((
                Mesh3d(self.artillery.puff.clone()),
                MeshMaterial3d(self.artillery.smoke[0].clone()),
                Transform::from_translation(impact).with_scale(Vec3::splat(0.056)),
                NotShadowCaster,
                Debris {
                    velocity: direction * 0.57,
                    remaining: lifetime,
                },
                ImpactSmoke {
                    lifetime,
                    spread: 0.17 + (i % 3) as f32 * 0.025,
                },
            ));
        }
        commands.spawn((
            Mesh3d(self.artillery.puff.clone()),
            MeshMaterial3d(visual.accent.clone()),
            Transform::from_translation(impact).with_scale(Vec3::splat(0.14)),
            NotShadowCaster,
            Debris {
                velocity: Vec3::Y * 0.20,
                remaining: 0.10,
            },
        ));
    }
}

#[derive(Component)]
pub(crate) struct ImpactSmoke {
    lifetime: f32,
    spread: f32,
}

pub(crate) fn animate_artillery_smoke(
    handles: Option<Res<SceneHandles>>,
    mut smoke: Query<
        (
            &Debris,
            &ImpactSmoke,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<Projectile>,
    >,
) {
    let Some(handles) = handles else {
        return;
    };
    for (debris, puff, mut transform, mut surface) in &mut smoke {
        let phase = (1.0 - debris.remaining / puff.lifetime).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(0.056 + puff.spread * phase.sqrt());
        let index = (phase * (SMOKE_STEPS - 1) as f32).round() as usize;
        let material = &handles.fx.artillery.smoke[index];
        if &surface.0 != material {
            surface.0 = material.clone();
        }
    }
}

pub(super) fn impact_offset(index: usize) -> Vec2 {
    [
        Vec2::new(-0.18, 0.02),
        Vec2::new(0.12, -0.10),
        Vec2::new(-0.07, 0.14),
        Vec2::new(0.21, 0.08),
        Vec2::new(0.00, -0.14),
        Vec2::new(-0.14, -0.10),
        Vec2::new(0.08, 0.13),
        Vec2::ZERO,
    ][index % 8]
}

pub(super) fn flight_variation(index: usize) -> f32 {
    [0.0, 0.060, 0.025, 0.075, 0.015, 0.050, 0.0, 0.085][index % 8]
}
