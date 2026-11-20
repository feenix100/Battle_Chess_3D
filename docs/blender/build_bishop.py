"""Sculpt the robed bishop and export the STL and Bevy skin from Blender."""

import math
import sys
from pathlib import Path

import numpy as np


PROJECT_ROOT = Path(__file__).resolve().parents[2]
OUTPUT = PROJECT_ROOT / 'art' / 'bishop'
STL_OUTPUT = PROJECT_ROOT / 'docs' / 'stl'
OUTPUT.mkdir(parents=True, exist_ok=True)
STL_OUTPUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(Path(__file__).resolve().parent))
from figure_mesh import Sculpt, curved_sections, ease, export_figure, soft_peak

# Y up, forward along -Z. Matches the pawn's biped joint order.
REST = [
    (0, 0.59, 0.018), (0, 1.027, -0.004), (0.333, 0.602, -0.103),
    (-0.182, 0.985, 0.012), (-0.254, 0.776, 0.012),
    (0.182, 0.985, 0.012), (0.251, 0.785, -0.010),
    (-0.105, 0.566, 0.035), (-0.118, 0.305, 0.016),
    (0.105, 0.566, 0.032), (0.115, 0.305, 0.012),
]


def tailored_robe(x, y, z, angle):
    cos, sin = math.cos(angle), math.sin(angle)
    # Broad hanging folds taper out above the hips.
    folds = (max(0, min(1, (0.61 - y) / 0.46)) *
             (0.012 * math.cos(9 * angle + 1.7 * y) + 0.008 * math.sin(13 * angle - 2 * y)))
    x += cos * folds
    z += sin * folds
    front = max(0, -sin) ** 3
    bust = 0.040 * soft_peak(y, 0.864, 0.085) * soft_peak(abs(x), 0.076, 0.049)
    stole_center = 0.086 + 0.095 * (y - 0.73)
    stole = 0.028 * soft_peak(abs(x), stole_center, 0.024)
    stole *= min(1, max(0, (y - 0.375) / 0.035)) * min(1, max(0, (1.008 - y) / 0.022))
    belt = 0.013 * soft_peak(y, 0.716, 0.017)
    return x + cos * belt, z + sin * belt - front * (bust + stole)


def build_robes(sculpt):
    profile = [
        (0.086, 0, 0.032, 0.246, 0.175), (0.103, 0, 0.033, 0.265, 0.189),
        (0.129, 0, 0.036, 0.267, 0.186), (0.168, 0, 0.041, 0.259, 0.181),
        (0.238, 0, 0.044, 0.241, 0.169), (0.325, 0, 0.043, 0.227, 0.158),
        (0.405, 0, 0.047, 0.219, 0.155), (0.475, 0, 0.048, 0.216, 0.151),
        (0.540, 0, 0.045, 0.211, 0.146), (0.595, 0, 0.042, 0.184, 0.128),
        (0.646, 0, 0.033, 0.160, 0.112), (0.692, 0, 0.020, 0.138, 0.097),
        (0.716, 0, 0.016, 0.136, 0.095), (0.752, 0, 0.015, 0.145, 0.110),
        (0.786, 0, 0.009, 0.161, 0.124), (0.822, 0, 0.004, 0.174, 0.138),
        (0.866, 0, 0.004, 0.177, 0.141), (0.910, 0, 0.010, 0.179, 0.130),
        (0.952, 0, 0.014, 0.185, 0.113), (0.980, 0, 0.018, 0.178, 0.096),
        (1.003, 0, 0.014, 0.134, 0.081), (1.021, 0, 0.008, 0.074, 0.067),
    ]
    sculpt.loft(curved_sections(profile), sides=96, relief=tailored_robe)

    for side in (-1, 1):
        foot = side * 0.119
        sculpt.loft([
            (0, foot, -0.046, 0.045, 0.081),
            (0.017, foot, -0.047, 0.051, 0.089),
            (0.043, foot, -0.044, 0.048, 0.085),
            (0.069, foot, -0.027, 0.045, 0.069),
            (0.111, foot, 0.016, 0.045, 0.053),
            (0.173, foot, 0.018, 0.047, 0.056),
        ])

    # Shaped sleeves open into broad cuffs, with the crozier hand resting higher.
    for side in (-1, 1):
        lift = 0.025 if side > 0 else 0.0
        forward = -0.081 if side > 0 else -0.015
        sculpt.loft([
            (0.596 + lift, side * 0.263, forward, 0.054, 0.060),
            (0.617 + lift, side * 0.266, forward, 0.058, 0.065),
            (0.638 + lift, side * 0.266, forward + 0.004, 0.056, 0.061),
            (0.686 + lift * 0.7, side * 0.261, forward * 0.63, 0.046, 0.053),
            (0.756, side * 0.253, forward * 0.35, 0.046, 0.053),
            (0.814, side * 0.241, 0.003, 0.052, 0.056),
            (0.883, side * 0.218, 0.014, 0.060, 0.068),
            (0.940, side * 0.191, 0.015, 0.067, 0.069),
            (0.969, side * 0.180, 0.014, 0.059, 0.062),
            (0.992, side * 0.168, 0.013, 0.045, 0.051),
        ], sides=40)
        palm = 0.297 if side > 0 else 0.266
        fingers = forward - 0.012
        sculpt.loft([
            (0.539 + lift, side * palm, fingers, 0.021, 0.024),
            (0.554 + lift, side * palm, fingers, 0.031, 0.032),
            (0.577 + lift, side * (palm - 0.004), fingers, 0.033, 0.036),
            (0.600 + lift, side * 0.272, forward, 0.028, 0.033),
            (0.627 + lift, side * 0.266, forward, 0.026, 0.031),
        ])
        sculpt.tube([
            ((side * 0.256, 0.608 + lift, fingers - 0.015), 0.014),
            ((side * (palm - 0.010), 0.592 + lift, fingers - 0.028), 0.015),
            ((side * palm, 0.580 + lift, fingers - 0.025), 0.010),
        ])


