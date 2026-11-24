//! A stylized rooftop bottle arc, wick flame, and temporary square-bound fire.
use super::{Effects, Flight, Projectile, lathe, material};
use crate::characters::{BOARD_SCALE, MansionMolotovAssets};
use crate::{CaptureFxRequest, Debris, square_world};
use bevy::{light::NotShadowCaster, prelude::*};
use std::f32::consts::{PI, TAU};
pub(crate) const LAUNCH: f32 = 0.42;
const FIRE_SECONDS: f32 = 1.0;

#[derive(Component)]
pub(crate) struct MolotovFire;
#[derive(Clone, Copy)]
enum Kind {
    Wick,
    Flame,
    Smoke,
    Scorch,
}
#[derive(Component)]
pub(crate) struct MolotovPart {
    root: Entity,
    kind: Kind,
    phase: f32,
    base: Vec3,
    scale: Vec3,
}

#[derive(Default)]
pub(super) struct MolotovFx {
    bottle: MansionMolotovAssets,
    flame: Handle<Mesh>,
    puff: Handle<Mesh>,
    shard: Handle<Mesh>,
    scorch: Handle<Mesh>,
    orange: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    core: Handle<StandardMaterial>,
    smoke: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    charcoal: Handle<StandardMaterial>,
}
impl MolotovFx {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            bottle: MansionMolotovAssets::new(meshes, materials),
            flame: meshes.add(
                lathe(
                    12,
                    &[
                        (-0.012, 0.009),
                        (0.012, 0.028),
                        (0.055, 0.017),
                        (0.110, 0.001),
                    ],
                )
                .transformed_by(Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0))),
            ),
            puff: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            shard: meshes.add(Cuboid::new(0.013, 0.023, 0.007)),
            scorch: meshes.add(
                Torus::new(0.215, 0.252)
                    .mesh()
                    .major_resolution(40)
                    .minor_resolution(6),
            ),
            orange: materials.add(material(0xff6a16, 0.87)),
            gold: materials.add(material(0xffbd35, 0.95)),
            core: materials.add(material(0xfff1aa, 1.0)),
            smoke: materials.add(material(0x5d5750, 0.25)),
            glass: materials.add(material(0x529e69, 0.80)),
            charcoal: materials.add(material(0x362719, 0.50)),
        }
    }
}
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub(crate) fn hatch_fraction(seconds: f32) -> f32 {
    ease(seconds / 0.28) * (1.0 - ease((seconds - 0.68) / 0.27))
}

impl Effects {
    pub(super) fn spawn_mansion_molotov(&self, commands: &mut Commands, request: CaptureFxRequest) {
        let from = square_world(request.from);
        let target = square_world(request.target);
        let distance = from.distance(target);
        let forward = (target - from).normalize();
        let sideways = forward.cross(Vec3::Y);
        let anchor = crate::characters::mansion_launch(LAUNCH);
        let start = from + sideways * anchor.x + Vec3::Y * anchor.y - forward * anchor.z;
        let shot = Projectile {
            start,
            end: target + Vec3::Y * 0.07,
            elapsed: -LAUNCH,
            duration: (0.73 + distance * 0.04).clamp(0.77, 1.01),
            role: shakmaty::Role::Rook,
            index: 0,
            sideways,
            lane: 0.0,
            flight: Flight::Molotov {
                arc: 0.58 + 0.055 * distance,
                last_smoke: -0.08,
            },
        };
        let root = commands
            .spawn((
                Transform::from_translation(start)
                    .with_rotation(rotation(&shot, 0.0))
                    .with_scale(Vec3::splat(BOARD_SCALE * 1.25)),
                Visibility::Hidden,
                shot,
            ))
            .id();
        self.molotov.bottle.spawn(commands, root);
        for (i, scale) in [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.62, 0.78, 0.62),
            Vec3::new(0.30, 0.51, 0.30),
        ]
        .into_iter()
        .enumerate()
        {
            self.molotov_part(
                commands,
                root,
                Kind::Wick,
                i as f32 * 1.3,
                Vec3::new(0.014, 0.135, -0.001),
                scale,
                i,
            );
        }
    }
    fn molotov_part(
        &self,
        commands: &mut Commands,
        root: Entity,
        kind: Kind,
        phase: f32,
        base: Vec3,
        scale: Vec3,
        color: usize,
    ) {
        let (mesh, surface) = match kind {
            Kind::Smoke => (&self.molotov.puff, &self.molotov.smoke),
            Kind::Scorch => (&self.molotov.scorch, &self.molotov.charcoal),
            _ => (
                &self.molotov.flame,
                match color {
                    0 => &self.molotov.orange,
                    1 => &self.molotov.gold,
                    _ => &self.molotov.core,
                },
            ),
        };
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(surface.clone()),
            Transform::from_translation(base).with_scale(scale),
            NotShadowCaster,
            MolotovPart {
                root,
                kind,
                phase,
                base,
                scale,
            },
            ChildOf(root),
        ));
    }
    pub(crate) fn emit_molotov_smoke(&self, commands: &mut Commands, shot: &mut Projectile) {
        let Flight::Molotov { arc, last_smoke } = shot.flight else {
            return;
        };
        if shot.elapsed - last_smoke < 0.07 {
            return;
        }
        shot.flight = Flight::Molotov {
            arc,
            last_smoke: shot.elapsed,
        };
        let t = (shot.elapsed / shot.duration).clamp(0.0, 1.0);
        commands.spawn((
            Mesh3d(self.molotov.puff.clone()),
            MeshMaterial3d(self.molotov.smoke.clone()),
            Transform::from_translation(shot.position(t)).with_scale(Vec3::splat(0.025)),
            NotShadowCaster,
            Debris {
                velocity: Vec3::Y * 0.48,
                remaining: 0.20,
            },
        ));
    }
    pub(crate) fn spawn_molotov_impact(&self, commands: &mut Commands, shot: &Projectile) {
        let floor = shot.end.with_y(0.122);
        let root = commands
            .spawn((
                Transform::from_translation(floor),
                Visibility::Inherited,
                Debris {
                    velocity: Vec3::ZERO,
                    remaining: FIRE_SECONDS,
                },
                MolotovFire,
            ))
            .id();
        self.molotov_part(
            commands,
            root,
            Kind::Scorch,
            0.0,
            Vec3::ZERO,
            Vec3::new(1.15, 0.08, 1.15),
            0,
        );
        for i in 0..9 {
            let a = TAU * i as f32 / 9.0;
            let radius = if i % 3 == 0 { 0.11 } else { 0.23 };
            let base = Vec3::new(radius * a.cos(), 0.006, radius * a.sin());
            let scale = Vec3::new(0.90, 1.8 + (i % 3) as f32 * 0.55, 0.90);
            self.molotov_part(commands, root, Kind::Flame, i as f32 * 1.7, base, scale, 0);
            self.molotov_part(
                commands,
                root,
                Kind::Flame,
                i as f32 * 1.7 + 0.3,
                base,
                scale * Vec3::new(0.55, 0.72, 0.55),
                1,
            );
        }
        for i in 0..5 {
            let a = TAU * i as f32 / 5.0;
            self.molotov_part(
                commands,
                root,
                Kind::Smoke,
                i as f32 * 0.8,
                Vec3::new(a.cos() * 0.10, 0.12 + i as f32 * 0.025, a.sin() * 0.10),
                Vec3::new(0.065, 0.085, 0.065),
                0,
            );
        }
        for i in 0..10 {
            let a = TAU * i as f32 / 10.0;
            commands.spawn((
                Mesh3d(self.molotov.shard.clone()),
                MeshMaterial3d(self.molotov.glass.clone()),
                Transform::from_translation(shot.end).with_rotation(Quat::from_rotation_y(a)),
                NotShadowCaster,
                Debris {
                    velocity: Vec3::new(
                        a.cos() * 0.65,
                        0.55 + (i % 3) as f32 * 0.12,
                        a.sin() * 0.65,
                    ),
                    remaining: 0.27 + (i % 3) as f32 * 0.06,
                },
            ));
        }
    }
}

