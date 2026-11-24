//! A single shotgun blast: small spreading pellets, muzzle smoke, and one shell.
use super::{Effects, Flight, Projectile, lathe, material};
use crate::{CaptureFxRequest, Debris, SceneHandles, square_world};
use bevy::{light::NotShadowCaster, prelude::*};
use std::f32::consts::TAU;
pub(crate) const LAUNCH: f32 = 0.68;
pub(crate) const PELLETS: usize = 9;
const SMOKE_STEPS: usize = 12;

pub(crate) fn aim_pitch(distance: f32, hit_height: f32) -> f32 {
    (hit_height - 0.72).atan2((distance - 0.078).max(0.35))
}
#[derive(Clone, Copy)]
enum Kind {
    Flash,
    Smoke { phase: f32 },
}
#[derive(Component)]
pub(crate) struct ShotgunCloud {
    origin: Vec3,
    direction: Vec3,
    lifetime: f32,
    kind: Kind,
}
#[derive(Default)]
pub(super) struct ShotgunFx {
    pellet: Handle<Mesh>,
    trail: Handle<Mesh>,
    flash: Handle<Mesh>,
    puff: Handle<Mesh>,
    shell: Handle<Mesh>,
    rim: Handle<Mesh>,
    steel: Handle<StandardMaterial>,
    brass: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    amber: Handle<StandardMaterial>,
    core: Handle<StandardMaterial>,
    smoke: [Handle<StandardMaterial>; SMOKE_STEPS],
}
impl ShotgunFx {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            pellet: meshes.add(Sphere::new(0.010).mesh().ico(1).unwrap()),
            trail: meshes.add(lathe(6, &[(-0.084, 0.0005), (-0.012, 0.004), (0.0, 0.003)])),
            flash: meshes.add(lathe(
                8,
                &[
                    (-0.012, 0.009),
                    (0.060, 0.074),
                    (0.145, 0.028),
                    (0.235, 0.001),
                ],
            )),
            puff: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            shell: meshes.add(lathe(
                16,
                &[
                    (-0.033, 0.013),
                    (-0.025, 0.015),
                    (0.031, 0.015),
                    (0.033, 0.013),
                ],
            )),
            rim: meshes.add(lathe(
                16,
                &[(-0.037, 0.017), (-0.031, 0.017), (-0.021, 0.015)],
            )),
            steel: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(184, 194, 199),
                metallic: 0.72,
                perceptual_roughness: 0.28,
                ..default()
            }),
            brass: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(218, 170, 73),
                metallic: 0.75,
                perceptual_roughness: 0.26,
                ..default()
            }),
            red: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(158, 39, 26),
                perceptual_roughness: 0.45,
                ..default()
            }),
            amber: materials.add(material(0xffcc79, 0.90)),
            core: materials.add(material(0xffefbd, 1.0)),
            smoke: std::array::from_fn(|i| {
                materials.add(material(
                    0xaaa59a,
                    0.32 * (1.0 - i as f32 / (SMOKE_STEPS - 1) as f32),
                ))
            }),
        }
    }
}
impl Effects {
    pub(super) fn spawn_king_shotgun(
        &self,
        commands: &mut Commands,
        request: CaptureFxRequest,
        hit_height: f32,
    ) {
        let from = square_world(request.from);
        let target = square_world(request.target);
        let forward = (target - from).normalize();
        let sideways = forward.cross(Vec3::Y);
        let aim = aim_pitch(from.distance(target), hit_height);
        let world = |p: Vec3| from + sideways * p.x + Vec3::Y * p.y - forward * p.z;
        let start = world(crate::characters::king_muzzle(LAUNCH, aim));
        let eject = world(crate::characters::king_ejection(LAUNCH, aim));
        for index in 0..PELLETS {
            let spread = if index == 0 {
                Vec2::ZERO
            } else {
                let angle = TAU * (index - 1) as f32 / (PELLETS - 1) as f32;
                Vec2::new(angle.cos() * 0.086, angle.sin() * 0.070)
            };
            let shot = Projectile {
                start,
                end: target + Vec3::Y * (hit_height + spread.y) + sideways * spread.x,
                elapsed: -LAUNCH,
                duration: 0.14 + from.distance(target) * 0.024,
                role: shakmaty::Role::King,
                index,
                sideways,
                lane: 0.0,
                flight: Flight::KingPellet { eject },
            };
            let root = commands
                .spawn((
                    Mesh3d(self.shotgun.pellet.clone()),
                    MeshMaterial3d(self.shotgun.steel.clone()),
                    Transform::from_translation(start).with_rotation(shot.rotation(0.0)),
                    Visibility::Hidden,
                    NotShadowCaster,
                    shot,
                ))
                .id();
            commands.spawn((
                Mesh3d(self.shotgun.trail.clone()),
                MeshMaterial3d(self.shotgun.amber.clone()),
                Transform::IDENTITY,
                NotShadowCaster,
                ChildOf(root),
            ));
        }
    }
    pub(crate) fn spawn_king_flash(&self, commands: &mut Commands, shot: &Projectile) {
        let Flight::KingPellet { eject } = shot.flight else {
            return;
        };
        let direction = (shot.end - shot.start).normalize();
        self.king_cloud(commands, shot.start, direction, 0.075, Kind::Flash);
        for i in 0..3 {
            self.king_cloud(
                commands,
                shot.start + direction * (0.07 + i as f32 * 0.065),
                direction,
                0.53 + i as f32 * 0.06,
                Kind::Smoke { phase: i as f32 },
            );
        }
        let shell = commands
            .spawn((
                Mesh3d(self.shotgun.shell.clone()),
                MeshMaterial3d(self.shotgun.red.clone()),
                Transform::from_translation(eject)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, shot.sideways)),
                NotShadowCaster,
                Debris {
                    velocity: shot.sideways * 0.70 + Vec3::Y * 0.82 - direction * 0.10,
                    remaining: 0.64,
                },
            ))
            .id();
        commands.spawn((
            Mesh3d(self.shotgun.rim.clone()),
            MeshMaterial3d(self.shotgun.brass.clone()),
            Transform::IDENTITY,
            NotShadowCaster,
            ChildOf(shell),
        ));
    }
    fn king_cloud(
        &self,
        commands: &mut Commands,
        origin: Vec3,
        direction: Vec3,
        lifetime: f32,
        kind: Kind,
    ) {
        let (mesh, surface, scale) = match kind {
            Kind::Flash => (&self.shotgun.flash, &self.shotgun.core, Vec3::ONE),
            Kind::Smoke { .. } => (
                &self.shotgun.puff,
                &self.shotgun.smoke[0],
                Vec3::splat(0.035),
            ),
        };
        commands.spawn((
            ShotgunCloud {
                origin,
                direction,
                lifetime,
                kind,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(surface.clone()),
            Transform::from_translation(origin)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, direction))
                .with_scale(scale),
            NotShadowCaster,
            Debris {
                velocity: Vec3::ZERO,
                remaining: lifetime,
            },
        ));
    }
    pub(crate) fn spawn_king_impact(&self, commands: &mut Commands, shot: &Projectile) {
        let direction = (shot.end - shot.start).normalize();
        for i in 0..9 {
            let angle = TAU * i as f32 / 9.0;
            let velocity = shot.sideways * angle.cos() * 0.65
                + Vec3::Y * (0.42 + angle.sin().abs() * 0.60)
                - direction * 0.32;
            commands.spawn((
                Mesh3d(self.shotgun.trail.clone()),
                MeshMaterial3d(self.shotgun.amber.clone()),
                Transform::from_translation(shot.end)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, velocity.normalize()))
                    .with_scale(Vec3::splat(0.9)),
                NotShadowCaster,
                Debris {
                    velocity,
                    remaining: 0.32 + (i % 3) as f32 * 0.08,
                },
            ));
        }
        for i in 0..2 {
            self.king_cloud(
                commands,
                shot.end + Vec3::Y * i as f32 * 0.03,
                -direction,
                0.56 + i as f32 * 0.06,
                Kind::Smoke {
                    phase: i as f32 + 3.0,
                },
            );
        }
    }
}
pub(crate) fn animate_shotgun(
    handles: Option<Res<SceneHandles>>,
    mut clouds: Query<(
        &ShotgunCloud,
        &Debris,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(handles) = handles else { return };
    for (cloud, debris, mut transform, mut surface) in &mut clouds {
        let t = (1.0 - debris.remaining / cloud.lifetime).clamp(0.0, 1.0);
        match cloud.kind {
            Kind::Flash => {
                transform.scale = Vec3::new(1.0 - t * 0.65, 1.0 - t * 0.65, 1.0 - t * 0.40)
            }
            Kind::Smoke { phase } => {
                transform.translation = cloud.origin
                    + cloud.direction * t * 0.13
                    + Vec3::Y * (t * 0.11)
                    + Vec3::X * (phase * 1.9).sin() * t * 0.025;
                transform.scale = Vec3::splat(0.035 + t * (0.095 + phase.min(3.0) * 0.012));
                surface.0 =
                    handles.fx.shotgun.smoke[(t * (SMOKE_STEPS - 1) as f32) as usize].clone();
            }
        }
    }
}
