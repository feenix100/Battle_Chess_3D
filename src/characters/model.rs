use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
use shakmaty::Role;

use super::animation::Pose;

mod horse;
mod pawn;
mod rook;
mod skin;

use skin::SkinAssets;

#[derive(Component)]
pub(crate) struct Joint;

#[derive(Clone, Copy)]
struct Limb {
    upper: Entity,
    lower: Entity,
}

pub(super) struct Rig {
    pub root: Entity,
    body: Entity,
    body_rest: Vec3,
    head: Option<Entity>,
    arms: Option<[Limb; 2]>,
    legs: [Option<Limb>; 4],
    weapon: Option<Entity>,
    cloth: Option<Entity>,
    retractable: bool,
    hatch: Option<[Entity; 2]>,
}

impl Rig {
    pub fn pose(&self, pose: Pose, joints: &mut Query<&mut Transform, With<Joint>>) {
        let lift = if self.retractable {
            rook::extend(self, pose.extension, joints)
        } else {
            0.0
        };
        if let Ok(mut body) = joints.get_mut(self.body) {
            body.translation = self.body_rest + pose.offset + Vec3::Y * lift;
            body.rotation = rotation(pose.body);
        }
        if let Some(head) = self.head {
            set_rotation(joints, head, rotation(pose.head));
        }
        if let Some(arms) = self.arms {
            for (limb, angles) in arms.into_iter().zip(pose.arms) {
                set_rotation(
                    joints,
                    limb.upper,
                    rotation(Vec3::new(angles.x, 0.0, angles.y)),
                );
                set_rotation(joints, limb.lower, Quat::from_rotation_x(angles.z));
            }
        }
        for (limb, angles) in self.legs.into_iter().zip(pose.legs) {
            if let Some(limb) = limb {
                set_rotation(joints, limb.upper, Quat::from_rotation_x(angles.x));
                set_rotation(joints, limb.lower, Quat::from_rotation_x(angles.y));
            }
        }
        if let Some(weapon) = self.weapon {
            set_rotation(joints, weapon, Quat::from_rotation_x(pose.weapon));
        }
        if let Some(cloth) = self.cloth {
            set_rotation(joints, cloth, Quat::from_rotation_x(pose.cloth));
        }
        if let Some([left, right]) = self.hatch {
            let open = pose.hatch * 1.72;
            set_rotation(joints, left, Quat::from_rotation_z(open));
            set_rotation(joints, right, Quat::from_rotation_z(-open));
        }
    }
}

fn rotation(angles: Vec3) -> Quat {
    Quat::from_euler(EulerRot::YXZ, angles.y, angles.x, angles.z)
}

fn set_rotation(joints: &mut Query<&mut Transform, With<Joint>>, entity: Entity, rotation: Quat) {
    if let Ok(mut joint) = joints.get_mut(entity) {
        joint.rotation = rotation;
    }
}

#[derive(Resource, Default)]
pub(crate) struct CharacterAssets {
    rook: rook::RookMeshes,
    horse: SkinAssets,
    pawn: SkinAssets,
    bishop: SkinAssets,
    queen: SkinAssets,
    king: SkinAssets,
}

impl CharacterAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Self {
        Self {
            rook: rook::RookMeshes::new(meshes),
            horse: SkinAssets::new(horse::MESH, meshes, bindposes),
            pawn: SkinAssets::new(pawn::MESH, meshes, bindposes),
            bishop: SkinAssets::new(
                include_bytes!("../../art/bishop/bishop.mesh"),
                meshes,
                bindposes,
            ),
            queen: SkinAssets::new(
                include_bytes!("../../art/queen/queen.mesh"),
                meshes,
                bindposes,
            ),
            king: SkinAssets::new(
                include_bytes!("../../art/king/king.mesh"),
                meshes,
                bindposes,
            ),
        }
    }
}

struct Builder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    assets: &'a CharacterAssets,
    team: Handle<StandardMaterial>,
}

impl Builder<'_, '_, '_> {
    fn joint(&mut self, parent: Entity, position: Vec3) -> Entity {
        self.joint_at(parent, Transform::from_translation(position))
    }

    fn joint_at(&mut self, parent: Entity, transform: Transform) -> Entity {
        self.commands
            .spawn((Joint, transform, Visibility::Inherited, ChildOf(parent)))
            .id()
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    kit: &CharacterAssets,
    role: Role,
    team: Handle<StandardMaterial>,
    transform: Transform,
) -> Rig {
    let root = commands.spawn((transform, Visibility::Inherited)).id();
    let mut builder = Builder {
        commands,
        assets: kit,
        team,
    };
    match role {
        Role::Knight => horse::spawn(&mut builder, root),
        Role::Pawn => spawn_biped(&mut builder, root, &kit.pawn),
        Role::Bishop => spawn_biped(&mut builder, root, &kit.bishop),
        Role::Queen => spawn_biped(&mut builder, root, &kit.queen),
        Role::King => spawn_biped(&mut builder, root, &kit.king),
        Role::Rook => rook::spawn(&mut builder, root),
    }
}

fn spawn_biped(builder: &mut Builder, root: Entity, skin: &SkinAssets) -> Rig {
    let rest = &skin.rest;
    let body = builder.joint(root, rest[0]);
    let head = builder.joint(body, rest[1] - rest[0]);
    let [left, right, left_leg, right_leg] = [3, 5, 7, 9].map(|i| {
        let upper = builder.joint(body, rest[i] - rest[0]);
        let lower = builder.joint(upper, rest[i + 1] - rest[i]);
        Limb { upper, lower }
    });
    let weapon = builder.joint(right.lower, rest[2] - rest[6]);
    let mut joints = vec![
        body,
        head,
        weapon,
        left.upper,
        left.lower,
        right.upper,
        right.lower,
        left_leg.upper,
        left_leg.lower,
        right_leg.upper,
        right_leg.lower,
    ];
    let cloth = rest.get(skin::JOINT_COUNT).map(|position| {
        let joint = builder.joint(body, *position - rest[0]);
        joints.push(joint);
        joint
    });
    skin.spawn(builder.commands, root, builder.team.clone(), joints);
    Rig {
        root,
        body,
        body_rest: rest[0],
        head: Some(head),
        arms: Some([left, right]),
        legs: [Some(left_leg), Some(right_leg), None, None],
        weapon: Some(weapon),
        cloth,
        retractable: false,
        hatch: None,
    }
}
