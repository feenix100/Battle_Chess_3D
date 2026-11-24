use bevy::prelude::*;

use super::{Builder, Joint, Limb, Rig, skin};

const BODY_PIVOT: Vec3 = Vec3::new(0.0, 0.46, 0.0);
const LEG_HEIGHT: f32 = 0.56;
const LEG_ANCHOR_Y: f32 = 0.035 - BODY_PIVOT.y;
const HIP: f32 = 0.158;

#[derive(Default)]
pub(super) struct RookMeshes {
    tower: Handle<Mesh>,
    brick: Handle<Mesh>,
    hatch: [Handle<Mesh>; 2],
}

impl RookMeshes {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            tower: meshes.add(skin::unpack(include_bytes!("../../../art/rook/rook.mesh")).0),
            brick: meshes.add(skin::unpack(include_bytes!("../../../art/rook/brick.mesh")).0),
            hatch: [
                meshes.add(skin::unpack(include_bytes!("../../../art/rook/hatch-left.mesh")).0),
                meshes.add(skin::unpack(include_bytes!("../../../art/rook/hatch-right.mesh")).0),
            ],
        }
    }
}

fn arm_anchor(side: f32, extension: f32) -> Transform {
    Transform::from_xyz(side * 0.17_f32.lerp(0.373, extension), 0.246, 0.0)
        .with_scale(Vec3::splat(0.018_f32.lerp(1.0, extension)))
}

fn leg_anchor(side: f32, extension: f32) -> Transform {
    let width = 0.43_f32.lerp(1.0, extension);
    Transform::from_xyz(side * HIP, LEG_ANCHOR_Y, 0.0).with_scale(Vec3::new(
        width,
        0.018_f32.lerp(1.0, extension),
        width,
    ))
}

pub(super) fn extend(
    rig: &Rig,
    amount: f32,
    joints: &mut Query<&mut Transform, With<Joint>>,
) -> f32 {
    for (arm, side) in rig.arms.into_iter().flatten().zip([-1.0, 1.0]) {
        if let Ok(mut transform) = joints.get_mut(arm.upper) {
            *transform = arm_anchor(side, amount);
        }
    }
    for (leg, side) in rig.legs.into_iter().flatten().zip([-1.0, 1.0]) {
        if let Ok(mut transform) = joints.get_mut(leg.upper) {
            *transform = leg_anchor(side, amount);
        }
    }
    LEG_HEIGHT * amount
}

fn brick(b: &mut Builder, parent: Entity, center: Vec3, dimensions: Vec3) {
    b.commands.spawn((
        Mesh3d(b.assets.rook.brick.clone()),
        MeshMaterial3d(b.team.clone()),
        Transform::from_translation(center).with_scale(dimensions),
        ChildOf(parent),
    ));
}

pub(super) fn spawn(b: &mut Builder, root: Entity) -> Rig {
    let body = b.joint(root, BODY_PIVOT);
    b.commands.spawn((
        Mesh3d(b.assets.rook.tower.clone()),
        MeshMaterial3d(b.team.clone()),
        Transform::from_translation(-BODY_PIVOT),
        ChildOf(body),
    ));
    let hatch = std::array::from_fn(|index| {
        let side = if index == 0 { -1.0 } else { 1.0 };
        let hinge = b.joint(body, Vec3::new(side * 0.40, 0.85, 0.0) - BODY_PIVOT);
        b.commands.spawn((
            Mesh3d(b.assets.rook.hatch[index].clone()),
            MeshMaterial3d(b.team.clone()),
            Transform::IDENTITY,
            ChildOf(hinge),
        ));
        hinge
    });

    let arms = [-1.0, 1.0].map(|side| {
        let upper = b.joint_at(body, arm_anchor(side, 0.0));
        let lower = b.joint(upper, Vec3::new(side * 0.050, -0.191, 0.0));
        brick(
            b,
            upper,
            Vec3::new(side * 0.022, -0.074, 0.0),
            Vec3::new(0.224, 0.226, 0.245),
        );
        brick(
            b,
            lower,
            Vec3::new(0.0, -0.074, 0.0),
            Vec3::new(0.171, 0.189, 0.210),
        );
        brick(
            b,
            lower,
            Vec3::new(side * 0.015, -0.198, -0.016),
            Vec3::new(0.212, 0.126, 0.231),
        );
        Limb { upper, lower }
    });
    let [left, right] = [-1.0, 1.0].map(|side| {
        let upper = b.joint_at(body, leg_anchor(side, 0.0));
        let lower = b.joint(upper, Vec3::new(0.0, -0.266, 0.0));
        brick(
            b,
            upper,
            Vec3::new(0.0, -0.122, 0.0),
            Vec3::new(0.203, 0.271, 0.214),
        );
        brick(
            b,
            lower,
            Vec3::new(0.0, -0.120, 0.015),
            Vec3::new(0.175, 0.281, 0.182),
        );
        brick(
            b,
            lower,
            Vec3::new(0.0, -0.280, -0.049),
            Vec3::new(0.223, 0.098, 0.279),
        );
        Limb { upper, lower }
    });
    Rig {
        root,
        body,
        body_rest: BODY_PIVOT,
        head: None,
        arms: Some(arms),
        legs: [Some(left), Some(right), None, None],
        weapon: None,
        cloth: None,
        retractable: true,
        hatch: Some(hatch),
        car: None,
        mansion: None,
    }
}
