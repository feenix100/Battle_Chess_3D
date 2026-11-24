use std::f32::consts::{PI, TAU};

use bevy::{
    asset::RenderAssetUsages,
    light::NotShadowCaster,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use shakmaty::Role;

use super::{CaptureFxRequest, square_world};

mod artillery;
mod bomb;
mod cannon;
mod drone;
mod lightning;
mod lowrider;
mod molotov;
mod shotgun;
mod smg;
pub(crate) use smg::{
    INTERVAL as QUEEN_SMG_INTERVAL, LAUNCH as QUEEN_SMG_LAUNCH, ROUNDS as QUEEN_SMG_ROUNDS,
    SmgEffect, aim_pitch as queen_smg_aim, animate_smg, flight_seconds as queen_smg_flight,
    spread as queen_smg_spread,
};
mod street;
mod taser;
use artillery::ArtilleryMeshes;
pub(crate) use artillery::animate_artillery_smoke;
use bomb::BombMeshes;
pub(super) use bomb::{BombBlast, animate_bomb_blast};
use cannon::CannonMeshes;
use drone::DroneMeshes;
pub(crate) use drone::{
    SEQUENCE_SECONDS as ROOK_SEQUENCE_SECONDS, animate_drones,
    hatch_fraction as rook_hatch_fraction,
};
use lightning::LightningMeshes;
pub(super) use lightning::animate_lightning;
pub(crate) use lowrider::{animate_lowrider_drones, trunk_fraction as lowrider_trunk_fraction};
#[cfg(test)]
pub(crate) use molotov::MolotovPart;
pub(crate) use molotov::{MolotovFire, animate_molotov, hatch_fraction as mansion_hatch_fraction};
pub(crate) use shotgun::{
    LAUNCH as KING_SHOTGUN_LAUNCH, ShotgunCloud, aim_pitch as king_shotgun_aim, animate_shotgun,
};
#[cfg(test)]
pub(crate) use taser::TaserPart;
pub(crate) use taser::{TaserCircuit, animate_taser, flight_seconds as taser_flight_seconds};

pub(super) const ATTACK_ROLES: [Role; 6] = [
    Role::Pawn,
    Role::Knight,
    Role::Bishop,
    Role::Rook,
    Role::Queen,
    Role::King,
];

fn role_index(role: Role) -> usize {
    match role {
        Role::Pawn => 0,
        Role::Knight => 1,
        Role::Bishop => 2,
        Role::Rook => 3,
        Role::Queen => 4,
        Role::King => 5,
    }
}

pub(super) struct Profile {
    pub count: usize,
    pub delay: f32,
    pub flight: f32,
    pub interval: f32,
    arc: f32,
    crest: f32,
    // X is the attacker's right, Y is up, and Z is forward toward the target.
    launch: Vec3,
    color: u32,
    accent: u32,
}

pub(super) fn profile(role: Role) -> Profile {
    let (count, delay, flight, interval, arc, crest, launch, color, accent) = match role {
        Role::Pawn => (
            3,
            0.12,
            0.33,
            0.126,
            0.19,
            -0.25,
            (0.15, 0.62, 0.40),
            0x08d7da,
            0xc4ffff,
        ),
        Role::Knight => (
            1,
            0.16,
            0.54,
            0.0,
            0.43,
            -0.10,
            (0.0, 0.64, 0.52),
            0x649dff,
            0xcfebff,
        ),
        Role::Bishop => (
            3,
            0.15,
            0.53,
            0.09,
            0.54,
            0.08,
            (0.13, 0.81, 0.34),
            0xac56ef,
            0xf1d8ff,
        ),
        Role::Rook => (
            1,
            drone::LAUNCH_DELAY,
            drone::FLIGHT_SECONDS,
            0.0,
            0.0,
            0.0,
            (0.0, 0.765 * crate::characters::BOARD_SCALE, 0.0),
            0xc54229,
            0xffba52,
        ),
        Role::Queen => (
            8,
            0.18,
            0.70,
            0.09,
            1.08,
            0.27,
            (0.13, 0.87, 0.37),
            0xff862e,
            0xf5c06c,
        ),
        Role::King => (
            1,
            0.24,
            0.64,
            0.0,
            0.0,
            0.0,
            (0.13, 0.91, 0.36),
            0xfdb842,
            0xffefb3,
        ),
    };
    Profile {
        count,
        delay,
        flight,
        interval,
        arc,
        crest,
        color,
        accent,
        launch: Vec3::from(launch),
    }
}

pub(crate) fn street_profile() -> Profile {
    Profile {
        count: 3,
        delay: 0.30,
        flight: 0.20,
        interval: 0.115,
        arc: 0.0,
        crest: 0.0,
        launch: Vec3::ZERO,
        color: 0xffc66b,
        accent: 0xffedb3,
    }
}

#[derive(Default)]
struct ShotVisual {
    shape: Handle<Mesh>,
    body: Handle<StandardMaterial>,
    accent: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
    impact: Handle<StandardMaterial>,
}

#[derive(Default)]
pub(super) struct Effects {
    shots: [ShotVisual; 6],
    scan_bands: Handle<Mesh>,
    trail: Handle<Mesh>,
    artillery: ArtilleryMeshes,
    bomb: BombMeshes,
    cannon: CannonMeshes,
    drone: DroneMeshes,
    lightning: LightningMeshes,
    street: street::StreetMeshes,
    lowrider: lowrider::LowriderFx,
    molotov: molotov::MolotovFx,
    taser: taser::TaserFx,
    shotgun: shotgun::ShotgunFx,
    smg: smg::SmgFx,
}

impl Effects {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let shots = ATTACK_ROLES.map(|role| {
            let data = profile(role);
            let shape = match role {
                Role::Pawn => lathe(
                    20,
                    &[
                        (-0.215, 0.094),
                        (-0.192, 0.119),
                        (0.012, 0.119),
                        (0.067, 0.110),
                        (0.146, 0.083),
                        (0.219, 0.037),
                        (0.242, 0.010),
                    ],
                ),
                Role::Knight => Sphere::new(cannon::RADIUS).mesh().ico(3).unwrap(),
                Role::Bishop => Sphere::new(0.108).mesh().ico(3).unwrap(),
                Role::Rook => drone::body(),
                Role::Queen => artillery::shell(),
                Role::King => bomb::body(),
            };
            let impact = materials.add(material(data.color, 1.0));
            let accent = materials.add(material(data.accent, 1.0));
            let body = match role {
                Role::Pawn => materials.add(material(data.color, 0.37)),
                Role::Knight => materials.add(cannon::iron()),
                Role::Bishop => accent.clone(),
                Role::Rook => materials.add(drone::metal()),
                Role::Queen => materials.add(artillery::steel()),
                Role::King => materials.add(bomb::metal()),
            };
            ShotVisual {
                shape: meshes.add(shape),
                body,
                accent,
                glow: if role == Role::Bishop {
                    materials.add(material(0xffffff, 0.99))
                } else if role == Role::Queen {
                    materials.add(material(0xd9d4ca, 0.23))
                } else {
                    materials.add(material(data.color, 0.27))
                },
                impact,
            }
        });
        let scan_bands = meshes.add(bullet_bands());
        let trail = meshes.add(lathe(
            10,
            &[
                (-0.56, 0.004),
                (-0.33, 0.029),
                (-0.15, 0.052),
                (-0.04, 0.032),
            ],
        ));
        Self {
            shots,
            scan_bands,
            trail,
            artillery: ArtilleryMeshes::new(meshes, materials),
            bomb: BombMeshes::new(meshes, materials),
            cannon: CannonMeshes::new(meshes),
            drone: DroneMeshes::new(meshes, materials),
            lightning: LightningMeshes::new(meshes),
            street: street::StreetMeshes::new(meshes, materials),
            lowrider: lowrider::LowriderFx::new(meshes, materials),
            molotov: molotov::MolotovFx::new(meshes, materials),
            taser: taser::TaserFx::new(meshes, materials),
            shotgun: shotgun::ShotgunFx::new(meshes, materials),
            smg: smg::SmgFx::new(meshes, materials),
        }
    }

    pub fn impact(&self, role: Role) -> Handle<StandardMaterial> {
        self.shots[role_index(role)].impact.clone()
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        request: CaptureFxRequest,
        style: crate::PieceStyle,
        target_role: Role,
    ) {
        if request.role == Role::Rook && style == crate::PieceStyle::GtaStreet {
            self.spawn_mansion_molotov(commands, request);
            return;
        }
        let hit_height = if style == crate::PieceStyle::GtaStreet && target_role == Role::Knight {
            0.28
        } else {
            0.56
        };
        if request.role == Role::Knight && style == crate::PieceStyle::GtaStreet {
            self.spawn_lowrider_drone(commands, request, hit_height);
            return;
        }
        if request.role == Role::Bishop && style == crate::PieceStyle::GtaStreet {
            self.spawn_police_taser(commands, request, hit_height);
            return;
        }
        if request.role == Role::King && style == crate::PieceStyle::GtaStreet {
            self.spawn_king_shotgun(
                commands,
                request,
                if target_role == Role::Knight {
                    0.28
                } else {
                    0.74
                },
            );
            return;
        }
        if request.role == Role::Queen && style == crate::PieceStyle::GtaStreet {
            self.spawn_queen_smg(commands, request, hit_height);
            return;
        }
        let street = request.role == Role::Pawn && style == crate::PieceStyle::GtaStreet;
        let spec = if street {
            street_profile()
        } else {
            profile(request.role)
        };
        let from = square_world(request.from);
        let target = square_world(request.target);
        let forward = (target - from).normalize();
        let sideways = forward.cross(Vec3::Y);
        let launch =
            from + sideways * spec.launch.x + Vec3::Y * spec.launch.y + forward * spec.launch.z;
        let origin = if request.role == Role::King {
            target + Vec3::Y * 3.1
        } else {
            launch
        };
        let end = target + Vec3::Y * hit_height;
        let middle = (spec.count as f32 - 1.0) * 0.5;
        let scale = if street {
            Vec3::ONE
        } else if request.role == Role::Pawn {
            Vec3::new(1.0, 1.0, 0.82) * 0.60
        } else {
            Vec3::ONE
        };
        for index in 0..spec.count {
            let shot_origin = if street {
                let muzzle = crate::characters::street_pawn_muzzle(
                    spec.delay + index as f32 * spec.interval,
                );
                from + sideways * muzzle.x + Vec3::Y * muzzle.y - forward * muzzle.z
            } else {
                origin
            };
            let lane = (index as f32 - middle) / middle.max(0.5);
            let (endpoint, extra_flight) = if street {
                (end + sideways * lane * 0.014 + Vec3::Y * lane * 0.009, 0.0)
            } else if request.role == Role::Queen {
                let spread = artillery::impact_offset(index);
                (
                    end + sideways * spread.x + forward * spread.y,
                    artillery::flight_variation(index),
                )
            } else {
                (end, 0.0)
            };
            let shot = Projectile {
                start: shot_origin,
                end: endpoint,
                sideways,
                elapsed: -spec.delay - index as f32 * spec.interval,
                duration: spec.flight + extra_flight,
                role: request.role,
                index,
                lane,
                flight: if street {
                    Flight::StreetBullet
                } else {
                    match request.role {
                        Role::Knight => Flight::KnightShell,
                        Role::Rook => Flight::RookDrone { released: false },
                        Role::King => Flight::BombDrop,
                        _ => Flight::Arc,
                    }
                },
            };
            let visual = &self.shots[role_index(request.role)];
            let entity = self.spawn_shot(commands, shot, scale);

            if street {
                shot_layer(
                    commands,
                    entity,
                    &self.trail,
                    &self.street.visual.glow,
                    Transform::from_scale(Vec3::new(0.12, 0.12, 0.32)),
                );
                continue;
            }

            let mut layer =
                |mesh: &Handle<Mesh>, surface: &Handle<StandardMaterial>, transform: Transform| {
                    shot_layer(commands, entity, mesh, surface, transform);
                };
            let length = match request.role {
                Role::Pawn => {
                    layer(
                        &visual.shape,
                        &visual.accent,
                        Transform::from_scale(Vec3::new(0.34, 0.34, 0.86)),
                    );
                    layer(&self.scan_bands, &visual.accent, Transform::IDENTITY);
                    0.48
                }
                Role::Knight => {
                    self.decorate_cannon(commands, entity, 0.68);
                    continue;
                }
                Role::Bishop => {
                    self.lightning.spawn(commands, entity, &visual.glow);
                    continue;
                }
                Role::Rook => {
                    self.decorate_drone(commands, entity);
                    continue;
                }
                Role::Queen => {
                    self.decorate_artillery(commands, entity);
                    continue;
                }
                Role::King => {
                    self.decorate_bomb(commands, entity);
                    continue;
                }
            };
            layer(
                &self.trail,
                &visual.glow,
                Transform::from_scale(Vec3::new(1.0, 1.0, length)),
            );
        }
    }

    fn spawn_shot(&self, commands: &mut Commands, shot: Projectile, scale: Vec3) -> Entity {
        let visual = if shot.is_street_bullet() {
            &self.street.visual
        } else {
            &self.shots[role_index(shot.role)]
        };
        let progress = (shot.elapsed / shot.duration).clamp(0.0, 1.0);
        commands
            .spawn((
                Mesh3d(visual.shape.clone()),
                MeshMaterial3d(visual.body.clone()),
                Transform::from_translation(shot.position(progress))
                    .with_rotation(shot.rotation(progress))
                    .with_scale(scale),
                if shot.elapsed >= 0.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                NotShadowCaster,
                shot,
            ))
            .id()
    }
}

