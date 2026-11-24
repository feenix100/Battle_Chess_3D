//! An original miniature mansion with a hinged rooftop hatch and estate gates.
use super::{Builder, Joint, Rig, rotation, skin};
use crate::characters::animation::Pose;
use bevy::prelude::*;
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/mansion/assets.rs"
));
const BODY: Vec3 = Vec3::new(0.0, 0.43, 0.0);
const HATCH: Vec3 = Vec3::new(0.0, 0.887, 0.118);
const GATES: [Vec3; 2] = [
    Vec3::new(-0.268, 0.085, -0.405),
    Vec3::new(0.268, 0.085, -0.405),
];
const LAUNCH: Vec3 = Vec3::new(0.0, 1.035, -0.025);

struct Layer {
    part: &'static str,
    mesh: Handle<Mesh>,
    surface: Option<Handle<StandardMaterial>>,
}
fn layers(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    house: bool,
) -> Vec<Layer> {
    let mut surfaces = std::collections::HashMap::new();
    PARTS
        .iter()
        .filter(|(part, _, _)| {
            (matches!(*part, "body" | "hatch" | "gate-0" | "gate-1") || part.starts_with("leg-"))
                == house
        })
        .map(|&(part, surface, bytes)| {
            let material = if surface == "paint" {
                None
            } else {
                Some(
                    surfaces
                        .entry(surface)
                        .or_insert_with(|| {
                            let (color, metallic, roughness, alpha) = match surface {
                                "stone" => (0xc7b99a, 0.02, 0.77, 1.0),
                                "ivory" => (0xf1e6c8, 0.02, 0.58, 1.0),
                                "tile" => (0xa65332, 0.03, 0.75, 1.0),
                                "glass" => (0x18394b, 0.30, 0.21, 1.0),
                                "iron" => (0x1c252c, 0.58, 0.37, 1.0),
                                "gold" => (0xe8b74d, 0.78, 0.27, 1.0),
                                "wood" => (0x643720, 0.03, 0.67, 1.0),
                                "green" => (0x426a35, 0.08, 0.52, 1.0),
                                "bottle-glass" => (0x3f9462, 0.10, 0.20, 0.70),
                                "label" => (0xe4d3a0, 0.01, 0.79, 1.0),
                                "cloth" => (0xa26332, 0.0, 0.90, 1.0),
                                _ => unreachable!("unknown mansion material"),
                            };
                            materials.add(StandardMaterial {
                                base_color: Color::srgba_u8(
                                    (color >> 16) as u8,
                                    (color >> 8) as u8,
                                    color as u8,
                                    (alpha * 255.0) as u8,
                                ),
                                metallic,
                                perceptual_roughness: roughness,
                                alpha_mode: if alpha < 1.0 {
                                    AlphaMode::Blend
                                } else {
                                    AlphaMode::Opaque
                                },
                                ..default()
                            })
                        })
                        .clone(),
                )
            };
            Layer {
                part,
                mesh: meshes.add(skin::unpack(bytes).0),
                surface: material,
            }
        })
        .collect()
}
fn spawn_layers(
    commands: &mut Commands,
    layers: &[Layer],
    part: &str,
    parent: Entity,
    team: &Handle<StandardMaterial>,
) {
    for layer in layers.iter().filter(|l| l.part == part) {
        commands.spawn((
            Mesh3d(layer.mesh.clone()),
            MeshMaterial3d(layer.surface.as_ref().unwrap_or(team).clone()),
            Transform::IDENTITY,
            ChildOf(parent),
        ));
    }
}
#[derive(Default)]
pub(super) struct MansionAssets {
    layers: Vec<Layer>,
}
impl MansionAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            layers: layers(meshes, materials, true),
        }
    }
}
pub(super) struct MansionRig {
    hatch: Entity,
    gates: [Entity; 2],
    legs: [[Entity; 3]; 2],
}
pub(super) fn spawn(b: &mut Builder, root: Entity) -> Rig {
    let body = b.joint(root, BODY);
    spawn_layers(b.commands, &b.assets.mansion.layers, "body", body, &b.team);
    let hatch = b.joint(body, HATCH - BODY);
    spawn_layers(
        b.commands,
        &b.assets.mansion.layers,
        "hatch",
        hatch,
        &b.team,
    );
    let gates = std::array::from_fn(|i| {
        let hinge = b.joint(body, GATES[i] - BODY);
        spawn_layers(
            b.commands,
            &b.assets.mansion.layers,
            &format!("gate-{i}"),
            hinge,
            &b.team,
        );
        hinge
    });
    let legs = std::array::from_fn(|i| {
        std::array::from_fn(|segment| {
            let joint = b.joint_at(root, leg_transforms(Pose::default(), i)[segment]);
            spawn_layers(
                b.commands,
                &b.assets.mansion.layers,
                &format!("leg-{i}-{}", ["upper", "lower", "foot"][segment]),
                joint,
                &b.team,
            );
            joint
        })
    });
    Rig {
        root,
        body,
        body_rest: BODY,
        head: None,
        arms: None,
        legs: [None; 4],
        weapon: None,
        cloth: None,
        retractable: false,
        hatch: None,
        car: None,
        mansion: Some(MansionRig { hatch, gates, legs }),
    }
}
pub(super) fn pose(
    rig: &Rig,
    mansion: &MansionRig,
    pose: Pose,
    joints: &mut Query<&mut Transform, With<Joint>>,
) {
    if let Ok(mut t) = joints.get_mut(rig.body) {
        t.translation = BODY + pose.offset + Vec3::Y * (0.350 * pose.leg_extension);
        t.rotation = rotation(pose.body);
    }
    if let Ok(mut t) = joints.get_mut(mansion.hatch) {
        t.rotation = Quat::from_rotation_x(pose.hatch * 1.68);
    }
    for (i, gate) in mansion.gates.into_iter().enumerate() {
        if let Ok(mut t) = joints.get_mut(gate) {
            t.rotation = Quat::from_rotation_y(pose.extension * if i == 0 { 0.75 } else { -0.75 });
        }
    }
    for (i, leg) in mansion.legs.iter().enumerate() {
        for (joint, transform) in leg.iter().zip(leg_transforms(pose, i)) {
            if let Ok(mut t) = joints.get_mut(*joint) {
                *t = transform;
            }
        }
    }
}
fn leg_transforms(pose: Pose, index: usize) -> [Transform; 3] {
    let deploy = pose.leg_extension.clamp(0.0, 1.0);
    let width = deploy.max(0.0001);
    let x = if index == 0 { -0.180 } else { 0.180 };
    let step = pose.legs[index];
    let hip = BODY
        + pose.offset
        + Vec3::Y * (0.350 * deploy)
        + rotation(pose.body) * (Vec3::new(x, 0.0, 0.0) - BODY);
    let lift = step.y.max(0.0) * 0.070 * deploy;
    let z = step.x.sin() * 0.125 * deploy;
    let foot = Vec3::new(x, 0.055 * deploy + lift, z);
    let knee = (hip + foot) * 0.5 + Vec3::new(0.0, 0.0, -0.025 * deploy);
    let segment = |a: Vec3, b: Vec3, length: f32| {
        let d = b - a;
        Transform::from_translation(a)
            .with_rotation(Quat::from_rotation_arc(
                Vec3::NEG_Y,
                if d.length_squared() > 0.00000001 {
                    d.normalize()
                } else {
                    Vec3::NEG_Y
                },
            ))
            .with_scale(Vec3::new(width, (d.length() / length).max(0.0001), width))
    };
    [
        segment(hip, knee, 0.150),
        segment(knee, foot, 0.145),
        Transform::from_translation(foot).with_scale(Vec3::splat(width)),
    ]
}
pub(super) fn launch(pose: Pose) -> Vec3 {
    BODY + pose.offset
        + Vec3::Y * (0.350 * pose.leg_extension)
        + rotation(pose.body) * (LAUNCH - BODY)
}

