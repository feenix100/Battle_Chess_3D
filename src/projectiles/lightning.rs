//! The bishop's ball lightning. The orb follows the projectile path while
//! branching discharges regenerate in a fixed downward frame of reference.

use std::f32::consts::TAU;

use bevy::{light::NotShadowCaster, prelude::*};

use super::{Projectile, connect, surface};
use crate::{MainCamera, SceneHandles};

const PATTERN_COUNT: usize = 23;
const FLASHES_PER_SECOND: f32 = 26.0;

#[derive(Default)]
pub(super) struct LightningMeshes {
    halo: Handle<Mesh>,
    patterns: [Handle<Mesh>; PATTERN_COUNT],
}

impl LightningMeshes {
    pub(super) fn new(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            halo: meshes.add(halo_mesh()),
            patterns: std::array::from_fn(|index| meshes.add(discharge_mesh(index))),
        }
    }

    pub(super) fn spawn(
        &self,
        commands: &mut Commands,
        orb: Entity,
        glow: &Handle<StandardMaterial>,
    ) {
        let offset = (orb.to_bits() % PATTERN_COUNT as u64) as usize;
        commands.spawn((
            Mesh3d(self.halo.clone()),
            MeshMaterial3d(glow.clone()),
            Transform::IDENTITY,
            NotShadowCaster,
            OrbHalo,
            ChildOf(orb),
        ));
        commands.spawn((
            Mesh3d(self.patterns[offset].clone()),
            MeshMaterial3d(glow.clone()),
            Transform::IDENTITY,
            NotShadowCaster,
            Discharge { offset },
            ChildOf(orb),
        ));
    }
}

#[derive(Component)]
pub(crate) struct Discharge {
    offset: usize,
}

#[derive(Component)]
pub(crate) struct OrbHalo;

pub(crate) fn animate_lightning(
    handles: Option<Res<SceneHandles>>,
    camera: Query<&Transform, (With<MainCamera>, Without<OrbHalo>)>,
    shots: Query<&Projectile>,
    mut sparks: Query<(&ChildOf, &Discharge, &mut Mesh3d)>,
    mut haloes: Query<(&ChildOf, &mut Transform), With<OrbHalo>>,
) {
    let Some(handles) = handles else {
        return;
    };
    for (parent, discharge, mut mesh) in &mut sparks {
        let Ok(shot) = shots.get(parent.parent()) else {
            continue;
        };
        let frame = (shot.elapsed.max(0.0) * FLASHES_PER_SECOND) as usize;
        let pattern = &handles.fx.lightning.patterns[(frame + discharge.offset) % PATTERN_COUNT];
        if &mesh.0 != pattern {
            mesh.0 = pattern.clone();
        }
    }
    if let Some(camera) = camera.iter().next() {
        for (parent, mut halo) in &mut haloes {
            let Ok(shot) = shots.get(parent.parent()) else {
                continue;
            };
            halo.rotation = camera.rotation;
            halo.translation = camera.rotation * Vec3::Z * 0.112;
            halo.scale = Vec3::splat(1.0 + 0.09 * (shot.elapsed * 31.0 + shot.index as f32).sin());
        }
    }
}

