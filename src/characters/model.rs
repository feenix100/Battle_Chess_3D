use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
use shakmaty::Role;

use super::animation::Pose;

mod horse;
mod lowrider;
mod mansion;
mod pawn;
mod police_bishop;
mod rook;
mod skin;
mod street_king;
mod street_pawn;
mod street_queen;

pub(crate) use lowrider::{LowriderArm, LowriderDroneAssets, LowriderRotor};
pub(crate) use mansion::MansionMolotovAssets;
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
    car: Option<lowrider::CarRig>,
    mansion: Option<mansion::MansionRig>,
}

impl Rig {
    pub fn pose(&self, pose: Pose, joints: &mut Query<&mut Transform, With<Joint>>) {
        if let Some(mansion) = &self.mansion {
            mansion::pose(self, mansion, pose, joints);
            return;
        }
        if let Some(car) = &self.car {
            lowrider::pose(self, car, pose, joints);
            return;
        }
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
            for (index, (limb, angles)) in arms.into_iter().zip(pose.arms).enumerate() {
                set_rotation(
                    joints,
                    limb.upper,
                    rotation(Vec3::new(angles.x, pose.arm_yaw[index], angles.y)),
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
            set_rotation(
                joints,
                weapon,
                rotation(Vec3::new(pose.weapon, pose.weapon_yaw, 0.0)),
            );
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
    street_pawn: SkinAssets,
    lowrider: lowrider::CarAssets,
    mansion: mansion::MansionAssets,
    bishop: SkinAssets,
    police_bishop: SkinAssets,
    queen: SkinAssets,
    street_queen: SkinAssets,
    king: SkinAssets,
    street_king: SkinAssets,
}

impl CharacterAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        Self {
            rook: rook::RookMeshes::new(meshes),
            horse: SkinAssets::new(horse::MESH, meshes, bindposes),
            pawn: SkinAssets::new(pawn::MESH, meshes, bindposes),
            street_pawn: street_pawn::new(meshes, bindposes, materials),
            lowrider: lowrider::CarAssets::new(meshes, materials),
            mansion: mansion::MansionAssets::new(meshes, materials),
            police_bishop: police_bishop::new(meshes, bindposes, materials),
            bishop: SkinAssets::new(
                include_bytes!("../../art/bishop/bishop.mesh"),
                meshes,
                bindposes,
            ),
            street_queen: street_queen::new(meshes, bindposes, materials),
            queen: SkinAssets::new(
                include_bytes!("../../art/queen/queen.mesh"),
                meshes,
                bindposes,
            ),
            street_king: street_king::new(meshes, bindposes, materials),
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
    style: crate::PieceStyle,
) -> Rig {
    let root = commands.spawn((transform, Visibility::Inherited)).id();
    let mut builder = Builder {
        commands,
        assets: kit,
        team,
    };
    match role {
        Role::Knight => {
            if style == crate::PieceStyle::GtaStreet {
                lowrider::spawn(&mut builder, root)
            } else {
                horse::spawn(&mut builder, root)
            }
        }
        Role::Pawn => spawn_biped(
            &mut builder,
            root,
            if style == crate::PieceStyle::GtaStreet {
                &kit.street_pawn
            } else {
                &kit.pawn
            },
        ),
        Role::Bishop => spawn_biped(
            &mut builder,
            root,
            if style == crate::PieceStyle::GtaStreet {
                &kit.police_bishop
            } else {
                &kit.bishop
            },
        ),
        Role::Queen => spawn_biped(
            &mut builder,
            root,
            if style == crate::PieceStyle::GtaStreet {
                &kit.street_queen
            } else {
                &kit.queen
            },
        ),
        Role::King => spawn_biped(
            &mut builder,
            root,
            if style == crate::PieceStyle::GtaStreet {
                &kit.street_king
            } else {
                &kit.king
            },
        ),
        Role::Rook => {
            if style == crate::PieceStyle::GtaStreet {
                mansion::spawn(&mut builder, root)
            } else {
                rook::spawn(&mut builder, root)
            }
        }
    }
}

pub(super) fn queen_muzzle(pose: Pose) -> Vec3 {
    street_queen::muzzle(pose)
}
pub(super) fn queen_ejection(pose: Pose) -> Vec3 {
    street_queen::ejection(pose)
}

pub(super) fn king_muzzle(pose: Pose) -> Vec3 {
    street_king::muzzle(pose)
}
pub(super) fn king_ejection(pose: Pose) -> Vec3 {
    street_king::ejection(pose)
}
pub(super) fn king_support(pose: Pose) -> Pose {
    street_king::support(pose)
}

pub(super) fn police_muzzle(pose: Pose) -> Vec3 {
    police_bishop::muzzle(pose)
}

pub(super) fn street_muzzle(pose: Pose) -> Vec3 {
    street_pawn::muzzle(pose)
}

pub(super) fn lowrider_launch(pose: Pose) -> Vec3 {
    lowrider::launch(pose)
}

pub(super) fn mansion_launch(pose: Pose) -> Vec3 {
    mansion::launch(pose)
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
        car: None,
        mansion: None,
    }
}
