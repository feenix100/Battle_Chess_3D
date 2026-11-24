//! Articulated coupe and deployed quadcopter, built from original static STL parts.
use super::{Builder, Joint, Rig, rotation, skin};
use crate::characters::animation::Pose;
use bevy::prelude::*;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/art/lowrider/assets.rs"
));

pub(crate) const BODY: Vec3 = Vec3::new(0.0, 0.245, 0.0);
pub(crate) const HINGE: Vec3 = Vec3::new(0.0, 0.337, 0.272);
pub(crate) const BAY: Vec3 = Vec3::new(0.0, 0.235, 0.423);
const WHEELS: [Vec3; 4] = [
    Vec3::new(-0.294, 0.137, -0.355),
    Vec3::new(0.294, 0.137, -0.355),
    Vec3::new(-0.294, 0.137, 0.355),
    Vec3::new(0.294, 0.137, 0.355),
];
const ARMS: [Vec3; 4] = [
    Vec3::new(-0.025, 0.014, -0.022),
    Vec3::new(0.025, 0.014, -0.022),
    Vec3::new(-0.025, 0.014, 0.022),
    Vec3::new(0.025, 0.014, 0.022),
];
const MOTORS: [Vec3; 4] = [
    Vec3::new(-0.091, 0.026, -0.071),
    Vec3::new(0.091, 0.026, -0.071),
    Vec3::new(-0.091, 0.026, 0.071),
    Vec3::new(0.091, 0.026, 0.071),
];

struct Layer {
    part: &'static str,
    mesh: Handle<Mesh>,
    surface: Option<Handle<StandardMaterial>>,
}

fn layers(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    car: bool,
) -> Vec<Layer> {
    let mut surfaces = std::collections::HashMap::new();
    PARTS
        .iter()
        .filter(|(part, _, _)| {
            (matches!(*part, "body" | "trunk") || part.starts_with("wheel-")) == car
        })
        .map(|&(part, surface, bytes)| {
            let paint = surface == "paint";
            let material = if paint {
                None
            } else {
                Some(
                    surfaces
                        .entry(surface)
                        .or_insert_with(|| {
                            let (color, metallic, roughness, alpha) = match surface {
                                "chrome" => (0xc1cbd1, 0.88, 0.19, 1.0),
                                "gold" => (0xe0ad45, 0.78, 0.26, 1.0),
                                "rubber" => (0x13191f, 0.04, 0.83, 1.0),
                                "interior" => (0x342d26, 0.0, 0.88, 1.0),
                                "ivory" => (0xf1e9d4, 0.10, 0.39, 1.0),
                                "glass" => (0x203c51, 0.35, 0.16, 0.74),
                                "red" => (0xd92336, 0.2, 0.25, 1.0),
                                "cyan" => (0x46e3eb, 0.25, 0.21, 1.0),
                                _ => unreachable!("unknown lowrider material"),
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
pub(super) struct CarAssets {
    layers: Vec<Layer>,
}
impl CarAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            layers: layers(meshes, materials, true),
        }
    }
}

pub(super) struct CarRig {
    wheels: [(Entity, Entity); 4],
    trunk: Entity,
}

pub(super) fn spawn(b: &mut Builder, root: Entity) -> Rig {
    let body = b.joint(root, BODY);
    spawn_layers(b.commands, &b.assets.lowrider.layers, "body", body, &b.team);
    let trunk = b.joint(body, HINGE - BODY);
    spawn_layers(
        b.commands,
        &b.assets.lowrider.layers,
        "trunk",
        trunk,
        &b.team,
    );
    let wheels = std::array::from_fn(|i| {
        let steering = b.joint(root, WHEELS[i]);
        let rolling = b.joint(steering, Vec3::ZERO);
        spawn_layers(
            b.commands,
            &b.assets.lowrider.layers,
            &format!("wheel-{i}"),
            rolling,
            &b.team,
        );
        (steering, rolling)
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
        car: Some(CarRig { wheels, trunk }),
        mansion: None,
    }
}

pub(super) fn pose(
    rig: &Rig,
    car: &CarRig,
    pose: Pose,
    joints: &mut Query<&mut Transform, With<Joint>>,
) {
    if let Ok(mut transform) = joints.get_mut(rig.body) {
        transform.translation = BODY + pose.offset;
        transform.rotation = rotation(pose.body);
    }
    for (i, &(steering, rolling)) in car.wheels.iter().enumerate() {
        if let Ok(mut transform) = joints.get_mut(steering) {
            transform.translation = WHEELS[i];
            transform.rotation =
                Quat::from_rotation_y(if i < 2 { pose.vehicle.steering } else { 0.0 });
        }
        if let Ok(mut transform) = joints.get_mut(rolling) {
            transform.rotation = Quat::from_rotation_x(pose.vehicle.wheel_roll);
        }
    }
    if let Ok(mut transform) = joints.get_mut(car.trunk) {
        transform.rotation = Quat::from_rotation_x(-pose.vehicle.trunk * 1.72);
    }
}

pub(super) fn launch(pose: Pose) -> Vec3 {
    BODY + pose.offset + rotation(pose.body) * (BAY - BODY)
}

#[derive(Default)]
pub(crate) struct LowriderDroneAssets {
    layers: Vec<Layer>,
}
#[derive(Component)]
pub(crate) struct LowriderRotor {
    pub root: Entity,
    pub phase: f32,
}
#[derive(Component)]
pub(crate) struct LowriderArm {
    pub root: Entity,
    pub fold: f32,
}

impl LowriderDroneAssets {
    pub(crate) fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            layers: layers(meshes, materials, false),
        }
    }
    pub(crate) fn spawn(&self, commands: &mut Commands, root: Entity) {
        spawn_layers(commands, &self.layers, "drone", root, &Handle::default());
        for i in 0..4 {
            let fold = if i == 0 || i == 3 { 1.0 } else { -1.0 };
            let arm = commands
                .spawn((
                    Transform::from_translation(ARMS[i]),
                    Visibility::Inherited,
                    LowriderArm { root, fold },
                    ChildOf(root),
                ))
                .id();
            spawn_layers(
                commands,
                &self.layers,
                &format!("arm-{i}"),
                arm,
                &Handle::default(),
            );
            let rotor = commands
                .spawn((
                    Transform::from_translation(MOTORS[i] - ARMS[i]),
                    Visibility::Inherited,
                    LowriderRotor {
                        root,
                        phase: i as f32 * 0.8,
                    },
                    ChildOf(arm),
                ))
                .id();
            spawn_layers(
                commands,
                &self.layers,
                &format!("rotor-{i}"),
                rotor,
                &Handle::default(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn lowrider_geometry_fits_a_square_and_has_valid_articulated_parts() {
        let (mesh, rest) = skin::unpack(include_bytes!("../../../art/lowrider/lowrider.mesh"));
        assert!(rest.is_empty());
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        assert!(points.iter().all(|p| p.iter().all(|v| v.is_finite())
            && p[1] >= -0.001
            && p[1] < 0.65
            && Vec2::new(p[0], p[2]).length() * crate::characters::BOARD_SCALE < 0.50));
        assert!(
            mesh.indices()
                .unwrap()
                .iter()
                .all(|i| i < mesh.count_vertices())
        );
        for part in [
            "body", "trunk", "wheel-0", "wheel-1", "wheel-2", "wheel-3", "drone", "arm-0", "arm-1",
            "arm-2", "arm-3", "rotor-0", "rotor-1", "rotor-2", "rotor-3",
        ] {
            assert!(PARTS.iter().any(|(p, _, _)| *p == part), "{part}");
        }
    }
}