fn shot_layer(
    commands: &mut Commands,
    entity: Entity,
    mesh: &Handle<Mesh>,
    surface: &Handle<StandardMaterial>,
    transform: Transform,
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(surface.clone()),
        transform,
        NotShadowCaster,
        ChildOf(entity),
    ));
}

#[derive(Clone, Copy)]
enum Flight {
    Arc,
    StreetBullet,
    QueenBullet { eject: Vec3, from: Vec3 },
    TaserDart,
    KingPellet { eject: Vec3 },
    LowriderDrone,
    Molotov { arc: f32, last_smoke: f32 },
    BombDrop,
    KnightShell,
    KnightFragment { control: Vec3 },
    RookDrone { released: bool },
    RookGrenade,
}

fn material(hex: u32, alpha: f32) -> StandardMaterial {
    let channel = |shift: u32| ((hex >> shift) & 0xff) as f32 / 255.0;
    StandardMaterial {
        base_color: Color::srgba(channel(16), channel(8), channel(0), alpha),
        alpha_mode: if alpha < 1.0 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
        cull_mode: None,
        unlit: true,
        ..default()
    }
}

#[derive(Component)]
pub(super) struct Projectile {
    pub start: Vec3,
    pub end: Vec3,
    pub elapsed: f32,
    pub duration: f32,
    pub role: Role,
    pub index: usize,
    sideways: Vec3,
    lane: f32,
    flight: Flight,
}

