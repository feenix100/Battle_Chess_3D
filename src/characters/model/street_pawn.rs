//! Original streetwear pawn, using BattleChess's existing eleven-joint rig.
use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};

use super::{SkinAssets, rotation};
use crate::characters::animation::Pose;

pub(super) const TEAM: &[u8] = include_bytes!("../../../art/street_pawn/team.mesh");
#[cfg(test)]
pub(super) const COMPLETE: &[u8] = include_bytes!("../../../art/street_pawn/street_pawn.mesh");

pub(super) fn new(
    meshes: &mut Assets<Mesh>,
    bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    materials: &mut Assets<StandardMaterial>,
) -> SkinAssets {
    let mut skin = SkinAssets::new(TEAM, meshes, bindposes);
    for (bytes, color, metallic, roughness) in [
        (
            include_bytes!("../../../art/street_pawn/skin.mesh").as_slice(),
            0xb8815e,
            0.0,
            0.72,
        ),
        (
            include_bytes!("../../../art/street_pawn/denim.mesh").as_slice(),
            0x283644,
            0.0,
            0.85,
        ),
        (
            include_bytes!("../../../art/street_pawn/ivory.mesh").as_slice(),
            0xe8e2cf,
            0.0,
            0.69,
        ),
        (
            include_bytes!("../../../art/street_pawn/gold.mesh").as_slice(),
            0xe6b64f,
            0.78,
            0.28,
        ),
        (
            include_bytes!("../../../art/street_pawn/dark.mesh").as_slice(),
            0x171d25,
            0.18,
            0.42,
        ),
        (
            include_bytes!("../../../art/street_pawn/stitch.mesh").as_slice(),
            0x63717c,
            0.05,
            0.64,
        ),
    ] {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb_u8((color >> 16) as u8, (color >> 8) as u8, color as u8),
            metallic,
            perceptual_roughness: roughness,
            ..default()
        });
        skin.add_layer(bytes, material, meshes);
    }
    skin
}

/// Forward kinematics agrees with the weapon joint and pose used by the model.
pub(super) fn muzzle(pose: Pose) -> Vec3 {
    let rest = [
        Vec3::new(0.0, 0.555, 0.010),
        Vec3::ZERO,
        Vec3::new(0.255, 0.646, -0.090),
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::new(0.205, 0.916, 0.005),
        Vec3::new(0.252, 0.748, -0.012),
    ];
    let arm = pose.arms[1];
    let body = Mat4::from_rotation_translation(rotation(pose.body), rest[0] + pose.offset);
    let upper =
        Mat4::from_rotation_translation(rotation(Vec3::new(arm.x, 0.0, arm.y)), rest[5] - rest[0]);
    let lower = Mat4::from_rotation_translation(Quat::from_rotation_x(arm.z), rest[6] - rest[5]);
    let weapon =
        Mat4::from_rotation_translation(Quat::from_rotation_x(pose.weapon), rest[2] - rest[6]);
    (body * upper * lower * weapon).transform_point3(Vec3::new(0.255, 0.674, -0.361) - rest[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn detailed_street_pawn_is_fully_weighted_and_fits_the_board() {
        let (mesh, rest) = super::super::skin::unpack(COMPLETE);
        assert_eq!(rest.len(), 11);
        assert!(mesh.count_vertices() > 7_000);
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        let Some(VertexAttributeValues::Float32x4(weights)) =
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
        for ((p, w), ids) in positions.iter().zip(weights).zip(ids) {
            assert!(p.iter().all(|v| v.is_finite()));
            assert!(p[0].abs() < 0.38 && p[1] >= -0.001 && p[1] < 1.36 && p[2].abs() < 0.38);
            assert!((w.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            for (&weight, &id) in w.iter().zip(ids) {
                assert!((0.0..=1.0).contains(&weight) && id < 11);
                active[id as usize] |= weight > 0.0;
            }
            if p[2] < -0.30 {
                let rigid: f32 = w
                    .iter()
                    .zip(ids)
                    .filter(|(_, id)| **id == 2)
                    .map(|(w, _)| *w)
                    .sum();
                assert!(
                    rigid > 0.999,
                    "pistol muzzle must follow its rigid weapon joint"
                );
            }
        }
        assert!(active.into_iter().all(|v| v));
    }
}
