//! Two tethered darts followed by a short, square-bound electrical stun.
use super::{Effects, Flight, Projectile, lathe, material};
use crate::{CaptureFxRequest, Debris, square_world};
use bevy::{light::NotShadowCaster, prelude::*};
use std::f32::consts::{PI, TAU};
pub(crate) const LAUNCH: f32 = 0.34;
pub(crate) const HOLD: f32 = 0.42;
const SEGMENTS: usize = 14;

pub(crate) fn flight_seconds(distance: f32) -> f32 {
    (0.22 + distance * 0.025).clamp(0.25, 0.48)
}

#[derive(Component)]
pub(crate) struct TaserCircuit {
    pub from: Vec3,
    forward: Vec3,
    sideways: Vec3,
    pub ends: [Vec3; 2],
    pub flight: f32,
    lifetime: f32,
}
impl TaserCircuit {
    fn muzzle(&self, age: f32) -> Vec3 {
        let p = crate::characters::police_muzzle(age, self.flight);
        self.from + self.sideways * p.x + Vec3::Y * p.y - self.forward * p.z
    }
    fn start(&self, age: f32, wire: usize) -> Vec3 {
        self.muzzle(age) + self.sideways * (wire as f32 - 0.5) * 0.0175
    }
    fn cable(&self, age: f32, wire: usize, t: f32) -> Vec3 {
        let progress = ((age - LAUNCH) / self.flight).clamp(0.0, 1.0);
        let dart = self.start(LAUNCH, wire).lerp(self.ends[wire], progress);
        let start = self.start(age, wire);
        let sag = (0.055 + start.distance(dart) * 0.025).min(0.19);
        start.lerp(dart, t) - Vec3::Y * (PI * t).sin() * sag * progress
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Cable { wire: usize, segment: usize },
    Pulse { wire: usize },
    Arc { segment: usize },
    Dart { wire: usize },
}
#[derive(Component)]
pub(crate) struct TaserPart {
    root: Entity,
    kind: Kind,
}
#[derive(Default)]
pub(super) struct TaserFx {
    dart: Handle<Mesh>,
    wire: Handle<Mesh>,
    spark: Handle<Mesh>,
    steel: Handle<StandardMaterial>,
    cable: Handle<StandardMaterial>,
    cyan: Handle<StandardMaterial>,
}
impl TaserFx {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            dart: meshes.add(lathe(
                12,
                &[
                    (-0.052, 0.010),
                    (-0.043, 0.017),
                    (0.013, 0.017),
                    (0.023, 0.011),
                    (0.064, 0.001),
                ],
            )),
            wire: meshes.add(Cylinder::new(0.0038, 1.0)),
            spark: meshes.add(Sphere::new(0.017).mesh().ico(1).unwrap()),
            steel: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(206, 218, 224),
                metallic: 0.75,
                perceptual_roughness: 0.28,
                ..default()
            }),
            cable: materials.add(material(0x9da7b8, 1.0)),
            cyan: materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(2.5, 7.0, 9.0),
                ..material(0xc7f9ff, 1.0)
            }),
        }
    }
}
impl Effects {
    pub(super) fn spawn_police_taser(
        &self,
        commands: &mut Commands,
        request: CaptureFxRequest,
        hit_height: f32,
    ) {
        let from = square_world(request.from);
        let target = square_world(request.target);
        let forward = (target - from).normalize();
        let sideways = forward.cross(Vec3::Y);
        let flight = flight_seconds(from.distance(target));
        let circuit = TaserCircuit {
            from,
            forward,
            sideways,
            flight,
            lifetime: LAUNCH + flight + HOLD + 0.50,
            ends: [-1.0, 1.0].map(|s| {
                target + Vec3::Y * (hit_height + s * 0.028) + sideways * s * 0.045 - forward * 0.06
            }),
        };
        for wire in 0..2 {
            let shot = Projectile {
                start: circuit.start(LAUNCH, wire),
                end: circuit.ends[wire],
                elapsed: -LAUNCH,
                duration: flight,
                role: shakmaty::Role::Bishop,
                index: wire,
                sideways,
                lane: 0.0,
                flight: Flight::TaserDart,
            };
            commands.spawn((
                Mesh3d(self.taser.dart.clone()),
                MeshMaterial3d(self.taser.steel.clone()),
                Transform::from_translation(shot.start).with_rotation(shot.rotation(0.0)),
                Visibility::Hidden,
                shot,
            ));
        }
        let root = commands
            .spawn((
                Debris {
                    velocity: Vec3::ZERO,
                    remaining: circuit.lifetime,
                },
                Transform::IDENTITY,
                Visibility::Inherited,
                circuit,
            ))
            .id();
        let mut part = |kind, mesh: &Handle<Mesh>, surface: &Handle<StandardMaterial>| {
            commands.spawn((
                TaserPart { root, kind },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(surface.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                NotShadowCaster,
                ChildOf(root),
            ));
        };
        for wire in 0..2 {
            for segment in 0..SEGMENTS {
                part(
                    Kind::Cable { wire, segment },
                    &self.taser.wire,
                    &self.taser.cable,
                );
            }
            part(Kind::Pulse { wire }, &self.taser.spark, &self.taser.cyan);
            part(Kind::Dart { wire }, &self.taser.dart, &self.taser.steel);
        }
        for segment in 0..8 {
            part(Kind::Arc { segment }, &self.taser.wire, &self.taser.cyan);
        }
    }
}
fn segment_transform(a: Vec3, b: Vec3) -> Transform {
    let d = b - a;
    Transform::from_translation((a + b) * 0.5)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, d.normalize_or_zero()))
        .with_scale(Vec3::new(1.0, d.length().max(0.0001), 1.0))
}
pub(crate) fn animate_taser(
    circuits: Query<(&TaserCircuit, &Debris)>,
    mut parts: Query<(&TaserPart, &mut Transform, &mut Visibility)>,
) {
    for (part, mut transform, mut visible) in &mut parts {
        let Ok((circuit, debris)) = circuits.get(part.root) else {
            continue;
        };
        let age = circuit.lifetime - debris.remaining;
        let connected = age - LAUNCH - circuit.flight;
        *visible = Visibility::Hidden;
        if age < LAUNCH || connected > HOLD {
            continue;
        }
        match part.kind {
            Kind::Cable { wire, segment } => {
                *transform = segment_transform(
                    circuit.cable(age, wire, segment as f32 / SEGMENTS as f32),
                    circuit.cable(age, wire, (segment + 1) as f32 / SEGMENTS as f32),
                );
            }
            Kind::Pulse { wire } => {
                if connected < 0.0 {
                    continue;
                }
                let phase = (connected / 0.21).fract();
                transform.translation = circuit.cable(age, wire, phase);
                transform.scale = Vec3::splat(0.85 + 0.3 * (TAU * phase).sin().abs());
            }
            Kind::Dart { wire } => {
                if connected < 0.0 {
                    continue;
                }
                transform.translation = circuit.ends[wire];
                transform.rotation = Quat::from_rotation_arc(
                    Vec3::Z,
                    (circuit.ends[wire] - circuit.start(LAUNCH, wire)).normalize(),
                );
            }
            Kind::Arc { segment } => {
                if connected < 0.0 || (connected / 0.105).fract() > 0.70 {
                    continue;
                }
                let point = |i: usize| {
                    let t = i as f32 / 8.0;
                    circuit.ends[0].lerp(circuit.ends[1], t)
                        + circuit.sideways
                            * (PI * t).sin()
                            * (if i % 2 == 0 { 0.022 } else { -0.022 })
                        + Vec3::Y * (TAU * connected * 17.0 + i as f32 * 2.3).sin() * 0.018
                };
                *transform = segment_transform(point(segment), point(segment + 1));
                transform.scale.x = 1.4;
                transform.scale.z = 1.4;
            }
        }
        *visible = Visibility::Visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn police_taser_cables_match_muzzle_and_dart_endpoints() {
        let c = TaserCircuit {
            from: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            sideways: Vec3::X,
            ends: [Vec3::new(-0.04, 0.56, -3.0), Vec3::new(0.04, 0.61, -3.0)],
            flight: flight_seconds(3.0),
            lifetime: 1.5,
        };
        for age in [
            LAUNCH,
            LAUNCH + 0.1,
            LAUNCH + c.flight,
            LAUNCH + c.flight + HOLD,
        ] {
            for wire in 0..2 {
                assert!(c.cable(age, wire, 0.0).distance(c.start(age, wire)) < 0.00001);
                let t = ((age - LAUNCH) / c.flight).clamp(0.0, 1.0);
                assert!(
                    c.cable(age, wire, 1.0)
                        .distance(c.start(LAUNCH, wire).lerp(c.ends[wire], t))
                        < 0.00001
                );
            }
        }
    }
}