impl Projectile {
    pub fn is_queen_bullet(&self) -> bool {
        matches!(self.flight, Flight::QueenBullet { .. })
    }
    pub fn is_king_pellet(&self) -> bool {
        matches!(self.flight, Flight::KingPellet { .. })
    }
    pub fn is_taser_dart(&self) -> bool {
        matches!(self.flight, Flight::TaserDart)
    }
    pub fn is_molotov(&self) -> bool {
        matches!(self.flight, Flight::Molotov { .. })
    }
    pub fn is_lowrider_drone(&self) -> bool {
        matches!(self.flight, Flight::LowriderDrone)
    }
    pub fn is_street_bullet(&self) -> bool {
        matches!(self.flight, Flight::StreetBullet)
    }

    pub fn position(&self, t: f32) -> Vec3 {
        let t = t.clamp(0.0, 1.0);
        if self.is_molotov() {
            return molotov::position(self, t);
        }
        if self.is_lowrider_drone() {
            return lowrider::position(self, t);
        }
        if self.is_street_bullet()
            || self.is_taser_dart()
            || self.is_king_pellet()
            || self.is_queen_bullet()
        {
            return self.start.lerp(self.end, t);
        }
        if matches!(self.flight, Flight::BombDrop) {
            return self.start.lerp(self.end, 0.14 * t + 0.86 * t * t);
        }
        if matches!(self.flight, Flight::RookDrone { .. }) {
            return drone::position(self, t * self.duration);
        }
        if matches!(self.flight, Flight::RookGrenade) {
            return self.start.lerp(self.end, 0.15 * t + 0.85 * t * t);
        }
        let lift = 4.0 * t * (1.0 - t);
        let line = self.start.lerp(self.end, t);
        if let Flight::KnightFragment { control } = self.flight {
            return line + lift * control;
        }
        let spec = profile(self.role);
        let rise = spec.arc * (1.0 + (2.0 * t - 1.0) * spec.crest);
        let phase = self.index as f32 * TAU / spec.count as f32;
        let (lateral, height) = match self.role {
            Role::Pawn => (self.lane * 0.068, self.lane * 0.034),
            Role::Knight => (0.0, 0.0),
            Role::Bishop => {
                let (swirl, rise) = (TAU * t + phase).sin_cos();
                (0.20 * swirl, 0.105 * rise)
            }
            Role::Rook => unreachable!("the rook uses a drone and a dropped grenade"),
            Role::Queen => (
                self.lane * 0.28 * (0.55 + 0.45 * t),
                (self.index % 3) as f32 * 0.095,
            ),
            Role::King => unreachable!("the king uses a vertical drop"),
        };
        line + lift * (Vec3::Y * (rise + height) + self.sideways * lateral)
    }

