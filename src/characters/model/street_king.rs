//! Original heavy king with a diagonally shoulder-carried shotgun.
use super::{SkinAssets, rotation};
use crate::characters::animation::Pose;
use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
include!("../../../art/street_king/anchors.rs");

pub(super) fn new(
    meshes: &mut Assets<Mesh>,
    bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    materials: &mut Assets<StandardMaterial>,
) -> SkinAssets {
    let mut skin = SkinAssets::new(
        include_bytes!("../../../art/street_king/team.mesh"),
        meshes,
        bindposes,
    );
    for (bytes, color, metallic, roughness) in [
        (
            include_bytes!("../../../art/street_king/skin.mesh").as_slice(),
            0xa56842,
            0.0,
            0.72,
        ),
        (
            include_bytes!("../../../art/street_king/dark.mesh").as_slice(),
            0x161d24,
            0.14,
            0.44,
        ),
        (
            include_bytes!("../../../art/street_king/gold.mesh").as_slice(),
            0xe5b65c,
            0.76,
            0.27,
        ),
        (
            include_bytes!("../../../art/street_king/steel.mesh").as_slice(),
            0x818d96,
            0.75,
            0.28,
        ),
        (
            include_bytes!("../../../art/street_king/wood.mesh").as_slice(),
            0x71401e,
            0.04,
            0.48,
        ),
        (
            include_bytes!("../../../art/street_king/stitch.mesh").as_slice(),
            0x90998d,
            0.0,
            0.76,
        ),
        (
            include_bytes!("../../../art/street_king/eye.mesh").as_slice(),
            0xe6dfca,
            0.0,
            0.54,
        ),
    ] {
        skin.add_layer(
            bytes,
            materials.add(StandardMaterial {
                base_color: Color::srgb_u8((color >> 16) as u8, (color >> 8) as u8, color as u8),
                metallic,
                perceptual_roughness: roughness,
                ..default()
            }),
            meshes,
        );
    }
    skin
}
fn gun_frame(pose: Pose) -> Mat4 {
    let arm = pose.arms[1];
    Mat4::from_rotation_translation(rotation(pose.body), REST[0] + pose.offset)
        * Mat4::from_rotation_translation(
            rotation(Vec3::new(arm.x, pose.arm_yaw[1], arm.y)),
            REST[5] - REST[0],
        )
        * Mat4::from_rotation_translation(Quat::from_rotation_x(arm.z), REST[6] - REST[5])
        * Mat4::from_rotation_translation(
            rotation(Vec3::new(pose.weapon, pose.weapon_yaw, 0.0)),
            REST[2] - REST[6],
        )
}
pub(super) fn muzzle(pose: Pose) -> Vec3 {
    gun_frame(pose).transform_point3(MUZZLE - REST[2])
}
pub(super) fn ejection(pose: Pose) -> Vec3 {
    gun_frame(pose).transform_point3(EJECT - REST[2])
}

