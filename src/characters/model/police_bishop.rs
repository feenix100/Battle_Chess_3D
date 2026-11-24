//! Original police uniform on the shared eleven-joint biped rig.
use super::{SkinAssets, rotation};
use crate::characters::animation::Pose;
use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
include!("../../../art/police_bishop/anchors.rs");

pub(super) fn new(
    meshes: &mut Assets<Mesh>,
    bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    materials: &mut Assets<StandardMaterial>,
) -> SkinAssets {
    let mut skin = SkinAssets::new(
        include_bytes!("../../../art/police_bishop/team.mesh"),
        meshes,
        bindposes,
    );
    for (bytes, color, metallic, roughness) in [
        (
            include_bytes!("../../../art/police_bishop/skin.mesh").as_slice(),
            0x925231,
            0.0,
            0.72,
        ),
        (
            include_bytes!("../../../art/police_bishop/navy.mesh").as_slice(),
            0x17283b,
            0.0,
            0.76,
        ),
        (
            include_bytes!("../../../art/police_bishop/dark.mesh").as_slice(),
            0x121820,
            0.1,
            0.46,
        ),
        (
            include_bytes!("../../../art/police_bishop/gold.mesh").as_slice(),
            0xe7bc60,
            0.76,
            0.27,
        ),
        (
            include_bytes!("../../../art/police_bishop/steel.mesh").as_slice(),
            0xaebcc5,
            0.72,
            0.32,
        ),
        (
            include_bytes!("../../../art/police_bishop/yellow.mesh").as_slice(),
            0xf3ca36,
            0.15,
            0.42,
        ),
        (
            include_bytes!("../../../art/police_bishop/visor.mesh").as_slice(),
            0x344e61,
            0.65,
            0.19,
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

pub(super) fn muzzle(pose: Pose) -> Vec3 {
    let arm = pose.arms[1];
    let body = Mat4::from_rotation_translation(rotation(pose.body), REST[0] + pose.offset);
    let upper =
        Mat4::from_rotation_translation(rotation(Vec3::new(arm.x, 0.0, arm.y)), REST[5] - REST[0]);
    let lower = Mat4::from_rotation_translation(Quat::from_rotation_x(arm.z), REST[6] - REST[5]);
    let weapon =
        Mat4::from_rotation_translation(Quat::from_rotation_x(pose.weapon), REST[2] - REST[6]);
    (body * upper * lower * weapon).transform_point3(MUZZLE - REST[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    #[test]
    fn police_bishop_mesh_has_valid_weights_and_board_bounds() {
        let (mesh, rest) = super::super::skin::unpack(include_bytes!(
            "../../../art/police_bishop/police-bishop.mesh"
        ));
        assert_eq!(rest, REST);
        assert!(mesh.count_vertices() > 10_000);
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
                p[0].abs() < 0.36 && p[1] >= -0.001 && p[1] < 1.56 && p[2].abs() < 0.36,
                "{p:?}"
            );
            assert!((w.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            for (&weight, &id) in w.iter().zip(ids) {
                assert!((0.0..=1.0).contains(&weight) && id < 11);
                active[id as usize] |= weight > 0.0;
            }
        }
        assert!(active.into_iter().all(|v| v));
        // All three primary uniform regions belong to the team-colored mesh.
        let (team, _) =
            super::super::skin::unpack(include_bytes!("../../../art/police_bishop/team.mesh"));
        let Some(VertexAttributeValues::Float32x3(p)) = team.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!()
        };
        for (low, high) in [(0.16, 0.30), (0.72, 1.02), (1.28, 1.54)] {
            assert!(
                p.iter().filter(|p| p[1] >= low && p[1] <= high).count() > 100,
                "pants, shirt, and helmet must follow team color"
            );
        }
    }
}
