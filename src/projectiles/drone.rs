//! The rook's drone lifts through its battlement, releases a grenade, and docks.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{light::NotShadowCaster, prelude::*};
use shakmaty::Role;

use super::{Effects, Flight, Projectile, lathe, role_index, shot_layer};
use crate::SceneHandles;

pub(crate) const LAUNCH_DELAY: f32 = 0.24;
pub(crate) const FLIGHT_SECONDS: f32 = 1.85;
pub(crate) const SEQUENCE_SECONDS: f32 = 2.40;
const TAKEOFF_END: f32 = 0.32;
const OUTBOUND_END: f32 = 0.86;
const RELEASE: f32 = 0.98;
const RETURN_START: f32 = 1.08;
const RETURN_END: f32 = 1.52;
const GRENADE_SECONDS: f32 = 0.54;

#[derive(Default)]
pub(super) struct DroneMeshes {
    propeller: Handle<Mesh>,
    cage: Handle<Mesh>,
    light: Handle<Mesh>,
    grenade: Handle<Mesh>,
    grenade_surface: Handle<StandardMaterial>,
}

impl DroneMeshes {
    pub(super) fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            propeller: meshes.add(join(vec![
                (Cuboid::new(0.141, 0.007, 0.020).into(), Transform::IDENTITY),
                (
                    Cuboid::new(0.141, 0.007, 0.020).into(),
                    Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_2)),
                ),
            ])),
            cage: meshes.add(
                Torus::new(0.068, 0.078)
                    .mesh()
                    .major_resolution(24)
                    .minor_resolution(6),
            ),
            light: meshes.add(Cuboid::new(0.041, 0.018, 0.012)),
            grenade: meshes.add(grenade()),
            grenade_surface: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(79, 96, 40),
                metallic: 0.12,
                perceptual_roughness: 0.62,
                ..default()
            }),
        }
    }
}

fn motors() -> [Vec3; 4] {
    [
        Vec3::new(-0.125, 0.035, -0.110),
        Vec3::new(0.125, 0.035, -0.110),
        Vec3::new(-0.125, 0.035, 0.110),
        Vec3::new(0.125, 0.035, 0.110),
    ]
}

pub(super) fn body() -> Mesh {
    let mut parts = vec![
        (Cuboid::new(0.172, 0.083, 0.147).into(), Transform::IDENTITY),
        (
            Cuboid::new(0.074, 0.037, 0.062).into(),
            Transform::from_xyz(0.0, -0.042, -0.057),
        ),
    ];
    for motor in motors() {
        let axis = motor;
        parts.push((
            Cuboid::new(0.026, axis.length(), 0.023).into(),
            Transform::from_translation(motor * 0.5)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, axis.normalize())),
        ));
        parts.push((
            Cylinder::new(0.031, 0.044).mesh().resolution(16).build(),
            Transform::from_translation(motor),
        ));
    }
    join(parts)
}

pub(super) fn metal() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(40, 49, 53),
        metallic: 0.38,
        perceptual_roughness: 0.43,
        ..default()
    }
}

fn grenade() -> Mesh {
    let mut mesh = lathe(
        20,
        &[
            (-0.094, 0.022),
            (-0.078, 0.052),
            (-0.038, 0.067),
            (0.024, 0.067),
            (0.063, 0.050),
            (0.078, 0.030),
        ],
    )
    .transformed_by(Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)));
    let cap: Mesh = Cylinder::new(0.027, 0.030).mesh().resolution(12).build();
    mesh.merge(&clean(cap).transformed_by(Transform::from_xyz(0.0, 0.082, 0.0)))
        .unwrap();
    let lever = Cuboid::new(0.028, 0.012, 0.112);
    mesh.merge(&clean(lever.into()).transformed_by(
        Transform::from_xyz(0.0, 0.099, -0.012).with_rotation(Quat::from_rotation_x(-0.21)),
    ))
    .unwrap();
    mesh
}

fn clean(mut mesh: Mesh) -> Mesh {
    mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
    mesh
}

fn join(parts: Vec<(Mesh, Transform)>) -> Mesh {
    let mut parts = parts.into_iter();
    let (first, transform) = parts.next().unwrap();
    let mut shape = clean(first).transformed_by(transform);
    for (mesh, transform) in parts {
        shape.merge(&clean(mesh).transformed_by(transform)).unwrap();
    }
    shape
}

#[derive(Component)]
pub(crate) struct Propeller {
    phase: f32,
}

#[derive(Component)]
pub(crate) struct Cargo;

