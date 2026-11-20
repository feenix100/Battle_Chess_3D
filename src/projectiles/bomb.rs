//! The king calls a vertical bomb drop and a short, board-scale mushroom cloud.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{light::NotShadowCaster, mesh::VertexAttributeValues, prelude::*};
use shakmaty::Role;

use super::{Effects, lathe, ribbon, role_index, shot_layer, surface};

const BLAST_SECONDS: f32 = 0.96;
const PALETTE_STEPS: usize = 21;

#[derive(Default)]
pub(super) struct BombMeshes {
    fins: Handle<Mesh>,
    band: Handle<Mesh>,
    cap: Handle<Mesh>,
    stem: Handle<Mesh>,
    flash: Handle<Mesh>,
    wave: Handle<Mesh>,
    flash_surface: Handle<StandardMaterial>,
    smoke: [Handle<StandardMaterial>; PALETTE_STEPS],
    wave_surfaces: [Handle<StandardMaterial>; PALETTE_STEPS],
}

pub(super) fn body() -> Mesh {
    lathe(
        28,
        &[
            (-0.375, 0.079),
            (-0.302, 0.087),
            (-0.223, 0.116),
            (-0.160, 0.172),
            (-0.075, 0.208),
            (0.048, 0.224),
            (0.134, 0.215),
            (0.217, 0.185),
            (0.293, 0.133),
            (0.346, 0.066),
            (0.368, 0.023),
        ],
    )
}

pub(super) fn metal() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(72, 75, 52),
        metallic: 0.42,
        perceptual_roughness: 0.35,
        ..default()
    }
}

impl BombMeshes {
    pub(super) fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let smoke = std::array::from_fn(|index| {
            let t = index as f32 / (PALETTE_STEPS - 1) as f32;
            let heat = (1.0 - (t / 0.78)).max(0.0).powi(2);
            let tint = Vec3::new(0.23, 0.19, 0.20).lerp(Vec3::new(5.8, 1.43, 0.19), heat);
            materials.add(luminous(tint, 0.96 * (1.0 - ease((t - 0.60) / 0.40))))
        });
        let wave_surfaces = std::array::from_fn(|index| {
            let t = index as f32 / (PALETTE_STEPS - 1) as f32;
            materials.add(luminous(
                Vec3::new(4.8, 2.1, 0.55),
                0.81 * (1.0 - t).powf(1.3),
            ))
        });
        Self {
            fins: meshes.add(fins_mesh()),
            band: meshes.add(lathe(28, &[(0.098, 0.226), (0.132, 0.221)])),
            cap: meshes.add(cloud(&[
                (-0.23, 0.112),
                (-0.175, 0.30),
                (-0.095, 0.49),
                (0.002, 0.547),
                (0.095, 0.514),
                (0.19, 0.43),
                (0.248, 0.30),
                (0.279, 0.147),
                (0.287, 0.052),
            ])),
            stem: meshes.add(cloud(&[
                (0.0, 0.220),
                (0.13, 0.223),
                (0.27, 0.156),
                (0.43, 0.161),
                (0.57, 0.141),
                (0.75, 0.193),
                (0.92, 0.238),
                (1.0, 0.210),
            ])),
            flash: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
            wave: meshes.add(ribbon(72, 0.0, TAU, |_| (0.88, 1.0, 0.0))),
            flash_surface: materials.add(luminous(Vec3::new(10.0, 6.6, 2.2), 0.79)),
            smoke,
            wave_surfaces,
        }
    }
}

impl Effects {
    pub(super) fn decorate_bomb(&self, commands: &mut Commands, entity: Entity) {
        let visual = &self.shots[role_index(Role::King)];
        shot_layer(
            commands,
            entity,
            &self.bomb.fins,
            &visual.body,
            Transform::IDENTITY,
        );
        shot_layer(
            commands,
            entity,
            &self.bomb.band,
            &visual.accent,
            Transform::IDENTITY,
        );
    }

    pub(crate) fn spawn_bomb_blast(&self, commands: &mut Commands, impact: Vec3) {
        let root = commands
            .spawn((
                BombBlast { elapsed: 0.0 },
                Transform::from_translation(impact - Vec3::Y * 0.49),
                Visibility::Inherited,
            ))
            .id();
        for part in [
            BlastPart::Stem,
            BlastPart::Cap,
            BlastPart::Dust,
            BlastPart::Wave,
            BlastPart::Flash,
        ] {
            let (mesh, surface) = match part {
                BlastPart::Stem => (&self.bomb.stem, &self.bomb.smoke[0]),
                BlastPart::Cap | BlastPart::Dust => (&self.bomb.cap, &self.bomb.smoke[0]),
                BlastPart::Wave => (&self.bomb.wave, &self.bomb.wave_surfaces[0]),
                BlastPart::Flash => (&self.bomb.flash, &self.bomb.flash_surface),
            };
            let (transform, visibility) = part.pose(0.0);
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(surface.clone()),
                transform,
                visibility,
                NotShadowCaster,
                part,
                ChildOf(root),
            ));
        }
    }
}

#[derive(Component)]
pub(crate) struct BombBlast {
    elapsed: f32,
}

impl BombBlast {
    pub(crate) fn tick(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        self.elapsed < BLAST_SECONDS
    }
}

