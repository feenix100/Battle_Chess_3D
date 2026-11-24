use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shakmaty::Role;

#[derive(Clone, Copy, Default)]
pub(super) struct VehiclePose {
    pub wheel_roll: f32,
    pub steering: f32,
    pub trunk: f32,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Pose {
    pub offset: Vec3,
    pub body: Vec3,
    pub head: Vec3,
    pub arms: [Vec3; 2],
    pub arm_yaw: [f32; 2],
    pub legs: [Vec2; 4],
    pub weapon: f32,
    pub weapon_yaw: f32,
    pub cloth: f32,
    pub extension: f32,
    pub leg_extension: f32,
    pub hatch: f32,
    pub vehicle: VehiclePose,
}

impl Pose {
    fn blend(self, next: Self, t: f32) -> Self {
        Self {
            offset: self.offset.lerp(next.offset, t),
            body: self.body.lerp(next.body, t),
            head: self.head.lerp(next.head, t),
            arms: std::array::from_fn(|i| self.arms[i].lerp(next.arms[i], t)),
            arm_yaw: std::array::from_fn(|i| self.arm_yaw[i].lerp(next.arm_yaw[i], t)),
            legs: std::array::from_fn(|i| self.legs[i].lerp(next.legs[i], t)),
            weapon: self.weapon.lerp(next.weapon, t),
            weapon_yaw: self.weapon_yaw.lerp(next.weapon_yaw, t),
            cloth: self.cloth.lerp(next.cloth, t),
            extension: self.extension.lerp(next.extension, t),
            leg_extension: self.leg_extension.lerp(next.leg_extension, t),
            hatch: self.hatch.lerp(next.hatch, t),
            vehicle: VehiclePose {
                wheel_roll: self.vehicle.wheel_roll.lerp(next.vehicle.wheel_roll, t),
                steering: self.vehicle.steering.lerp(next.vehicle.steering, t),
                trunk: self.vehicle.trunk.lerp(next.vehicle.trunk, t),
            },
        }
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn keyframes(t: f32, frames: &[(f32, Pose)]) -> Pose {
    for pair in frames.windows(2) {
        let [(start, from), (end, to)] = pair else {
            unreachable!()
        };
        if t <= *end {
            return from.blend(*to, ease((t - start) / (end - start)));
        }
    }
    frames.last().unwrap().1
}

const ROOK_UNFOLD_SECONDS: f32 = 0.18;
const ROOK_FOLD_SECONDS: f32 = 0.18;

pub(super) fn move_seconds(role: Role, distance: f32) -> f32 {
    match role {
        Role::Knight => 0.99,
        Role::Rook => ROOK_UNFOLD_SECONDS + (0.37 + distance * 0.08).min(0.76) + ROOK_FOLD_SECONDS,
        Role::Queen => (0.25 + distance * 0.065).min(0.62),
        _ => (0.27 + distance * 0.085).min(0.68),
    }
}

pub(super) fn rook_walk_phase(elapsed: f32, duration: f32) -> (f32, f32) {
    let walk_seconds = duration - ROOK_UNFOLD_SECONDS - ROOK_FOLD_SECONDS;
    let travel = ((elapsed - ROOK_UNFOLD_SECONDS) / walk_seconds).clamp(0.0, 1.0);
    let unfold = ease(elapsed / ROOK_UNFOLD_SECONDS);
    let fold = ease((duration - elapsed) / ROOK_FOLD_SECONDS);
    (travel, unfold.min(fold))
}

pub(super) fn walk_position(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    // A short acceleration and deceleration keep the stride legible.
    from.lerp(to, 0.70 * t + 0.30 * ease(t))
}

const FIRST_LANDING: f32 = 0.47;
const SECOND_TAKEOFF: f32 = 0.65;

pub(super) struct KnightRoute {
    from: Vec3,
    corner: Vec3,
    to: Vec3,
}

impl KnightRoute {
    pub fn new(from: Vec3, to: Vec3) -> Self {
        let delta = to - from;
        let corner = if delta.x.abs() > delta.z.abs() {
            Vec3::new(to.x, from.y, from.z)
        } else {
            Vec3::new(from.x, from.y, to.z)
        };
        Self { from, corner, to }
    }

    pub fn sample(&self, t: f32) -> (Vec3, Quat, Pose) {
        let first = heading(self.from, self.corner);
        let second = heading(self.corner, self.to);
        if t < FIRST_LANDING {
            let phase = t / FIRST_LANDING;
            (
                self.from.lerp(self.corner, ease((phase - 0.10) / 0.79)),
                first,
                jump(phase, 1.34),
            )
        } else if t < SECOND_TAKEOFF {
            let phase = (t - FIRST_LANDING) / (SECOND_TAKEOFF - FIRST_LANDING);
            let shift = (PI * phase).sin();
            let pose = Pose {
                offset: Vec3::new(0.0, 0.035 * shift, 0.0),
                head: Vec3::new(0.0, -0.20 * shift, 0.12 * shift),
                legs: [
                    Vec2::new(0.26 * shift, -0.34 * shift),
                    Vec2::ZERO,
                    Vec2::ZERO,
                    Vec2::new(-0.21 * shift, 0.26 * shift),
                ],
                ..default()
            };
            (self.corner, first.slerp(second, ease(phase)), pose)
        } else {
            let phase = (t - SECOND_TAKEOFF) / (1.0 - SECOND_TAKEOFF);
            (
                self.corner.lerp(self.to, ease((phase - 0.10) / 0.79)),
                second,
                jump(phase, 0.70),
            )
        }
    }

    pub fn clearances(&self, t: f32) -> [(Vec3, Vec3, f32); 2] {
        let incoming = (self.corner - self.from).normalize();
        let outgoing = (self.to - self.corner).normalize();
        [
            (
                self.from.lerp(self.corner, 0.5),
                -outgoing,
                ease((t - 0.035) / 0.13).min(ease((0.56 - t) / 0.15)),
            ),
            (
                self.corner,
                (incoming - outgoing).normalize(),
                ease((t - 0.19) / 0.18).min(ease((1.0 - t) / 0.20)),
            ),
        ]
    }
}

fn jump(t: f32, height: f32) -> Pose {
    let neutral = Pose::default();
    let crouch = Pose {
        offset: Vec3::new(0.0, -0.12, 0.04),
        body: Vec3::new(0.15, 0.0, 0.0),
        head: Vec3::new(-0.24, 0.0, 0.0),
        legs: [
            Vec2::new(0.25, -0.65),
            Vec2::new(0.37, -0.54),
            Vec2::new(-0.3, 0.67),
            Vec2::new(-0.26, 0.63),
        ],
        ..neutral
    };
    let airborne = Pose {
        offset: Vec3::new(0.0, height, -0.06),
        body: Vec3::new(0.28, 0.0, 0.0),
        head: Vec3::new(0.18, 0.0, 0.0),
        legs: [
            Vec2::new(0.98, -1.65),
            Vec2::new(0.82, -1.53),
            Vec2::new(-0.62, 0.89),
            Vec2::new(-0.44, 0.91),
        ],
        cloth: 0.48,
        ..neutral
    };
    let land = Pose {
        offset: Vec3::new(0.0, -0.025, 0.0),
        body: Vec3::new(-0.19, 0.0, 0.0),
        head: Vec3::new(0.25, 0.0, 0.0),
        legs: [
            Vec2::new(0.36, -0.28),
            Vec2::new(0.29, -0.18),
            Vec2::new(0.11, 0.13),
            Vec2::new(0.18, 0.18),
        ],
        ..neutral
    };
    keyframes(
        t,
        &[
            (0.0, neutral),
            (0.12, crouch),
            (0.43, airborne),
            (0.52, airborne),
            (0.88, land),
            (1.0, neutral),
        ],
    )
}

/// Drive along the knight's two legs; steer at the corner and keep tires planted.
pub(super) fn lowrider_route(from: Vec3, to: Vec3, t: f32) -> (Vec3, Quat, Pose) {
    let route = KnightRoute::new(from, to);
    let first = heading(from, route.corner);
    let second = heading(route.corner, to);
    let (position, facing, distance, steering) = if t < 0.51 {
        let u = ease(t / 0.51);
        (
            from.lerp(route.corner, u),
            first,
            from.distance(route.corner) * u,
            0.0,
        )
    } else if t < 0.69 {
        let u = ease((t - 0.51) / 0.18);
        (
            route.corner,
            first.slerp(second, u),
            from.distance(route.corner),
            0.48 * (PI * u).sin()
                * (first.inverse() * second)
                    .to_euler(EulerRot::YXZ)
                    .0
                    .signum(),
        )
    } else {
        let u = ease((t - 0.69) / 0.31);
        (
            route.corner.lerp(to, u),
            second,
            from.distance(route.corner) + route.corner.distance(to) * u,
            0.0,
        )
    };
    let envelope = ease(t * 9.0) * ease((1.0 - t) * 9.0);
    let bounce = 0.012 * (t * TAU * 3.0).sin() * envelope;
    let pose = Pose {
        offset: Vec3::Y * (0.015 * envelope + bounce),
        body: Vec3::new(
            0.025 * (t * TAU).sin() * envelope,
            -steering * 0.06,
            steering * 0.06,
        ),
        vehicle: VehiclePose {
            wheel_roll: -distance / (0.133 * crate::characters::BOARD_SCALE),
            steering,
            trunk: 0.0,
        },
        ..default()
    };
    (position, facing, pose)
}

pub(super) fn lowrider_attack(seconds: f32) -> Pose {
    let bounce = 0.018 * (PI * (seconds / 0.42).clamp(0.0, 1.0)).sin();
    Pose {
        offset: Vec3::Y * bounce,
        body: Vec3::new(bounce * 0.7, 0.0, 0.0),
        vehicle: VehiclePose {
            trunk: crate::projectiles::lowrider_trunk_fraction(seconds),
            ..default()
        },
        ..default()
    }
}

pub(super) fn lowrider_disabled(t: f32) -> Pose {
    let down = ease(t);
    Pose {
        offset: Vec3::Y * (-0.035 * down),
        body: Vec3::new(-0.035 * down, 0.0, 0.10 * down),
        vehicle: VehiclePose {
            steering: 0.20 * down,
            ..default()
        },
        ..default()
    }
}

pub(super) fn mansion_attack(seconds: f32) -> Pose {
    let recoil = 0.008 * (PI * ((seconds - 0.35) / 0.26).clamp(0.0, 1.0)).sin();
    Pose {
        offset: Vec3::Y * recoil,
        body: Vec3::new(recoil * 0.8, 0.0, 0.0),
        hatch: crate::projectiles::mansion_hatch_fraction(seconds),
        ..default()
    }
}
pub(super) fn mansion_move_seconds(distance: f32) -> f32 {
    0.36 + (0.40 + distance * 0.20).min(1.80)
}
pub(super) fn mansion_moving(t: f32, distance: f32, deploy: f32) -> Pose {
    let weight = ease(t * 8.0) * ease((1.0 - t) * 8.0);
    let phase = t * TAU * (distance * 0.85).clamp(1.0, 5.0);
    let feet: [Vec2; 2] = std::array::from_fn(|i| {
        let p = phase + i as f32 * PI;
        Vec2::new(0.82 * p.cos() * weight, p.sin().max(0.0) * weight)
    });
    Pose {
        offset: Vec3::Y * (0.008 * phase.sin().abs() * weight),
        body: Vec3::new(
            0.012 * phase.cos() * weight,
            0.0,
            0.015 * phase.sin() * weight,
        ),
        legs: [feet[0], feet[1], Vec2::ZERO, Vec2::ZERO],
        leg_extension: deploy,
        extension: 0.06 * phase.sin().powi(2) * weight,
        ..default()
    }
}
pub(super) fn mansion_disabled(t: f32) -> Pose {
    let t = ease(t);
    Pose {
        offset: Vec3::Y * (-0.010 * t),
        body: Vec3::new(-0.025 * t, 0.0, 0.040 * t),
        extension: 0.12 * t,
        ..default()
    }
}
pub(super) fn mansion_yielding(amount: f32) -> Pose {
    Pose {
        offset: Vec3::Y * (0.007 * amount),
        body: Vec3::new(0.0, 0.04 * amount, 0.015 * amount),
        extension: 0.07 * amount,
        ..default()
    }
}

pub(super) fn moving(role: Role, t: f32, distance: f32) -> Pose {
    let neutral = Pose::default();
    // Arm carriage and cadence vary by role even when the legs share a walk cycle.
    let (cadence, stride, lean, sway, arm) = match role {
        Role::Pawn => (1.05, 0.76, -0.18, 0.10, [0.31, 0.21]),
        Role::Bishop => (0.83, 0.52, -0.07, 0.05, [0.23, 0.08]),
        Role::Rook => (0.79, 0.63, -0.07, 0.08, [0.36, 0.31]),
        Role::Queen => (1.22, 0.73, -0.17, 0.13, [0.28, 0.41]),
        Role::King => (0.95, 0.59, -0.05, 0.07, [0.17, 0.10]),
        Role::Knight => unreachable!(),
    };
    let phase = t * TAU * (distance * cadence).clamp(0.90, 3.6);
    let (step, side) = phase.sin_cos();
    let weight = ease(t * 7.0).min(ease((1.0 - t) * 6.0));
    let pose = Pose {
        offset: Vec3::new(0.022 * side, 0.035 + 0.039 * step.abs(), -0.02),
        body: Vec3::new(lean, sway * step, sway * side),
        head: Vec3::new(-lean * 0.7, -sway * step * 0.4, -sway * side * 0.4),
        arms: [
            Vec3::new(-arm[0] * step, 0.04, 0.14),
            Vec3::new(arm[1] * step, -0.06, 0.19),
        ],
        legs: [
            Vec2::new(stride * step, -0.85 * step.max(0.0)),
            Vec2::new(-stride * step, 0.85 * step.min(0.0)),
            Vec2::ZERO,
            Vec2::ZERO,
        ],
        weapon: -arm[1] * step * 0.9,
        cloth: side * 0.13 + 0.18,
        ..neutral
    };
    neutral.blend(pose, weight)
}

pub(super) fn street_moving(t: f32, distance: f32) -> Pose {
    let mut pose = moving(Role::Pawn, t, distance);
    let envelope = ease(t * 7.0).min(ease((1.0 - t) * 6.0));
    let step = (t * TAU * (distance * 1.02).clamp(0.90, 3.6)).sin();
    pose.body.z = 0.065 * step * envelope;
    pose.head.z = -0.035 * step * envelope;
    pose.arms[1] *= 0.38;
    pose.weapon = -pose.arms[1].x;
    pose
}

/// Seconds are shared with the projectile profile, so each recoil matches a shot.
pub(super) fn street_attack(seconds: f32) -> Pose {
    let timing = crate::projectiles::street_profile();
    let neutral = Pose::default();
    let ready = Pose {
        body: Vec3::new(0.035, 0.0, 0.0),
        head: Vec3::new(-0.025, 0.0, 0.0),
        arms: [Vec3::new(0.15, 0.10, 0.10), Vec3::new(1.06, 0.0, 0.12)],
        weapon: -1.215,
        ..neutral
    };
    let final_shot = timing.delay + (timing.count - 1) as f32 * timing.interval;
    let recovery_start = final_shot + 0.12;
    let strength =
        ease(seconds / (timing.delay - 0.035)) * (1.0 - ease((seconds - recovery_start) / 0.28));
    let mut pose = neutral.blend(ready, strength);
    for shot in 0..timing.count {
        let age = seconds - timing.delay - shot as f32 * timing.interval;
        if (0.0..0.11).contains(&age) {
            let kick = (PI * age / 0.11).sin() * if shot == 2 { 1.15 } else { 1.0 };
            pose.offset.z += 0.014 * kick;
            pose.arms[1].x += 0.065 * kick;
            pose.weapon -= 0.065 * kick;
            pose.body.x -= 0.025 * kick;
        }
    }
    pose
}

pub(super) fn yielding(role: Role, direction: Vec3, amount: f32) -> Pose {
    let (duck, shoulder, head_turn, left_arm, right_arm, weapon, cloth) = match role {
        Role::Pawn => (
            0.11,
            0.20,
            0.34,
            Vec3::new(0.67, 0.29, 0.45),
            Vec3::new(0.14, -0.17, 0.34),
            -0.20,
            0.00,
        ),
        Role::Knight => (0.05, 0.12, 0.53, Vec3::ZERO, Vec3::ZERO, 0.00, 0.30),
        Role::Bishop => (
            0.065,
            0.12,
            0.30,
            Vec3::new(0.32, -0.22, 0.67),
            Vec3::new(0.27, 0.15, 0.66),
            -0.89,
            0.19,
        ),
        Role::Rook => (
            0.13,
            0.25,
            0.23,
            Vec3::new(0.54, 0.25, 0.63),
            Vec3::new(-0.18, -0.12, 0.42),
            -0.40,
            0.00,
        ),
        Role::Queen => (
            0.025,
            0.31,
            0.56,
            Vec3::new(-0.30, -0.42, 0.54),
            Vec3::new(0.33, 0.26, 0.48),
            -0.54,
            0.58,
        ),
        Role::King => (
            0.045,
            0.10,
            0.39,
            Vec3::new(0.11, 0.17, 0.73),
            Vec3::new(0.61, -0.29, 0.83),
            -1.46,
            0.35,
        ),
    };
    let step = (PI * amount).sin();
    let crouch = (duck / 0.22_f32).sqrt() * amount;
    let front = Vec2::new(
        (0.21 - direction.z * 0.28) * step + crouch,
        -0.48 * step - 1.85 * crouch,
    );
    let back = Vec2::new(
        (-0.15 - direction.z * 0.18) * step + 0.81 * crouch,
        -0.27 * step - 1.69 * crouch,
    );
    let hips = if direction.x < 0.0 {
        [front, back]
    } else {
        [back, front]
    };
    Pose {
        offset: Vec3::Y * (0.06 * step - duck * amount),
        body: Vec3::new(
            (0.17 - 0.12 * direction.z) * amount,
            -0.13 * direction.x * amount,
            -shoulder * direction.x * amount,
        ),
        head: Vec3::new(
            0.18 * amount,
            head_turn * direction.x * amount,
            shoulder * direction.x * amount * 0.60,
        ),
        arms: [left_arm * amount, right_arm * amount],
        arm_yaw: [0.0; 2],
        legs: [hips[0], hips[1], -hips[1], -hips[0]],
        weapon: weapon * amount,
        weapon_yaw: 0.0,
        cloth: cloth * amount,
        extension: if role == Role::Rook {
            ease(amount / 0.65)
        } else {
            0.0
        },
        hatch: 0.0,
        leg_extension: 0.0,
        vehicle: VehiclePose::default(),
    }
}

pub(super) fn attacking(role: Role, t: f32) -> Pose {
    let neutral = Pose::default();
    let (windup, strike) = match role {
        // The revolver already faces forward; counter the raised arm's pitch
        // while the infantry pawn braces to fire.
        Role::Pawn => (
            Pose {
                body: Vec3::new(0.04, -0.20, -0.06),
                arms: [Vec3::new(0.30, 0.30, 0.58), Vec3::new(0.43, 0.22, 0.75)],
                weapon: -1.22,
                ..neutral
            },
            Pose {
                offset: Vec3::new(0.0, -0.055, -0.08),
                body: Vec3::new(-0.20, 0.10, 0.03),
                arms: [Vec3::new(0.72, 0.46, 0.33), Vec3::new(0.96, 0.19, 0.38)],
                legs: [
                    Vec2::new(0.31, -0.50),
                    Vec2::new(-0.33, -0.18),
                    Vec2::ZERO,
                    Vec2::ZERO,
                ],
                weapon: -1.14,
                ..neutral
            },
        ),
        // A horse rears, then strikes with both front hooves.
        Role::Knight => (
            Pose {
                offset: Vec3::new(0.0, 0.19, 0.13),
                body: Vec3::new(0.72, 0.0, 0.04),
                head: Vec3::new(-0.55, 0.0, 0.0),
                legs: [
                    Vec2::new(0.77, -1.35),
                    Vec2::new(0.95, -1.38),
                    Vec2::new(-0.12, 0.24),
                    Vec2::new(-0.16, 0.29),
                ],
                cloth: 0.45,
                ..neutral
            },
            Pose {
                offset: Vec3::new(0.0, 0.23, -0.17),
                body: Vec3::new(0.28, 0.0, 0.0),
                head: Vec3::new(-0.22, 0.0, 0.0),
                legs: [
                    Vec2::new(1.12, -0.32),
                    Vec2::new(0.93, -0.13),
                    Vec2::new(-0.28, 0.17),
                    Vec2::new(-0.18, 0.22),
                ],
                cloth: -0.35,
                ..neutral
            },
        ),
        // The bishop raises the crozier and opens the casting hand.
        Role::Bishop => (
            Pose {
                body: Vec3::new(0.08, -0.21, -0.11),
                head: Vec3::new(0.19, 0.0, 0.06),
                arms: [Vec3::new(0.58, -0.24, 1.06), Vec3::new(0.64, 0.28, 0.81)],
                weapon: -0.78,
                cloth: -0.17,
                ..neutral
            },
            Pose {
                offset: Vec3::new(0.0, 0.06, -0.04),
                body: Vec3::new(-0.14, 0.09, 0.0),
                arms: [Vec3::new(1.12, 0.36, 0.45), Vec3::new(1.31, -0.34, 0.62)],
                weapon: -1.78,
                cloth: 0.23,
                ..neutral
            },
        ),
        Role::Rook => return rook_attack(t),
        // The queen draws back and sweeps the rapier across her body.
        Role::Queen => (
            Pose {
                body: Vec3::new(0.15, -0.65, 0.18),
                arms: [Vec3::new(-0.28, -0.37, 0.54), Vec3::new(1.19, 0.74, 0.75)],
                weapon: -0.48,
                ..neutral
            },
            Pose {
                offset: Vec3::new(0.05, 0.06, -0.07),
                body: Vec3::new(-0.24, 0.68, -0.20),
                arms: [Vec3::new(0.47, 0.44, 0.46), Vec3::new(1.51, -0.67, 0.37)],
                weapon: -2.12,
                ..neutral
            },
        ),
        // The king lifts the scepter and gives a broad command gesture.
        Role::King => (
            Pose {
                body: Vec3::new(0.15, -0.18, 0.06),
                head: Vec3::new(0.15, 0.0, 0.0),
                arms: [Vec3::new(0.48, -0.47, 1.14), Vec3::new(1.47, 0.21, 1.44)],
                weapon: -1.15,
                cloth: -0.17,
                ..neutral
            },
            Pose {
                offset: Vec3::new(0.0, 0.07, -0.05),
                body: Vec3::new(-0.25, 0.14, -0.07),
                arms: [Vec3::new(1.32, 0.51, 0.27), Vec3::new(1.71, -0.59, 0.22)],
                weapon: -1.91,
                cloth: 0.44,
                ..neutral
            },
        ),
    };
    keyframes(
        t,
        &[
            (0.0, neutral),
            (0.21, windup),
            (0.43, strike),
            (0.65, strike.blend(neutral, 0.25)),
            (1.0, neutral),
        ],
    )
}

fn rook_attack(t: f32) -> Pose {
    let hatch =
        crate::projectiles::rook_hatch_fraction(t * crate::projectiles::ROOK_SEQUENCE_SECONDS);
    Pose {
        hatch,
        offset: Vec3::Y * (0.012 * (PI * hatch).sin()),
        ..default()
    }
}

/// A held surrender pose. All roles share the same clock, including the horse
/// folding its forelegs and the tower extending brick limbs before kneeling.
pub(super) fn kneeling(role: Role, t: f32) -> Pose {
    let neutral = Pose::default();
    let down = if role == Role::Knight {
        Pose {
            offset: Vec3::new(0.0, -0.27, 0.015),
            body: Vec3::new(-0.18, 0.0, 0.0),
            head: Vec3::new(-0.28, 0.0, 0.0),
            legs: [
                Vec2::new(1.20, -2.50),
                Vec2::new(1.20, -2.50),
                Vec2::new(-0.50, 1.90),
                Vec2::new(-0.50, 1.90),
            ],
            cloth: -0.12,
            ..neutral
        }
    } else {
        Pose {
            offset: Vec3::new(
                0.0,
                match role {
                    Role::Queen | Role::Rook => -0.30,
                    Role::King | Role::Bishop => -0.255,
                    _ => -0.245,
                },
                0.015,
            ),
            body: Vec3::new(-0.14, 0.0, 0.0),
            head: Vec3::new(-0.28, 0.0, 0.0),
            arms: [Vec3::new(0.64, -0.08, 0.58), Vec3::new(0.29, 0.08, 0.42)],
            legs: [
                Vec2::new(1.69, -1.55),
                Vec2::new(0.14, -1.58),
                Vec2::ZERO,
                Vec2::ZERO,
            ],
            weapon: 0.18,
            cloth: if role == Role::King { 0.45 } else { 0.0 },
            extension: if role == Role::Rook { 1.0 } else { 0.0 },
            ..neutral
        }
    };
    if role == Role::Rook {
        keyframes(
            t,
            &[
                (0.0, neutral),
                (
                    0.22,
                    Pose {
                        extension: 1.0,
                        ..neutral
                    },
                ),
                (1.0, down),
            ],
        )
    } else {
        neutral.blend(down, ease(t))
    }
}

pub(super) fn captured(role: Role, t: f32) -> Pose {
    let neutral = Pose::default();
    let (recoil, fall) = match role {
        Role::Pawn => (Vec3::new(0.44, -0.27, 0.23), Vec3::new(1.35, -0.22, 0.42)),
        Role::Knight => (Vec3::new(0.17, -0.18, -0.46), Vec3::new(0.01, -0.15, -1.38)),
        Role::Bishop => (Vec3::new(-0.30, 0.32, 0.19), Vec3::new(-1.09, 0.19, 0.70)),
        Role::Rook => (Vec3::new(0.30, 0.19, -0.32), Vec3::new(1.46, 0.03, -0.09)),
        Role::Queen => (Vec3::new(0.13, -0.68, -0.34), Vec3::new(0.39, -0.72, -1.36)),
        Role::King => (Vec3::new(-0.31, 0.05, 0.17), Vec3::new(-0.68, 0.26, 0.80)),
    };
    let stagger = Pose {
        offset: Vec3::new(0.0, -0.04, 0.10),
        body: recoil,
        head: Vec3::new(-0.29, 0.0, -recoil.z * 0.60),
        arms: [Vec3::new(0.92, -0.36, 0.53), Vec3::new(-0.63, 0.37, 0.44)],
        legs: [
            Vec2::new(0.32, -0.51),
            Vec2::new(-0.31, -0.15),
            Vec2::new(0.24, 0.45),
            Vec2::new(-0.12, 0.32),
        ],
        cloth: 0.36,
        extension: if role == Role::Rook { 0.74 } else { 0.0 },
        ..neutral
    };
    let down = Pose {
        offset: Vec3::new(
            fall.z * 0.10,
            match role {
                Role::Rook => -0.14,
                Role::King => -0.23,
                _ => -0.33,
            },
            0.17,
        ),
        body: fall,
        head: Vec3::new(0.12, -0.29, 0.31),
        arms: [Vec3::new(0.35, -0.77, 0.95), Vec3::new(-0.82, 0.69, 0.40)],
        legs: [
            Vec2::new(0.69, -1.03),
            Vec2::new(-0.31, -0.72),
            Vec2::new(-0.52, 0.93),
            Vec2::new(-0.31, 0.87),
        ],
        weapon: 0.52,
        weapon_yaw: 0.0,
        cloth: -0.25,
        extension: if role == Role::Rook { 0.10 } else { 0.0 },
        hatch: 0.0,
        leg_extension: 0.0,
        vehicle: VehiclePose::default(),
        arm_yaw: [0.0; 2],
    };
    let buckle = Pose {
        body: recoil.lerp(fall, 0.56),
        offset: down.offset * 0.56,
        ..stagger.blend(down, 0.56)
    };
    keyframes(
        t,
        &[
            (0.0, neutral),
            (0.18, stagger),
            (0.47, buckle),
            (0.85, down),
            (1.0, down),
        ],
    )
}

pub(super) fn heading(from: Vec3, to: Vec3) -> Quat {
    let delta = to - from;
    Quat::from_rotation_y((-delta.x).atan2(-delta.z))
}

pub(super) fn turn_fraction(t: f32) -> f32 {
    ease(t * 6.0).min(ease((1.0 - t) * 5.0))
}

/// Patrol stride keeps the taser low and level instead of swinging the weapon.
pub(super) fn police_moving(t: f32, distance: f32) -> Pose {
    let mut pose = moving(Role::Bishop, t, distance);
    pose.arms[1] *= 0.25;
    pose.weapon = -pose.arms[1].x - pose.arms[1].z;
    pose.body.z *= 0.45;
    pose
}
pub(super) fn police_attack(seconds: f32, flight: f32) -> Pose {
    let ready = Pose {
        body: Vec3::new(0.025, 0.0, 0.0),
        head: Vec3::new(-0.025, 0.0, 0.0),
        arms: [Vec3::new(0.65, 0.15, 2.0), Vec3::new(1.02, 0.0, 0.13)],
        weapon: -1.175,
        ..default()
    };
    let recovery = 0.34 + flight + 0.42;
    let strength = ease(seconds / 0.29) * (1.0 - ease((seconds - recovery) / 0.26));
    let mut pose = Pose::default().blend(ready, strength);
    let age = seconds - 0.34;
    if (0.0..0.09).contains(&age) {
        let kick = (PI * age / 0.09).sin();
        pose.offset.z += 0.008 * kick;
        pose.arms[1].x += 0.035 * kick;
        pose.weapon -= 0.035 * kick;
    }
    pose
}
pub(super) fn police_tased(role: Role, age: f32) -> Pose {
    if age < 0.42 {
        let shake = (age * TAU * 19.0).sin() * (PI * (age / 0.42).clamp(0.0, 1.0)).sin();
        if matches!(role, Role::Knight | Role::Rook) {
            return Pose {
                offset: Vec3::X * shake * 0.012,
                ..default()
            };
        }
        return Pose {
            body: Vec3::new(-0.06, 0.0, shake * 0.025),
            head: Vec3::new(-0.08, shake * 0.035, 0.0),
            arms: [Vec3::new(0.24, -0.10, 0.16), Vec3::new(0.24, 0.10, 0.16)],
            offset: Vec3::X * shake * 0.008,
            ..default()
        };
    }
    let t = ((age - 0.42) / 0.46).min(1.0);
    match role {
        Role::Knight => lowrider_disabled(t),
        Role::Rook => mansion_disabled(t),
        _ => captured(role, t),
    }
}

/// The bind pose carries the shotgun diagonally against the shoulder.
pub(super) fn street_king_moving(t: f32, distance: f32) -> Pose {
    let mut pose = moving(Role::King, t, distance);
    pose.arms[1] = Vec3::ZERO;
    pose.arm_yaw[1] = 0.0;
    pose.weapon = 0.0;
    pose.body *= 0.40;
    pose.head *= 0.50;
    pose
}
pub(super) fn street_king_attack(seconds: f32, aim: f32) -> Pose {
    let ready = super::model::king_support(Pose {
        offset: Vec3::new(0.0, -0.012, 0.0),
        body: Vec3::new(0.025, 0.0, 0.0),
        head: Vec3::new(-0.035, 0.0, 0.0),
        arms: [Vec3::ZERO, Vec3::new(0.0, 0.0, 0.20)],
        weapon: 0.961 - aim,
        weapon_yaw: PI,
        legs: [
            Vec2::new(0.10, -0.13),
            Vec2::new(-0.04, -0.05),
            Vec2::ZERO,
            Vec2::ZERO,
        ],
        ..default()
    });
    let strength = ease((seconds - 0.08) / 0.48) * (1.0 - ease((seconds - 1.04) / 0.40));
    let mut pose = Pose::default().blend(ready, strength);
    let age = seconds - crate::projectiles::KING_SHOTGUN_LAUNCH;
    if (0.0..0.18).contains(&age) {
        let kick = (PI * age / 0.18).sin();
        pose.offset.z += 0.040 * kick;
        pose.body.x -= 0.045 * kick;
        pose.head.x -= 0.040 * kick;
        pose.arms[1].x += 0.06 * kick;
        pose.weapon -= 0.08 * kick;
    }
    if strength > 0.999 {
        pose = super::model::king_support(pose);
    }
    pose
}
pub(super) fn street_king_kneeling(t: f32) -> Pose {
    let mut pose = kneeling(Role::King, t);
    pose.arms[1] = Vec3::ZERO;
    pose.weapon = 0.0;
    pose
}
pub(super) fn street_king_yielding(direction: Vec3, amount: f32) -> Pose {
    let mut pose = yielding(Role::King, direction, amount);
    pose.arms[1] = Vec3::ZERO;
    pose.weapon = 0.0;
    pose.weapon_yaw = 0.0;
    pose
}

pub(super) fn street_queen_moving(t: f32, distance: f32) -> Pose {
    let mut pose = moving(Role::Queen, t, distance);
    pose.arms[0] *= 0.18;
    pose.arms[1] *= 0.22;
    pose.weapon = -pose.arms[1].x - pose.arms[1].z;
    pose.body *= 0.45;
    pose.head *= 0.55;
    pose.legs[0] *= 0.62;
    pose.legs[1] *= 0.62;
    pose.cloth = 0.055 * (t * TAU * 2.0).sin() * ease(t * 7.0) * ease((1.0 - t) * 6.0);
    pose
}
pub(super) fn street_queen_attack(seconds: f32, distance: f32, aim: f32) -> Pose {
    let launch = crate::projectiles::QUEEN_SMG_LAUNCH;
    let interval = crate::projectiles::QUEEN_SMG_INTERVAL;
    let last = launch + (crate::projectiles::QUEEN_SMG_ROUNDS - 1) as f32 * interval;
    let phase = ((seconds - launch) / interval)
        .clamp(0.0, (crate::projectiles::QUEEN_SMG_ROUNDS - 1) as f32);
    let spread = crate::projectiles::queen_smg_spread(phase);
    let ready = Pose {
        offset: Vec3::new(0.0, -0.008, 0.0),
        body: Vec3::new(0.035, 0.0, 0.0),
        head: Vec3::new(-0.025, 0.0, 0.0),
        arms: [Vec3::new(0.06, 0.04, 0.04), Vec3::new(1.0, 0.0, 0.12)],
        arm_yaw: [0.0, -spread.x.atan2((distance - 0.35).max(0.4))],
        weapon: -0.77 + aim,
        legs: [
            Vec2::new(0.06, -0.10),
            Vec2::new(-0.04, -0.06),
            Vec2::ZERO,
            Vec2::ZERO,
        ],
        cloth: 0.025,
        ..default()
    };
    let strength = ease(seconds / 0.38) * (1.0 - ease((seconds - last - 0.16) / 0.30));
    let mut pose = Pose::default().blend(ready, strength);
    for shot in 0..crate::projectiles::QUEEN_SMG_ROUNDS {
        let age = seconds - launch - shot as f32 * interval;
        if (0.0..0.055).contains(&age) {
            let kick = (PI * age / 0.055).sin();
            pose.offset.z += 0.011 * kick;
            pose.body.x -= 0.016 * kick;
            pose.arms[1].x += 0.030 * kick;
            pose.weapon -= 0.015 * kick;
        }
    }
    pose
}
