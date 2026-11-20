pub(super) const MESH: &[u8] = include_bytes!("../../../art/pawn/pawn.mesh");

#[cfg(test)]
mod tests {
    use bevy::{mesh::VertexAttributeValues, prelude::*};

    use super::*;
    use crate::characters::model::skin::{JOINT_COUNT, unpack};

    #[test]
    fn infantry_pawn_uses_every_joint_and_fits_its_square() {
        let (mesh, rest) = unpack(MESH);
        assert!(rest.iter().all(|position| position.is_finite()));
        let indices = mesh.indices().unwrap();
        assert!(indices.len() > 1_500 && indices.len() % 3 == 0);
        assert!(indices.iter().all(|index| index < mesh.count_vertices()));
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("Missing positions")
        };
        let Some(VertexAttributeValues::Float32x4(weights)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        else {
            panic!("Missing weights")
        };
        let Some(VertexAttributeValues::Uint16x4(joints)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        else {
            panic!("Missing joint indices")
        };
        let mut active = [false; JOINT_COUNT];
        for (influences, ids) in weights.iter().zip(joints) {
            assert!((influences.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            for (&weight, &joint) in influences.iter().zip(ids) {
                assert!((0.0..=1.0).contains(&weight));
                if weight > 0.0 {
                    active[joint as usize] = true;
                }
            }
        }
        assert!(active.into_iter().all(|used| used));
        assert!(positions.iter().all(|&[x, y, z]| {
            x.abs() <= 0.36 && (0.0..=1.35).contains(&y) && (-0.37..=0.22).contains(&z)
        }));
        let mut barrel_vertices = 0;
        for ((&[x, y, z], influences), ids) in positions.iter().zip(weights).zip(joints) {
            if z < -0.30 {
                barrel_vertices += 1;
                assert!((0.22..=0.29).contains(&x) && (0.59..=0.70).contains(&y));
                let weapon_weight: f32 = influences
                    .iter()
                    .zip(ids)
                    .filter(|(_, joint)| **joint == 2)
                    .map(|(weight, _)| *weight)
                    .sum();
                assert!(weapon_weight > 0.99, "the forward barrel must stay rigid");
            }
        }
        assert!(barrel_vertices > 20, "the revolver must project forward");
    }
}