pub(super) fn position(shot: &Projectile, t: f32) -> Vec3 {
    let Flight::Molotov { arc, .. } = shot.flight else {
        unreachable!()
    };
    let t = t.clamp(0.0, 1.0);
    shot.start.lerp(shot.end, t) + Vec3::Y * (4.0 * t * (1.0 - t) * arc)
}
pub(super) fn rotation(shot: &Projectile, t: f32) -> Quat {
    let forward = (shot.end - shot.start).with_y(0.0).normalize();
    Quat::from_rotation_arc(Vec3::NEG_Z, forward)
        * Quat::from_rotation_x(-t.clamp(0.0, 1.0) * TAU * 1.35)
        * Quat::from_rotation_z(0.18)
}
pub(crate) fn animate_molotov(
    shots: Query<&Projectile>,
    fires: Query<&Debris, With<MolotovFire>>,
    mut parts: Query<(&MolotovPart, &mut Transform)>,
) {
    for (part, mut transform) in &mut parts {
        let age = if let Ok(shot) = shots.get(part.root) {
            shot.elapsed.max(0.0)
        } else if let Ok(fire) = fires.get(part.root) {
            FIRE_SECONDS - fire.remaining
        } else {
            continue;
        };
        let flicker = 0.88
            + 0.13 * (age * 22.0 + part.phase).sin()
            + 0.07 * (age * 37.0 + part.phase * 1.8).sin();
        let envelope = if matches!(part.kind, Kind::Wick) {
            1.0
        } else {
            ease(age / 0.07) * ease((FIRE_SECONDS - age) / 0.22)
        };
        match part.kind {
            Kind::Wick | Kind::Flame => {
                transform.translation = part.base
                    + Vec3::new(
                        0.006 * (age * 15.0 + part.phase).sin(),
                        0.0,
                        0.004 * (age * 19.0 + part.phase).cos(),
                    );
                transform.scale = part.scale * Vec3::new(envelope, envelope * flicker, envelope);
                transform.rotation = Quat::from_rotation_y(age * 1.7 + part.phase);
            }
            Kind::Smoke => {
                transform.translation = part.base
                    + Vec3::Y * age * 0.36
                    + Vec3::X * (0.023 * (age * 3.0 + part.phase).sin());
                transform.scale = part.scale * ((0.7 + age * 1.8) * envelope);
            }
            Kind::Scorch => {
                transform.scale = part.scale * ease(age / 0.10) * ease((FIRE_SECONDS - age) / 0.12);
            }
        }
    }
}
