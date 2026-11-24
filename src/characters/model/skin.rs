use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::DynamicSkinnedMeshBounds,
    mesh::{
        Indices, PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};

pub(super) const JOINT_COUNT: usize = 11;
const POSITION_NORMAL_BYTES: usize = 24;
const SKIN_BYTES: usize = 24;

pub(super) struct SkinAssets {
    mesh: Handle<Mesh>,
    bindposes: Handle<SkinnedMeshInverseBindposes>,
    pub rest: Vec<Vec3>,
    layers: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
}

impl Default for SkinAssets {
    fn default() -> Self {
        Self {
            mesh: Handle::default(),
            bindposes: Handle::default(),
            rest: vec![Vec3::ZERO; JOINT_COUNT],
            layers: Vec::new(),
        }
    }
}

impl SkinAssets {
    pub fn new(
        bytes: &[u8],
        meshes: &mut Assets<Mesh>,
        bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Self {
        let (mesh, rest) = unpack(bytes);
        Self {
            mesh: meshes.add(mesh),
            bindposes: bindposes.add(
                rest.iter()
                    .map(|position| Mat4::from_translation(-*position))
                    .collect::<Vec<_>>(),
            ),
            rest,
            layers: Vec::new(),
        }
    }

    pub fn add_layer(
        &mut self,
        bytes: &[u8],
        material: Handle<StandardMaterial>,
        meshes: &mut Assets<Mesh>,
    ) {
        let (mesh, rest) = unpack(bytes);
        assert_eq!(
            rest, self.rest,
            "material layers must use the same bind pose"
        );
        self.layers.push((meshes.add(mesh), material));
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        parent: Entity,
        material: Handle<StandardMaterial>,
        joints: Vec<Entity>,
    ) {
        assert_eq!(joints.len(), self.rest.len());
        for (mesh, material) in std::iter::once((&self.mesh, &material))
            .chain(self.layers.iter().map(|(mesh, surface)| (mesh, surface)))
        {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
                SkinnedMesh {
                    inverse_bindposes: self.bindposes.clone(),
                    joints: joints.clone(),
                },
                DynamicSkinnedMeshBounds,
                ChildOf(parent),
            ));
        }
    }
}

pub(super) fn unpack(mut bytes: &[u8]) -> (Mesh, Vec<Vec3>) {
    let header = take::<4>(&mut bytes);
    let vertex_count = u32::from_le_bytes(take(&mut bytes)) as usize;
    let index_count = u32::from_le_bytes(take(&mut bytes)) as usize;
    let joint_count = match &header {
        b"BCM1" => JOINT_COUNT,
        b"BCM2" => u32::from_le_bytes(take(&mut bytes)) as usize,
        _ => panic!("Invalid character mesh header"),
    };
    let skinned = joint_count > 0;
    let vertex_bytes = POSITION_NORMAL_BYTES + if skinned { SKIN_BYTES } else { 0 };
    assert_eq!(
        bytes.len(),
        joint_count * 12 + vertex_count * vertex_bytes + index_count * 4
    );

    let rest = (0..joint_count)
        .map(|_| Vec3::from_array(vector(&mut bytes)))
        .collect();
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut joints = Vec::new();
    let mut weights: Vec<[f32; 4]> = Vec::new();
    if skinned {
        joints.reserve_exact(vertex_count);
        weights.reserve_exact(vertex_count);
    }
    for _ in 0..vertex_count {
        positions.push(vector(&mut bytes));
        normals.push(vector(&mut bytes));
        if skinned {
            joints.push(std::array::from_fn(|_| {
                u16::from_le_bytes(take(&mut bytes))
            }));
            weights.push(std::array::from_fn(|_| {
                f32::from_le_bytes(take(&mut bytes))
            }));
        }
    }
    let indices = bytes
        .chunks_exact(4)
        .map(|index| u32::from_le_bytes(index.try_into().unwrap()))
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices));
    if skinned {
        mesh = mesh
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(joints),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights)
            .with_generated_skinned_mesh_bounds()
            .expect("Invalid character skin");
    }
    (mesh, rest)
}

fn take<const N: usize>(bytes: &mut &[u8]) -> [u8; N] {
    let (head, tail) = bytes.split_at(N);
    *bytes = tail;
    head.try_into().unwrap()
}

fn vector(bytes: &mut &[u8]) -> [f32; 3] {
    std::array::from_fn(|_| f32::from_le_bytes(take(bytes)))
}