/// Position the free hand on the fore-end, including downward shots and recoil.
pub(super) fn support(mut pose: Pose) -> Pose {
    let body = Mat4::from_rotation_translation(rotation(pose.body), REST[0] + pose.offset);
    let point =
        REST[2] + Vec3::new(0.0, BARREL_AXIS.z, -BARREL_AXIS.y) * -0.035 + BARREL_AXIS * 0.175;
    let target = REST[0]
        + body
            .inverse()
            .transform_point3(gun_frame(pose).transform_point3(point - REST[2]));
    let upper = REST[4] - REST[3];
    let lower = LEFT_HAND - REST[4];
    let a = upper.length();
    let b = lower.length();
    let axis = (target - REST[3]).normalize();
    let distance = target
        .distance(REST[3])
        .clamp((a - b).abs() + 0.0001, a + b - 0.0001);
    let along = (a * a - b * b + distance * distance) / (2.0 * distance);
    let reach = (a * a - along * along).max(0.0).sqrt();
    let preferred = Vec3::new(0.0, -0.6, -1.0);
    let side = (preferred - axis * preferred.dot(axis)).normalize();
    let elbow = REST[3] + axis * along + side * reach;
    let end = REST[3] + axis * distance;
    let desired_upper = elbow - REST[3];
    let desired_lower = end - elbow;
    let cosine = desired_upper.dot(desired_lower) / (a * b);
    let (mut lo, mut hi) = (0.0, std::f32::consts::PI);
    for _ in 0..24 {
        let r = (lo + hi) * 0.5;
        if upper.dot(Quat::from_rotation_x(r) * lower) / (a * b) > cosine {
            lo = r;
        } else {
            hi = r;
        }
    }
    let bend = (lo + hi) * 0.5;
    let basis = |u: Vec3, v: Vec3| {
        let x = u.normalize();
        let z = u.cross(v).normalize();
        Mat3::from_cols(x, z.cross(x), z)
    };
    let turn = basis(desired_upper, desired_lower)
        * basis(upper, Quat::from_rotation_x(bend) * lower).transpose();
    let (yaw, pitch, roll) = Quat::from_mat3(&turn).to_euler(EulerRot::YXZ);
    pose.arms[0] = Vec3::new(pitch, roll, bend);
    pose.arm_yaw[0] = yaw;
    pose
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn street_king_mesh_has_valid_weights_and_board_bounds() {
        let (mesh, rest) =
            super::super::skin::unpack(include_bytes!("../../../art/street_king/street-king.mesh"));
        assert_eq!(rest, REST);
        assert!(mesh.count_vertices() > 18_000);
        let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        let Some(VertexAttributeValues::Float32x4(w)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        else {
            panic!()
        };
        let Some(VertexAttributeValues::Uint16x4(ids)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        else {
            panic!()
        };
        let mut active = [false; 11];
        for ((p, w), ids) in p.iter().zip(w).zip(ids) {
            assert!(p.iter().all(|v| v.is_finite()));
            assert!(
                p[0].abs() < 0.40 && p[1] >= -0.001 && p[1] < 1.60 && p[2].abs() < 0.48,
                "{p:?}"
            );
            assert!((w.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            for (&weight, &id) in w.iter().zip(ids) {
                assert!((0.0..=1.0).contains(&weight) && id < 11);
                active[id as usize] |= weight > 0.0;
            }
        }
        assert!(active.into_iter().all(|v| v));
    }
    #[test]
    fn street_king_carries_diagonally_then_aims_and_returns_to_shoulder() {
        let axis = |pose| gun_frame(pose).transform_vector3(BARREL_AXIS).normalize();
        let carry = axis(Pose::default());
        assert!(carry.y > 0.65 && carry.z > 0.65, "barrel leans up and back");
        let up = Vec3::new(0.0, BARREL_AXIS.z, -BARREL_AXIS.y);
        assert!(
            (REST[2] + BARREL_AXIS * 0.17 - up * 0.035).distance(REST[5]) < 0.09,
            "gun rests against the shoulder"
        );
        let ready = crate::characters::animation::street_king_attack(
            crate::projectiles::KING_SHOTGUN_LAUNCH,
            0.0,
        );
        assert!(
            axis(ready).dot(Vec3::NEG_Z) > 0.98,
            "lowered barrel must aim at the defender"
        );
        assert!(
            gun_frame(ready)
                .transform_vector3(Vec3::new(0.0, BARREL_AXIS.z, -BARREL_AXIS.y))
                .y
                > 0.95,
            "the gun stays upright when lowered"
        );
        let support =
            REST[2] + Vec3::new(0.0, BARREL_AXIS.z, -BARREL_AXIS.y) * -0.035 + BARREL_AXIS * 0.175;
        let target = gun_frame(ready).transform_point3(support - REST[2]);
        let a = ready.arms[0];
        let left = Mat4::from_rotation_translation(rotation(ready.body), REST[0] + ready.offset)
            * Mat4::from_rotation_translation(
                rotation(Vec3::new(a.x, ready.arm_yaw[0], a.y)),
                REST[3] - REST[0],
            )
            * Mat4::from_rotation_translation(Quat::from_rotation_x(a.z), REST[4] - REST[3]);
        assert!(
            left.transform_point3(LEFT_HAND - REST[4]).distance(target) < 0.015,
            "support hand meets the fore-end"
        );
        let restored = crate::characters::animation::street_king_attack(1.60, 0.0);
        assert!(axis(restored).distance(carry) < 0.0001);
        for t in [0.1, 0.25, 0.50, 0.75, 0.90] {
            assert!(
                axis(crate::characters::animation::street_king_moving(t, 1.0)).dot(carry) > 0.98,
                "gun stays carried while walking"
            );
        }
        for direction in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
            let p = crate::characters::animation::street_king_yielding(direction, 0.7);
            assert_eq!(p.weapon, 0.0);
            assert_eq!(p.weapon_yaw, 0.0);
            assert_eq!(p.arms[1], Vec3::ZERO);
        }
        let low = crate::characters::animation::street_king_attack(
            0.68,
            crate::projectiles::king_shotgun_aim(1.0, 0.28),
        );
        assert!(axis(low).y < -0.25, "aim down at the lowrider");
    }
}