impl Effects {
    pub(super) fn decorate_drone(&self, commands: &mut Commands, carrier: Entity) {
        let visual = &self.shots[role_index(Role::Rook)];
        for (index, motor) in motors().into_iter().enumerate() {
            shot_layer(
                commands,
                carrier,
                &self.drone.cage,
                &visual.body,
                Transform::from_translation(motor),
            );
            commands.spawn((
                Mesh3d(self.drone.propeller.clone()),
                MeshMaterial3d(visual.accent.clone()),
                Transform::from_translation(motor + Vec3::Y * 0.012),
                NotShadowCaster,
                Propeller {
                    phase: index as f32 * 1.4,
                },
                ChildOf(carrier),
            ));
        }
        for side in [-1.0, 1.0] {
            shot_layer(
                commands,
                carrier,
                &self.drone.light,
                &visual.accent,
                Transform::from_xyz(side * 0.061, 0.003, -0.080),
            );
        }
        commands.spawn((
            Mesh3d(self.drone.grenade.clone()),
            MeshMaterial3d(self.drone.grenade_surface.clone()),
            Transform::from_xyz(0.0, -0.137, 0.0),
            Visibility::Inherited,
            NotShadowCaster,
            Cargo,
            ChildOf(carrier),
        ));
    }

    pub(crate) fn advance_drone(&self, commands: &mut Commands, shot: &mut Projectile) -> bool {
        let Flight::RookDrone { released } = shot.flight else {
            return false;
        };
        if !released && shot.elapsed >= RELEASE {
            let hover = position(shot, RELEASE) - Vec3::Y * 0.137;
            let grenade = Projectile {
                start: hover,
                end: shot.end,
                elapsed: shot.elapsed - RELEASE,
                duration: GRENADE_SECONDS,
                role: Role::Rook,
                index: 1,
                sideways: shot.sideways,
                lane: 0.0,
                flight: Flight::RookGrenade,
            };
            let entity = self.spawn_shot(commands, grenade, Vec3::ONE);
            commands.entity(entity).insert((
                Mesh3d(self.drone.grenade.clone()),
                MeshMaterial3d(self.drone.grenade_surface.clone()),
            ));
            shot.flight = Flight::RookDrone { released: true };
        }
        shot.elapsed >= shot.duration
    }
}

pub(crate) fn hatch_fraction(elapsed: f32) -> f32 {
    let open = ease(elapsed / 0.23);
    let close = ease((elapsed - LAUNCH_DELAY - FLIGHT_SECONDS) / 0.20);
    open * (1.0 - close)
}

pub(super) fn position(shot: &Projectile, elapsed: f32) -> Vec3 {
    let roof = shot.start + Vec3::Y * 0.72;
    let hover = shot.end + Vec3::Y * 1.20;
    if elapsed <= TAKEOFF_END {
        shot.start.lerp(roof, ease(elapsed / TAKEOFF_END))
    } else if elapsed < OUTBOUND_END {
        let t = ((elapsed - TAKEOFF_END) / (OUTBOUND_END - TAKEOFF_END)).clamp(0.0, 1.0);
        roof.lerp(hover, ease(t)) + Vec3::Y * 0.12 * (PI * t).sin()
    } else if elapsed < RETURN_START {
        hover
    } else if elapsed < RETURN_END {
        let t = ((elapsed - RETURN_START) / (RETURN_END - RETURN_START)).clamp(0.0, 1.0);
        hover.lerp(roof, ease(t)) + Vec3::Y * 0.10 * (PI * t).sin()
    } else {
        roof.lerp(
            shot.start,
            ease((elapsed - RETURN_END) / (FLIGHT_SECONDS - RETURN_END)),
        )
    }
}

pub(super) fn rotation(shot: &Projectile, t: f32) -> Quat {
    let direction = (shot.end - shot.start).with_y(0.0).normalize();
    let yaw = Quat::from_rotation_arc(Vec3::NEG_Z, direction);
    let elapsed = t * shot.duration;
    let tilt = if (TAKEOFF_END..OUTBOUND_END).contains(&elapsed) {
        -0.13 * (PI * (elapsed - TAKEOFF_END) / (OUTBOUND_END - TAKEOFF_END)).sin()
    } else if (RETURN_START..RETURN_END).contains(&elapsed) {
        0.12 * (PI * (elapsed - RETURN_START) / (RETURN_END - RETURN_START)).sin()
    } else {
        0.0
    };
    yaw * Quat::from_rotation_x(tilt)
}

pub(crate) fn animate_drones(
    handles: Option<Res<SceneHandles>>,
    shots: Query<&Projectile>,
    mut rotors: Query<(&ChildOf, &Propeller, &mut Transform)>,
    mut cargo: Query<(&ChildOf, &mut Visibility), With<Cargo>>,
) {
    if handles.is_none() {
        return;
    }
    for (parent, rotor, mut transform) in &mut rotors {
        let Ok(shot) = shots.get(parent.parent()) else {
            continue;
        };
        transform.rotation =
            Quat::from_rotation_y(rotor.phase + shot.elapsed.max(0.0) * TAU * 35.0);
    }
    for (parent, mut visible) in &mut cargo {
        let Ok(shot) = shots.get(parent.parent()) else {
            continue;
        };
        *visible = if shot.elapsed < RELEASE {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
