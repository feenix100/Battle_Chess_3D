//! Original adult street-fashion queen with a ponytail, skirt rig, and mini Uzi.
use super::{SkinAssets, rotation};
use crate::characters::animation::Pose;
use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
include!("../../../art/street_queen/anchors.rs");
pub(super) fn new(
    meshes: &mut Assets<Mesh>,
    bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    materials: &mut Assets<StandardMaterial>,
) -> SkinAssets {
    let mut skin = SkinAssets::new(
        include_bytes!("../../../art/street_queen/team.mesh"),
        meshes,
        bindposes,
    );
    for (bytes, color, metallic, roughness) in [
        (
            include_bytes!("../../../art/street_queen/skin.mesh").as_slice(),
            0xad7556,
            0.0,
            0.72,
        ),
        (
            include_bytes!("../../../art/street_queen/hair.mesh").as_slice(),
            0x291b13,
            0.04,
            0.55,
        ),
        (
            include_bytes!("../../../art/street_queen/dark.mesh").as_slice(),
            0x141d29,
            0.15,
            0.43,
        ),
        (
            include_bytes!("../../../art/street_queen/gold.mesh").as_slice(),
            0xe5b95b,
            0.76,
            0.27,
        ),
        (
            include_bytes!("../../../art/street_queen/steel.mesh").as_slice(),
            0x8797a3,
            0.75,
            0.29,
        ),
        (
            include_bytes!("../../../art/street_queen/rose.mesh").as_slice(),
            0x98324f,
            0.12,
            0.46,
        ),
        (
            include_bytes!("../../../art/street_queen/ivory.mesh").as_slice(),
            0xe8e2ce,
            0.02,
            0.65,
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn street_queen_mesh_has_skirt_weights_long_ponytail_and_board_bounds() {
        let (mesh, rest) = super::super::skin::unpack(include_bytes!(
            "../../../art/street_queen/street-queen.mesh"
        ));
        assert_eq!(rest, REST);
        assert_eq!(rest.len(), 12);
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
        let mut active = [false; 12];
        for ((p, w), ids) in p.iter().zip(w).zip(ids) {
            assert!(p.iter().all(|v| v.is_finite()));
            assert!(
                p[0].abs() < 0.37 && p[1] >= -0.001 && p[1] < 1.56 && p[2].abs() < 0.45,
                "{p:?}"
            );
            assert!((w.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            for (&weight, &id) in w.iter().zip(ids) {
                assert!((0.0..=1.0).contains(&weight) && id < 12);
                active[id as usize] |= weight > 0.0;
            }
        }
        assert!(active.into_iter().all(|v| v));
        let (hair, _) =
            super::super::skin::unpack(include_bytes!("../../../art/street_queen/hair.mesh"));
        let Some(VertexAttributeValues::Float32x3(p)) = hair.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        assert!(
            p.iter().any(|p| p[1] < 0.74 && p[2] > 0.16),
            "ponytail reaches the waist"
        );
    }
    #[test]
    fn street_queen_aims_uzi_upright_and_returns_to_idle_after_spray() {
        let axis = Vec3::new(0.0, (-0.35_f32).sin(), -(-0.35_f32).cos());
        let up = Vec3::new(0.0, (-0.35_f32).cos(), (-0.35_f32).sin());
        let pose = crate::characters::animation::street_queen_attack(
            crate::projectiles::QUEEN_SMG_LAUNCH,
            3.0,
            0.0,
        );
        assert!(gun_frame(pose).transform_vector3(axis).dot(Vec3::NEG_Z) > 0.98);
        assert!(gun_frame(pose).transform_vector3(up).y > 0.98);
        assert!(
            muzzle(pose).y > muzzle(Pose::default()).y + 0.20,
            "raises the gun from the hip"
        );
        let low = crate::characters::animation::street_queen_attack(
            0.46,
            1.0,
            crate::projectiles::queen_smg_aim(1.0, 0.28),
        );
        assert!(gun_frame(low).transform_vector3(axis).y < -0.25);
        let done = crate::characters::animation::street_queen_attack(2.0, 3.0, 0.0);
        assert!(muzzle(done).distance(muzzle(Pose::default())) < 0.0001);
    }
}