    pub fn rotation(&self, t: f32) -> Quat {
        if self.is_taser_dart() || self.is_king_pellet() || self.is_queen_bullet() {
            return Quat::from_rotation_arc(Vec3::Z, (self.end - self.start).normalize());
        }
        if self.is_molotov() {
            return molotov::rotation(self, t);
        }
        if self.is_lowrider_drone() {
            return lowrider::rotation(self, t);
        }
        if matches!(self.flight, Flight::RookDrone { .. }) {
            return drone::rotation(self, t);
        }
        let roll = match self.role {
            Role::Pawn => 0.1 + PI * t,
            Role::Knight => self.lane + TAU * 0.8 * t,
            Role::Bishop => return Quat::IDENTITY,
            Role::Rook => return Quat::from_euler(EulerRot::YXZ, t * TAU * 1.2, t * 2.7, 0.24),
            Role::Queen => 0.2 + self.lane * 0.30 + 4.0 * t,
            Role::King => {
                return Quat::from_rotation_arc(Vec3::Z, Vec3::NEG_Y)
                    * Quat::from_rotation_z(0.24 + 1.1 * t);
            }
        };
        let tangent = (self.position(t + 0.012) - self.position(t - 0.012)).normalize();
        let facing = Quat::from_rotation_arc(Vec3::Z, tangent);
        let roll = Quat::from_rotation_z(roll);
        facing * roll
    }
}