#[derive(Default)]
pub(crate) struct MansionMolotovAssets {
    layers: Vec<Layer>,
}
impl MansionMolotovAssets {
    pub(crate) fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            layers: layers(meshes, materials, false),
        }
    }
    pub(crate) fn spawn(&self, commands: &mut Commands, root: Entity) {
        for part in ["bottle", "wick"] {
            spawn_layers(commands, &self.layers, part, root, &Handle::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn mansion_walk_legs_hide_at_rest_and_plant_feet_on_the_board() {
        for i in 0..2 {
            let rest = leg_transforms(Pose::default(), i);
            assert!(rest.iter().all(|t| t.scale.x <= 0.001));
            for segment in ["upper", "lower", "foot"] {
                assert!(
                    PARTS
                        .iter()
                        .any(|(part, _, _)| *part == format!("leg-{i}-{segment}"))
                );
            }
        }
        let mut lifted = false;
        for sample in 0..=60 {
            let t = sample as f32 / 60.0;
            let p = crate::characters::animation::mansion_moving(t, 3.0, 1.0);
            let feet = [0, 1].map(|i| leg_transforms(p, i)[2]);
            for foot in feet {
                let floor = foot.translation.y - 0.055 * foot.scale.y;
                assert!(floor >= -0.0001, "feet stay above the board");
                lifted |= floor > 0.03;
                assert!(foot.translation.z.abs() < 0.13);
            }
            assert!(
                feet.iter()
                    .any(|f| f.translation.y - 0.055 * f.scale.y < 0.0001),
                "one foot stays planted"
            );
            for i in 0..2 {
                for joint in leg_transforms(p, i) {
                    assert!(
                        joint.translation.is_finite()
                            && joint.rotation.is_finite()
                            && joint.scale.is_finite()
                    );
                }
            }
        }
        assert!(lifted);
    }
    #[test]
    fn mansion_geometry_fits_its_square_and_contains_the_animated_parts() {
        let (mesh, rest) = skin::unpack(include_bytes!("../../../art/mansion/mansion.mesh"));
        assert!(rest.is_empty());
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        assert!(points.iter().all(|p| p.iter().all(|v| v.is_finite())
            && p[1] >= -0.001
            && p[1] < 1.10
            && Vec2::new(p[0], p[2]).length() * crate::characters::BOARD_SCALE < 0.5));
        assert!(
            mesh.indices()
                .unwrap()
                .iter()
                .all(|i| i < mesh.count_vertices())
        );
        for part in ["body", "hatch", "gate-0", "gate-1", "bottle", "wick"] {
            assert!(PARTS.iter().any(|(p, _, _)| *p == part), "{part}");
        }
    }
}
