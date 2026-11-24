use bevy::prelude::*;

use super::{Builder, Limb, Rig};

pub(super) const MESH: &[u8] = include_bytes!("../../../art/knight/warhorse.mesh");

pub(super) fn spawn(builder: &mut Builder, root: Entity) -> Rig {
    let skin = &builder.assets.horse;
    let rest = &skin.rest;
    let body = builder.joint(root, rest[0]);
    let head = builder.joint(body, rest[1] - rest[0]);
    let tail = builder.joint(body, rest[2] - rest[0]);
    let legs = [3, 5, 7, 9].map(|i| {
        let upper = builder.joint(body, rest[i] - rest[0]);
        let lower = builder.joint(upper, rest[i + 1] - rest[i]);
        Limb { upper, lower }
    });
    let [left_fore, right_fore, left_hind, right_hind] = legs;
    skin.spawn(
        builder.commands,
        root,
        builder.team.clone(),
        vec![
            body,
            head,
            tail,
            left_fore.upper,
            left_fore.lower,
            right_fore.upper,
            right_fore.lower,
            left_hind.upper,
            left_hind.lower,
            right_hind.upper,
            right_hind.lower,
        ],
    );
    Rig {
        root,
        body,
        body_rest: rest[0],
        head: Some(head),
        arms: None,
        legs: legs.map(Some),
        weapon: None,
        cloth: Some(tail),
        retractable: false,
        hatch: None,
        car: None,
        mansion: None,
    }
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;

    use super::super::skin::{JOINT_COUNT, unpack};
    use super::*;

    #[test]
    fn embedded_horse_is_fully_weighted_and_fits_a_board_square() {
        let (mesh, rest) = unpack(MESH);
        assert!(rest.iter().all(|joint| joint.is_finite()));
        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len() % 3, 0);
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
        let mut used = [false; JOINT_COUNT];
        for (influences, ids) in weights.iter().zip(joints) {
            assert!((influences.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            for (&weight, &joint) in influences.iter().zip(ids) {
                assert!((0.0..=1.0).contains(&weight));
                if weight > 0.0 {
                    used[joint as usize] = true;
                }
            }
        }
        assert!(used.into_iter().all(|joint| joint));
        assert!(
            positions
                .iter()
                .all(|&[x, y, z]| x.abs() <= 0.27 && (0.0..=1.15).contains(&y) && z.abs() <= 0.621)
        );
    }
}