fn surface(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices))
    .with_computed_normals()
}

fn connect(indices: &mut Vec<u32>, a: u32, b: u32, c: u32, d: u32) {
    indices.extend([a, b, d, b, c, d]);
}

fn lathe(sides: u32, profile: &[(f32, f32)]) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for &(z, radius) in profile {
        for i in 0..sides {
            let (sin, cos) = (i as f32 * TAU / sides as f32).sin_cos();
            positions.push([cos * radius, sin * radius, z]);
        }
    }
    for section in 0..profile.len() as u32 - 1 {
        let lower = section * sides;
        let upper = lower + sides;
        for i in 0..sides {
            let next = (i + 1) % sides;
            connect(
                &mut indices,
                lower + i,
                lower + next,
                upper + next,
                upper + i,
            );
        }
    }
    let cap = positions.len() as u32;
    let top = cap - sides;
    positions.push([0.0, 0.0, profile[0].0]);
    positions.push([0.0, 0.0, profile.last().unwrap().0]);
    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.extend([cap, next, i, cap + 1, top + i, top + next]);
    }
    surface(positions, indices)
}

fn ribbon(steps: u32, start: f32, end: f32, widths: impl Fn(f32) -> (f32, f32, f32)) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let (sin, cos) = start.lerp(end, t).sin_cos();
        let (inner, outer, z) = widths(t);
        positions.extend([[cos * inner, sin * inner, z], [cos * outer, sin * outer, z]]);
    }
    for i in 0..steps {
        let inner = 2 * i;
        connect(&mut indices, inner, inner + 1, inner + 3, inner + 2);
    }
    surface(positions, indices)
}

fn bullet_bands() -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let sides = 24;
    for (z, radius, width) in [
        (-0.198, 0.121, 0.010),
        (-0.154, 0.121, 0.007),
        (-0.045, 0.121, 0.007),
        (0.048, 0.116, 0.008),
    ] {
        let first = positions.len() as u32;
        for i in 0..sides {
            let (sin, cos) = (i as f32 * TAU / sides as f32).sin_cos();
            positions.extend([
                [cos * radius, sin * radius, z],
                [cos * radius, sin * radius, z + width],
            ]);
        }
        for i in 0..sides {
            let a = first + 2 * i;
            let b = first + 2 * ((i + 1) % sides);
            connect(&mut indices, a, b, b + 1, a + 1);
        }
    }
    surface(positions, indices)
}