fn halo_mesh() -> Mesh {
    const SIDES: u32 = 40;
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let mut colors = vec![[5.8, 5.3, 7.8, 1.0]];
    let mut indices = Vec::new();
    for (ring, (radius, color)) in [
        (0.036, [4.9, 4.3, 7.1, 1.0]),
        (0.075, [2.65, 2.05, 5.2, 0.99]),
        (0.110, [1.15, 0.58, 3.45, 0.92]),
        (0.145, [0.48, 0.14, 2.05, 0.63]),
        (0.207, [0.29, 0.05, 1.21, 0.29]),
        (0.335, [0.17, 0.024, 0.61, 0.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let start = 1 + ring as u32 * SIDES;
        for side in 0..SIDES {
            let (sin, cos) = (TAU * side as f32 / SIDES as f32).sin_cos();
            positions.push([cos * radius, sin * radius, 0.0]);
            colors.push(color);
            let a = start + side;
            let b = start + (side + 1) % SIDES;
            if ring == 0 {
                indices.extend([0, a, b]);
            } else {
                indices.extend([a - SIDES, a, b, a - SIDES, b, b - SIDES]);
            }
        }
    }
    surface(positions, indices).with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
}

struct Pattern {
    state: u32,
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

fn discharge_mesh(index: usize) -> Mesh {
    let mut pattern = Pattern {
        state: (index as u32 + 1).wrapping_mul(0x9e37_79b9),
        positions: Vec::new(),
        colors: Vec::new(),
        indices: Vec::new(),
    };
    let start_angle = pattern.pick(0.0, TAU);
    let branch_count = if pattern.pick(0.0, 1.0) < 0.45 { 4 } else { 5 };
    for i in 0..branch_count {
        let angle =
            start_angle + TAU * (i as f32 + pattern.pick(-0.22, 0.22)) / branch_count as f32;
        let outward = Vec3::new(angle.cos(), 0.0, angle.sin());
        let sideways = outward.cross(Vec3::Y);
        let radius = pattern.pick(0.13, 0.47);
        let drop = pattern.pick(if i == 0 { 0.37 } else { 0.22 }, 0.54);
        let path = std::array::from_fn::<_, 6, _>(|point| {
            let t = point as f32 / 5.0;
            let jitter = if (1..5).contains(&point) {
                pattern.pick(0.025, 0.086) * if point % 2 == 0 { -1.0 } else { 1.0 }
            } else {
                0.0
            };
            outward * (radius * t) + sideways * jitter - Vec3::Y * (drop * t.powf(1.22))
        });
        pattern.trace(&path, 1.0);

        if i % 2 == 0 {
            let split = 2 + usize::from(pattern.pick(0.0, 1.0) > 0.55);
            let side = sideways * if i % 4 == 0 { -1.0 } else { 1.0 };
            let reach = pattern.pick(0.10, 0.21);
            let fork = [
                path[split],
                path[split] + side * (reach * 0.37) + outward * 0.068 - Vec3::Y * 0.027,
                path[split] + side * (reach * 0.71) + outward * 0.044 - Vec3::Y * 0.089,
                path[split] + side * reach + outward * 0.095 - Vec3::Y * 0.148,
            ];
            pattern.trace(&fork, 0.61);
        }
    }
    surface(pattern.positions, pattern.indices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, pattern.colors)
}

impl Pattern {
    fn pick(&mut self, min: f32, max: f32) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        let unit = (self.state >> 8) as f32 / 16_777_216.0;
        min.lerp(max, unit)
    }

    fn trace(&mut self, path: &[Vec3], strength: f32) {
        self.stroke(path, strength * 2.5, [0.35, 0.08, 1.85, 0.40]);
        self.stroke(path, strength, [2.35, 1.85, 4.4, 0.99]);
    }

    fn stroke(&mut self, path: &[Vec3], strength: f32, color: [f32; 4]) {
        const SIDES: u32 = 5;
        let first = self.positions.len() as u32;
        let last = path.len() - 1;
        for (i, &point) in path.iter().enumerate() {
            let t = i as f32 / last as f32;
            let axis = (path[(i + 1).min(last)] - path[i.saturating_sub(1)]).normalize();
            let side = axis.cross(Vec3::X).normalize();
            let normal = axis.cross(side);
            let radius = (0.001 + 0.011 * (1.0 - t)) * strength;
            let [red, green, blue, alpha] = color;
            for face in 0..SIDES {
                let (sin, cos) = (TAU * face as f32 / SIDES as f32).sin_cos();
                self.positions
                    .push((point + radius * (side * cos + normal * sin)).to_array());
                self.colors
                    .push([red, green, blue, alpha * (1.0 - 0.62 * t)]);
            }
        }
        for i in 0..last as u32 {
            let ring = first + i * SIDES;
            for face in 0..SIDES {
                let a = ring + face;
                let b = ring + (face + 1) % SIDES;
                connect(&mut self.indices, a, b, b + SIDES, a + SIDES);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn lightning_patterns_branch_downward_at_varying_radii() {
        let radii: Vec<_> = (0..PATTERN_COUNT)
            .map(|index| {
                let mesh = discharge_mesh(index);
                let Some(VertexAttributeValues::Float32x3(positions)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    panic!("missing discharge geometry");
                };
                assert!(positions.iter().all(|point| point[1] <= 0.04));
                assert!(positions.iter().any(|point| point[1] < -0.32));
                positions
                    .iter()
                    .map(|[x, _, z]| x.hypot(*z))
                    .fold(0.0, f32::max)
            })
            .collect();
        let range = radii.iter().copied().fold(0.0, f32::max)
            - radii.iter().copied().fold(f32::INFINITY, f32::min);
        assert!(range > 0.08);
    }
}