def facial_features(x, y, z, angle):
    front = max(0, -math.sin(angle)) ** 6
    eyes = 0.015 * soft_peak(abs(x), 0.045, 0.024) * soft_peak(y, 1.149, 0.012)
    brow = 0.011 * soft_peak(abs(x), 0.044, 0.031) * soft_peak(y, 1.169, 0.011)
    nose = 0.032 * soft_peak(x, 0, 0.021) * soft_peak(y, 1.132, 0.022)
    lips = (0.013 * soft_peak(y, 1.100, 0.010) - 0.006 * soft_peak(y, 1.099, 0.003))
    cheeks = 0.010 * soft_peak(abs(x), 0.063, 0.028) * soft_peak(y, 1.128, 0.018)
    return x, z + front * (eyes - brow - nose - cheeks - lips * soft_peak(x, 0, 0.032))


def build_head(sculpt):
    sculpt.loft([
        (0.992, 0, 0.002, 0.058, 0.062), (1.023, 0, 0.001, 0.055, 0.054),
        (1.065, 0, -0.007, 0.050, 0.053), (1.096, 0, -0.010, 0.053, 0.059),
    ])
    levels = [
        (1.056, 0, -0.018, 0.029, 0.048), (1.074, 0, -0.020, 0.047, 0.062),
        (1.102, 0, -0.016, 0.065, 0.072), (1.133, 0, -0.009, 0.081, 0.079),
        (1.151, 0, -0.004, 0.083, 0.077), (1.181, 0, 0.002, 0.086, 0.079),
        (1.217, 0, 0.010, 0.085, 0.078), (1.249, 0, 0.011, 0.067, 0.063),
    ]
    # Dense slices through the face preserve the shallow sculpted features.
    sections = []
    for start, end in zip(levels, levels[1:]):
        for i in range(5):
            sections.append(tuple(a + (b - a) * i / 5 for a, b in zip(start, end)))
    sections.append(levels[-1])
    sculpt.loft(sections, sides=64, relief=facial_features)

    # Three broad locks emerge beneath the mitre and brush the collar.
    sculpt.loft([
        (1.008, 0, 0.086, 0.062, 0.035), (1.043, 0, 0.084, 0.086, 0.046),
        (1.096, 0, 0.070, 0.100, 0.055), (1.162, 0, 0.048, 0.105, 0.068),
        (1.215, 0, 0.030, 0.104, 0.080), (1.249, 0, 0.022, 0.084, 0.078),
    ], sides=48, relief=lambda x, y, z, a: (x, z + max(0, math.sin(a)) * 0.007 * math.cos(13 * a)))
    for side in (-1, 1):
        sculpt.tube([
            ((side * 0.043, 1.235, -0.067), 0.026),
            ((side * 0.075, 1.202, -0.055), 0.031),
            ((side * 0.089, 1.164, -0.025), 0.024),
            ((side * 0.087, 1.106, -0.006), 0.026),
            ((side * 0.100, 1.059, -0.013), 0.023),
            ((side * 0.087, 1.015, -0.022), 0.010),
        ])

    # A tapered shield outline with the chess bishop's diagonal open cut.
    outline = [
        (-0.105, 1.200), (0.104, 1.200), (0.132, 1.247), (0.143, 1.303),
        (0.124, 1.368), (0.104, 1.402), (-0.022, 1.353), (-0.039, 1.357),
        (-0.046, 1.375), (-0.040, 1.388), (0.080, 1.441),
        (0.029, 1.528), (0.009, 1.555), (-0.009, 1.556), (-0.031, 1.533),
        (-0.092, 1.448), (-0.128, 1.378), (-0.146, 1.316), (-0.132, 1.248),
    ]
    sculpt.rings([
        [(x * scale, 1.32 + (y - 1.32) * scale, z) for x, y in outline]
        for z, scale in [(-0.117, 0.83), (-0.091, 0.97), (-0.036, 1), (0.058, 0.98), (0.114, 0.81)]
    ][::-1])
    sculpt.loft([
        (1.200, 0, 0.012, 0.100, 0.092), (1.212, 0, 0.009, 0.119, 0.106),
        (1.232, 0, 0.008, 0.123, 0.106), (1.246, 0, 0.007, 0.122, 0.100),
    ], sides=40)


