//! The queen's short, sweeping mini-Uzi burst. Only the last round defeats.
use super::{Effects, Flight, Projectile, lathe, material};
use crate::{CaptureFxRequest, Debris, SceneHandles, square_world};
use bevy::{light::NotShadowCaster, prelude::*};
use std::f32::consts::{PI, TAU};
pub(crate) const LAUNCH: f32 = 0.46;
pub(crate) const INTERVAL: f32 = 0.07;
pub(crate) const ROUNDS: usize = 10;
const SMOKE_STEPS: usize = 10;
pub(crate) fn flight_seconds(distance: f32) -> f32 {
    (0.15 + distance * 0.026).clamp(0.18, 0.37)
}
pub(crate) fn aim_pitch(distance: f32, hit_height: f32) -> f32 {
    (hit_height - 0.72).atan2((distance - 0.25).max(0.5))
}
pub(crate) fn spread(phase: f32) -> Vec2 {
    let angle = TAU * phase.clamp(0.0, (ROUNDS - 1) as f32) / (ROUNDS - 1) as f32;
    Vec2::new(angle.sin() * 0.082, (angle * 1.5).cos() * 0.014)
}
#[derive(Component)]
pub(crate) enum SmgEffect {
    Smoke {
        origin: Vec3,
        direction: Vec3,
        lifetime: f32,
    },
    Casing {
        center: Vec3,
        ground: f32,
        index: usize,
        landed: bool,
    },
    Fluid {
        ground: f32,
        lifetime: f32,
        size: f32,
    },
}
impl SmgEffect {
    pub(crate) fn is_settled_casing(&self) -> bool {
        matches!(self, Self::Casing { landed: true, .. })
    }
}
#[derive(Default)]
pub(super) struct SmgFx {
    bullet: Handle<Mesh>,
    trail: Handle<Mesh>,
    flash: Handle<Mesh>,
    case: Handle<Mesh>,
    puff: Handle<Mesh>,
    brass: Handle<StandardMaterial>,
    amber: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
    core: Handle<StandardMaterial>,
    smoke: [Handle<StandardMaterial>; SMOKE_STEPS],
    fluid: [Handle<StandardMaterial>; SMOKE_STEPS],
}
impl SmgFx {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            bullet: meshes.add(lathe(
                12,
                &[
                    (-0.029, 0.006),
                    (-0.023, 0.007),
                    (0.010, 0.007),
                    (0.023, 0.004),
                    (0.029, 0.001),
                ],
            )),
            trail: meshes.add(lathe(6, &[(-0.11, 0.0003), (-0.025, 0.004), (0.0, 0.002)])),
            flash: meshes.add(lathe(6, &[(-0.008, 0.004), (0.025, 0.034), (0.105, 0.001)])),
            case: meshes.add(lathe(
                12,
                &[(-0.021, 0.007), (-0.017, 0.008), (0.020, 0.007)],
            )),
            puff: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            brass: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(218, 172, 77),
                metallic: 0.75,
                perceptual_roughness: 0.28,
                ..default()
            }),
            amber: materials.add(material(0xffd88b, 0.80)),
            orange: materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(3.0, 1.1, 0.15),
                ..material(0xffb43d, 1.0)
            }),
            core: materials.add(material(0xfff1cc, 1.0)),
            smoke: std::array::from_fn(|i| {
                materials.add(material(
                    0xaaa59a,
                    0.24 * (1.0 - i as f32 / (SMOKE_STEPS - 1) as f32),
                ))
            }),
            fluid: std::array::from_fn(|i| {
                materials.add(material(
                    0xb42e45,
                    0.90 * (1.0 - i as f32 / (SMOKE_STEPS - 1) as f32),
                ))
            }),
        }
    }
}
impl Effects {
    pub(super) fn spawn_queen_smg(
        &self,
        commands: &mut Commands,
        request: CaptureFxRequest,
        hit_height: f32,
    ) {
        let from = square_world(request.from);
        let target = square_world(request.target);
        let distance = from.distance(target);
        let forward = (target - from).normalize();
        let sideways = forward.cross(Vec3::Y);
        let aim = aim_pitch(distance, hit_height);
        for index in 0..ROUNDS {
            let seconds = LAUNCH + index as f32 * INTERVAL;
            let muzzle = crate::characters::queen_muzzle(seconds, distance, aim);
            let eject = crate::characters::queen_ejection(seconds, distance, aim);
            let world = |p: Vec3| from + sideways * p.x + Vec3::Y * p.y - forward * p.z;
            let offset = spread(index as f32);
            let shot = Projectile {
                start: world(muzzle),
                end: target + sideways * offset.x + Vec3::Y * (hit_height + offset.y),
                elapsed: -seconds,
                duration: flight_seconds(distance),
                role: shakmaty::Role::Queen,
                index,
                sideways,
                lane: 0.0,
                flight: Flight::QueenBullet {
                    eject: world(eject),
                    from,
                },
            };
            let root = commands
                .spawn((
                    Mesh3d(self.smg.bullet.clone()),
                    MeshMaterial3d(self.smg.brass.clone()),
                    Transform::from_translation(shot.start).with_rotation(shot.rotation(0.0)),
                    Visibility::Hidden,
                    NotShadowCaster,
                    shot,
                ))
                .id();
            commands.spawn((
                Mesh3d(self.smg.trail.clone()),
                MeshMaterial3d(self.smg.amber.clone()),
                Transform::IDENTITY,
                NotShadowCaster,
                ChildOf(root),
            ));
        }
    }
    pub(crate) fn spawn_smg_flash(&self, commands: &mut Commands, shot: &Projectile) {
        let Flight::QueenBullet { eject, from } = shot.flight else {
            return;
        };
        let direction = (shot.end - shot.start).normalize();
        let flash = commands
            .spawn((
                Mesh3d(self.smg.flash.clone()),
                MeshMaterial3d(self.smg.orange.clone()),
                Transform::from_translation(shot.start)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, direction)),
                NotShadowCaster,
                Debris {
                    velocity: Vec3::ZERO,
                    remaining: 0.048,
                },
            ))
            .id();
        commands.spawn((
            Mesh3d(self.smg.flash.clone()),
            MeshMaterial3d(self.smg.core.clone()),
            Transform::from_scale(Vec3::new(0.5, 0.5, 0.73)),
            NotShadowCaster,
            ChildOf(flash),
        ));
        commands.spawn((
            SmgEffect::Casing {
                center: from,
                ground: from.y + 0.008,
                index: shot.index,
                landed: false,
            },
            Mesh3d(self.smg.case.clone()),
            MeshMaterial3d(self.smg.brass.clone()),
            Transform::from_translation(eject)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, shot.sideways)),
            NotShadowCaster,
            Debris {
                velocity: shot.sideways * (0.18 + (shot.index % 3) as f32 * 0.025) + Vec3::Y * 0.28
                    - direction * 0.04,
                remaining: 1.10,
            },
        ));
        if shot.index % 3 == 0 {
            self.smg_smoke(commands, shot.start + direction * 0.05, direction, 0.25);
        }
    }
    pub(crate) fn spawn_smg_impact(&self, commands: &mut Commands, shot: &Projectile) {
        let Flight::QueenBullet { from, .. } = shot.flight else {
            return;
        };
        let ground = from.y + 0.008;
        let final_round = shot.index + 1 == ROUNDS;
        let count = if final_round { 7 } else { 2 };
        let direction = (shot.end - shot.start).normalize();
        for i in 0..count {
            let angle = TAU * i as f32 / count as f32 + shot.index as f32 * 0.4;
            let velocity = shot.sideways * angle.cos() * 0.40
                + Vec3::Y * (0.26 + angle.sin().abs() * 0.32)
                - direction * 0.15;
            commands.spawn((
                Mesh3d(self.smg.trail.clone()),
                MeshMaterial3d(self.smg.amber.clone()),
                Transform::from_translation(shot.end)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, velocity.normalize()))
                    .with_scale(Vec3::splat(if final_round { 0.8 } else { 0.4 })),
                NotShadowCaster,
                Debris {
                    velocity,
                    remaining: if final_round {
                        0.46 + (i % 2) as f32 * 0.06
                    } else {
                        0.11
                    },
                },
            ));
        }
        if final_round || shot.index % 3 == 0 {
            self.smg_smoke(
                commands,
                shot.end,
                -direction,
                if final_round { 0.49 } else { 0.25 },
            );
        }
        // One stylized fluid-and-smoke treatment is used for every defender.
        for i in 0..if final_round { 14 } else { 2 } {
            let angle = TAU * i as f32 / 7.0 + shot.index as f32 * 0.61;
            let size = 0.006 + (i % 3) as f32 * 0.0015;
            let lifetime = if final_round {
                0.46 + (i % 3) as f32 * 0.025
            } else {
                0.25
            };
            let velocity = direction * (0.24 + (i % 4) as f32 * 0.07)
                + shot.sideways * angle.cos() * 0.45
                + Vec3::Y * (0.35 + angle.sin().abs() * 0.45);
            commands.spawn((
                SmgEffect::Fluid {
                    ground,
                    lifetime,
                    size,
                },
                Mesh3d(self.smg.puff.clone()),
                MeshMaterial3d(self.smg.fluid[0].clone()),
                Transform::from_translation(shot.end)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, velocity.normalize()))
                    .with_scale(Vec3::new(
                        size,
                        size * if final_round && i < 2 { 5.0 } else { 2.4 },
                        size,
                    )),
                NotShadowCaster,
                Debris {
                    velocity,
                    remaining: lifetime,
                },
            ));
        }
    }
    fn smg_smoke(&self, commands: &mut Commands, origin: Vec3, direction: Vec3, lifetime: f32) {
        commands.spawn((
            SmgEffect::Smoke {
                origin,
                direction,
                lifetime,
            },
            Mesh3d(self.smg.puff.clone()),
            MeshMaterial3d(self.smg.smoke[0].clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(0.025)),
            NotShadowCaster,
            Debris {
                velocity: Vec3::ZERO,
                remaining: lifetime,
            },
        ));
    }
}
pub(crate) fn animate_smg(
    handles: Option<Res<SceneHandles>>,
    time: Res<Time>,
    mut effects: Query<(
        &mut SmgEffect,
        &mut Debris,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(handles) = handles else { return };
    let dt = time.delta_secs();
    for (mut effect, mut debris, mut t, mut surface) in &mut effects {
        match &mut *effect {
            SmgEffect::Smoke {
                origin,
                direction,
                lifetime,
            } => {
                let age = (1.0 - debris.remaining / *lifetime).clamp(0.0, 1.0);
                t.translation = *origin + *direction * age * 0.10 + Vec3::Y * age * 0.07;
                t.scale = Vec3::splat(0.025 + age * 0.060);
                t.rotation = Quat::from_rotation_y(PI * age);
                surface.0 = handles.fx.smg.smoke[(age * (SMOKE_STEPS - 1) as f32) as usize].clone();
            }
            SmgEffect::Casing {
                center,
                ground,
                index,
                landed,
            } => {
                if *landed {
                    continue;
                }
                debris.velocity.y -= 9.2 * dt;
                t.translation += debris.velocity * dt;
                t.rotate_x(dt * 15.0);
                t.rotate_z(dt * 11.0);
                let offset = Vec2::new(t.translation.x - center.x, t.translation.z - center.z)
                    .clamp_length_max(0.43);
                t.translation.x = center.x + offset.x;
                t.translation.z = center.z + offset.y;
                if t.translation.y <= *ground {
                    t.translation.y = *ground;
                    debris.velocity = Vec3::ZERO;
                    *landed = true;
                    t.rotation = Quat::from_rotation_y(*index as f32 * 2.17);
                }
            }
            SmgEffect::Fluid {
                ground,
                lifetime,
                size,
            } => {
                let age = (1.0 - debris.remaining / *lifetime).clamp(0.0, 1.0);
                debris.velocity.y -= 5.8 * dt;
                t.translation += debris.velocity * dt;
                t.rotation = Quat::from_rotation_arc(Vec3::Y, debris.velocity.normalize_or_zero());
                t.scale = Vec3::new(*size, *size * (2.4 + 2.0 * (1.0 - age)), *size);
                surface.0 = handles.fx.smg.fluid[(age * (SMOKE_STEPS - 1) as f32) as usize].clone();
                if t.translation.y < *ground {
                    t.translation.y = *ground;
                    debris.remaining = debris.remaining.min(0.06);
                    t.scale.y = *size * 0.4;
                }
            }
        }
    }
}