#[derive(Clone, Copy, Component)]
pub(crate) enum BlastPart {
    Flash,
    Wave,
    Dust,
    Stem,
    Cap,
}

impl BlastPart {
    fn pose(self, phase: f32) -> (Transform, Visibility) {
        let grow = ease(phase / 0.67);
        let rise = ease(phase / 0.83);
        let up = Quat::from_rotation_y(phase * 0.94) * Quat::from_rotation_x(-FRAC_PI_2);
        let height = 0.24 + 1.24 * rise;
        let (transform, visible) = match self {
            Self::Flash => {
                let pulse = ease(phase / 0.13) * (1.0 - ease((phase - 0.095) / 0.10));
                (
                    Transform::from_xyz(0.0, 0.24, 0.0).with_scale(
                        Vec3::new(0.17, 0.14, 0.17) + Vec3::new(0.81, 0.48, 0.81) * pulse,
                    ),
                    phase < 0.19,
                )
            }
            Self::Wave => {
                let radius = 0.16 + 1.23 * ease(phase / 0.48);
                (
                    Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2))
                        .with_scale(Vec3::new(radius, radius, 1.0)),
                    phase < 0.48,
                )
            }
            Self::Dust => (
                Transform::from_xyz(0.0, 0.083 + 0.12 * grow, 0.0)
                    .with_rotation(up)
                    .with_scale(Vec3::new(
                        0.44 + grow * 1.88,
                        0.44 + grow * 1.88,
                        0.40 + grow * 0.19,
                    )),
                true,
            ),
            Self::Stem => (
                Transform::from_xyz(0.0, 0.035, 0.0)
                    .with_rotation(up)
                    .with_scale(Vec3::new(0.55 + grow * 0.44, 0.55 + grow * 0.44, height)),
                true,
            ),
            Self::Cap => (
                Transform::from_xyz(0.0, height, 0.0)
                    .with_rotation(up)
                    .with_scale(Vec3::new(
                        0.36 + 1.20 * grow,
                        0.36 + 1.20 * grow,
                        0.34 + 1.10 * grow,
                    )),
                true,
            ),
        };
        (
            transform,
            if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        )
    }
}

pub(crate) fn animate_bomb_blast(
    handles: Option<Res<crate::SceneHandles>>,
    blasts: Query<&BombBlast>,
    mut parts: Query<(
        &ChildOf,
        &BlastPart,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(handles) = handles else {
        return;
    };
    for (parent, part, mut transform, mut visibility, mut surface) in &mut parts {
        let Ok(blast) = blasts.get(parent.parent()) else {
            continue;
        };
        let phase = (blast.elapsed / BLAST_SECONDS).clamp(0.0, 1.0);
        (*transform, *visibility) = part.pose(phase);
        let material = match part {
            BlastPart::Flash => continue,
            BlastPart::Wave => &handles.fx.bomb.wave_surfaces[palette_index(phase / 0.48)],
            _ => &handles.fx.bomb.smoke[palette_index(phase)],
        };
        if &surface.0 != material {
            surface.0 = material.clone();
        }
    }
}

fn palette_index(phase: f32) -> usize {
    (phase.clamp(0.0, 1.0) * (PALETTE_STEPS - 1) as f32).round() as usize
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn luminous(rgb: Vec3, alpha: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::linear_rgba(rgb.x, rgb.y, rgb.z, alpha),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

fn cloud(profile: &[(f32, f32)]) -> Mesh {
    let mut mesh = lathe(48, profile);
    let lowest = profile[0].0;
    let span = profile.last().unwrap().0 - lowest;
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    else {
        unreachable!("lathe supplies its vertices");
    };
    let mut colors = Vec::with_capacity(positions.len());
    for [x, y, z] in positions {
        let angle = y.atan2(*x);
        let h = (*z - lowest) / span;
        let curls = (angle * 6.0 + h * 7.8).cos() * 0.103 + (angle * 11.0 - h * 8.0).sin() * 0.042;
        *x *= 1.0 + curls;
        *y *= 1.0 + curls;
        *z += 0.041 * (angle * 4.0 + h * 5.0).sin() * h * (1.0 - h);
        let shade = (0.69 + 0.20 * h + 0.094 * angle.cos() + curls).clamp(0.43, 1.0);
        colors.push([
            shade,
            shade * (0.80 + 0.19 * h),
            shade * (0.73 + 0.25 * h),
            1.0,
        ]);
    }
    mesh.compute_normals();
    mesh.with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
}

fn fins_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let outline = [
        (0.058, -0.392),
        (0.170, -0.392),
        (0.170, -0.287),
        (0.102, -0.215),
    ];
    for fin in 0..4 {
        let (sin, cos) = (TAU * fin as f32 / 4.0).sin_cos();
        let first = positions.len() as u32;
        for side in [-0.011, 0.011] {
            for (radius, z) in outline {
                positions.push([radius * cos - side * sin, radius * sin + side * cos, z]);
            }
        }
        indices.extend([first, first + 2, first + 1, first, first + 3, first + 2]);
        indices.extend([
            first + 4,
            first + 5,
            first + 6,
            first + 4,
            first + 6,
            first + 7,
        ]);
        for i in 0..4 {
            let a = first + i;
            let b = first + (i + 1) % 4;
            super::connect(&mut indices, a, b, b + 4, a + 4);
        }
    }
    surface(positions, indices)
}