def build_crozier(sculpt):
    sculpt.loft([
        (0.188, 0.333, -0.105, 0.012, 0.012), (0.208, 0.333, -0.105, 0.019, 0.019),
        (0.238, 0.333, -0.105, 0.020, 0.020), (0.263, 0.333, -0.105, 0.016, 0.016),
        (0.682, 0.333, -0.105, 0.017, 0.017), (1.227, 0.333, -0.105, 0.018, 0.018),
        (1.252, 0.333, -0.105, 0.026, 0.026), (1.276, 0.333, -0.105, 0.021, 0.021),
        (1.302, 0.333, -0.105, 0.019, 0.019),
    ], sides=16)
    curl = []
    for step in range(40):
        t = step / 39
        angle = 5.43 * t
        radius = 0.081 - 0.046 * t
        curl.append(((0.252 + radius * math.cos(angle), 1.287 + radius * math.sin(angle), -0.105),
                     0.021 - 0.008 * t + 0.004 * soft_peak(t, 0.94, 0.08)))
    sculpt.tube(curl, sides=16)


def skin_weights(positions):
    x, y, z = positions.T
    weights = np.zeros((len(x), len(REST)), dtype=np.float32)
    weights[:, 1] = ease(0.994, 1.065, y)
    # The staff pivots at her grip and its hook sits clear of the head.
    staff_gate = ease(0.290, 0.315, x)
    staff_gate = np.maximum(staff_gate, ease(0.141, 0.165, x) * ease(1.200, 1.235, y))
    weights[:, 2] = staff_gate * (1 - ease(-0.065, -0.050, z))
    weights[:, 1] *= 1 - weights[:, 2]

    free = 1 - weights[:, 1] - weights[:, 2]
    inner_sleeve = np.interp(y, [0.55, 0.65, 0.75, 0.86, 0.96, 1.0],
                            [0.212, 0.183, 0.177, 0.174, 0.163, 0.164])
    arms = (free * ease(inner_sleeve, inner_sleeve + 0.067, abs(x)) *
            ease(0.508, 0.560, y) * (1 - ease(0.975, 1.007, y)))
    forearm = 1 - ease(0.725, 0.818, y)

    # Let the feet drive the hem and fade to the stationary folds at the hips.
    legs = (free - arms) * np.interp(y, [0.080, 0.108, 0.245, 0.437, 0.556],
                                   [1.0, 0.92, 0.64, 0.28, 0.0])
    shin = 1 - ease(0.208, 0.358, y)
    right_leg = ease(-0.100, 0.100, x)
    for shoulder, hip, arm_side, leg_side in [
        (3, 7, x < 0, 1 - right_leg), (5, 9, x >= 0, right_leg)
    ]:
        weights[:, shoulder] = arms * arm_side * (1 - forearm)
        weights[:, shoulder + 1] = arms * arm_side * forearm
        weights[:, hip] = legs * leg_side * (1 - shin)
        weights[:, hip + 1] = legs * leg_side * shin
    weights[:, 0] = np.maximum(0, free - arms - legs)
    ids = np.argsort(-weights, axis=1)[:, :4]
    weights = np.take_along_axis(weights, ids, axis=1)
    return ids, weights / weights.sum(axis=1, keepdims=True)


def build():
    sculpt = Sculpt()
    build_robes(sculpt)
    build_head(sculpt)
    build_crozier(sculpt)

    export_figure(sculpt, REST, skin_weights, OUTPUT, 'bishop', 'Robed bishop', 0.0048, 0.095, stl_output=STL_OUTPUT)


if __name__ == '__main__':
    build()

