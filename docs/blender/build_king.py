"""Build BattleChess's king, including his crowned head and articulated cape."""

import math
import sys
from pathlib import Path

import numpy as np


PROJECT_ROOT = Path(__file__).resolve().parents[2]
OUTPUT = PROJECT_ROOT / 'art' / 'king'
STL_OUTPUT = PROJECT_ROOT / 'docs' / 'stl'
OUTPUT.mkdir(parents=True, exist_ok=True)
STL_OUTPUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(Path(__file__).resolve().parent))
from figure_mesh import Sculpt, curved_sections, ease, export_figure, soft_peak

# Body, head, scepter, arms and legs, followed by the cape at the shoulders.
REST = [
    (0, 0.573, 0.013), (0, 1.083, 0.001), (0.322, 0.628, -0.130),
    (-0.217, 0.985, 0.011), (-0.268, 0.776, 0.008),
    (0.217, 0.985, 0.011), (0.270, 0.790, -0.017),
    (-0.115, 0.564, 0.020), (-0.128, 0.313, -0.010),
    (0.115, 0.564, 0.020), (0.126, 0.316, -0.016),
    (0, 0.982, 0.147),
]


def tunic_relief(x, y, z, angle):
    cos, sin = math.cos(angle), math.sin(angle)
    hem = 0.012 * soft_peak(y, 0.487, 0.025)
    belt = 0.021 * soft_peak(y, 0.652, 0.018)
    drape = 0.009 * soft_peak(y, 0.520, 0.135) * math.cos(9 * angle + 4 * y)
    front = max(0, -sin) ** 2
    seam = front * 0.012 * soft_peak(x, 0, 0.025) * soft_peak(y, 0.83, 0.213)
    return x + cos * (hem + belt + drape), z + sin * (hem + belt + drape) - seam


def build_body(sculpt):
    sculpt.loft(curved_sections([
        (0.450, 0, 0.022, 0.169, 0.106), (0.482, 0, 0.024, 0.199, 0.122),
        (0.540, 0, 0.025, 0.185, 0.116), (0.605, 0, 0.024, 0.166, 0.106),
        (0.661, 0, 0.014, 0.171, 0.109), (0.736, 0, 0.014, 0.188, 0.113),
        (0.841, 0, 0.010, 0.215, 0.133), (0.938, 0, 0.009, 0.225, 0.143),
        (1.003, 0, 0.017, 0.215, 0.124), (1.045, 0, 0.014, 0.184, 0.102),
        (1.076, 0, 0.007, 0.107, 0.075), (1.097, 0, 0.001, 0.063, 0.060),
    ]), sides=72, relief=tunic_relief)

    # A raised clasp secures the mantle and a broad clasp sits on the belt.
    for x, y, z, width in [(-0.094, 0.992, -0.123, 0.038), (0, 0.655, -0.131, 0.047)]:
        sculpt.loft([
            (y - 0.027, x, z, width * 0.68, 0.021),
            (y - 0.013, x, z - 0.007, width, 0.020),
            (y + 0.016, x, z - 0.007, width, 0.019),
            (y + 0.028, x, z, width * 0.66, 0.017),
        ], sides=16)

    for side in (-1, 1):
        foot = side * 0.130
        sculpt.loft(curved_sections([
            (0, foot, -0.047, 0.054, 0.097), (0.020, foot, -0.048, 0.063, 0.103),
            (0.056, foot, -0.047, 0.062, 0.095), (0.102, foot, -0.021, 0.059, 0.067),
            (0.155, foot, -0.004, 0.054, 0.054), (0.212, foot, 0.001, 0.059, 0.059),
            (0.236, foot, 0.000, 0.066, 0.067), (0.251, foot, 0.000, 0.062, 0.066),
            (0.316, side * 0.126, -0.014, 0.063, 0.071),
            (0.410, side * 0.119, 0.008, 0.074, 0.079),
            (0.497, side * 0.115, 0.022, 0.078, 0.083),
            (0.559, side * 0.114, 0.023, 0.079, 0.082),
        ]), sides=36)

        right = side > 0
        front = -0.102 if right else -0.020
        sculpt.loft(curved_sections([
            (0.623, side * 0.282, front, 0.039, 0.042),
            (0.645, side * 0.281, front, 0.047, 0.046),
            (0.667, side * 0.278, front * 0.8, 0.048, 0.044),
            (0.718, side * 0.272, front * 0.54, 0.048, 0.056),
            (0.785, side * 0.266, front * 0.23, 0.054, 0.058),
            (0.852, side * 0.249, 0.007, 0.062, 0.065),
            (0.940, side * 0.226, 0.013, 0.071, 0.075),
            (0.995, side * 0.202, 0.010, 0.077, 0.073),
            (1.028, side * 0.182, 0.010, 0.063, 0.061),
        ]), sides=36)
        x = 0.301 if right else 0.286
        sculpt.loft([
            (0.567, side * x, front - 0.009, 0.023, 0.027),
            (0.585, side * x, front - 0.006, 0.035, 0.033),
            (0.614, side * (x - 0.003), front - 0.005, 0.037, 0.037),
            (0.638, side * 0.288, front, 0.031, 0.035),
            (0.653, side * 0.282, front, 0.027, 0.032),
        ])
        sculpt.tube([
            ((side * 0.272, 0.634, front - 0.032), 0.015),
            ((side * (x - 0.012), 0.617, front - 0.039), 0.016),
            ((side * x, 0.600, front - 0.034), 0.011),
        ])

    # The turned edge of the mantle appears on both shoulders.
    for side in (-1, 1):
        sculpt.tube([
            ((side * 0.052, 1.067, 0.006), 0.029),
            ((side * 0.108, 1.042, -0.043), 0.034),
            ((side * 0.153, 1.007, -0.083), 0.033),
            ((side * 0.193, 0.973, -0.058), 0.033),
            ((side * 0.229, 0.965, 0.012), 0.034),
            ((side * 0.242, 0.966, 0.072), 0.029),
            ((side * 0.236, 0.972, 0.132), 0.027),
        ], sides=18)


def build_cape(sculpt):
    """Make one thick folded panel attached across the back and shoulders."""
    rows = [
        (0.247, 0.309, 0.326, 0.068, 0.020),
        (0.282, 0.328, 0.324, 0.078, 0.022),
        (0.400, 0.318, 0.303, 0.081, 0.021),
        (0.547, 0.301, 0.267, 0.081, 0.019),
        (0.688, 0.278, 0.232, 0.080, 0.018),
        (0.814, 0.266, 0.198, 0.081, 0.018),
        (0.929, 0.259, 0.157, 0.070, 0.019),
        (0.986, 0.238, 0.128, 0.049, 0.023),
        (1.027, 0.161, 0.095, 0.036, 0.022),
        (1.064, 0.088, 0.053, 0.020, 0.019),
    ]
    rings = []
    for y, width, back, wrap, thick in curved_sections(rows):
        outside, inside = [], []
        for t in np.linspace(-1, 1, 51):
            flare = ease(1.035, 0.35, y)
            drape = (0.022 * math.cos(3 * math.pi * t + y * 1.5) +
                     0.009 * math.cos(5 * math.pi * t - 2 * y)) * flare
            z = back - wrap * abs(t) ** 2 + drape
            edge = 0.018 * abs(t) ** 14
            y_point = y + soft_peak(y, 0.25, 0.09) * (0.023 * abs(t) ** 2 + 0.016 * math.cos(2 * math.pi * t))
            x = t * width
            outside.append((x, y_point, z + thick + edge))
            inside.append((x, y_point, z - thick))
        rings.append(outside + inside[::-1])
    sculpt.rings(rings)


def face_relief(x, y, z, angle):
    front = max(0, -math.sin(angle)) ** 5
    brows = 0.014 * soft_peak(abs(x), 0.047, 0.029) * soft_peak(y, 1.301, 0.014)
    sockets = 0.016 * soft_peak(abs(x), 0.046, 0.026) * soft_peak(y, 1.283, 0.011)
    nose = 0.033 * soft_peak(x, 0, 0.024) * soft_peak(y, 1.267, 0.033)
    return x, z + front * (sockets - brows - nose)


def build_head(sculpt):
    sculpt.loft([
        (1.056, 0, 0.007, 0.069, 0.064), (1.096, 0, 0.003, 0.061, 0.057),
        (1.147, 0, -0.001, 0.061, 0.059), (1.188, 0, -0.001, 0.068, 0.065),
    ])
    sculpt.loft(curved_sections([
        (1.124, 0, -0.019, 0.046, 0.059), (1.161, 0, -0.019, 0.074, 0.077),
        (1.211, 0, -0.009, 0.090, 0.083), (1.247, 0, -0.003, 0.094, 0.079),
        (1.281, 0, -0.004, 0.092, 0.080), (1.317, 0, 0.005, 0.094, 0.082),
        (1.352, 0, 0.013, 0.094, 0.085), (1.411, 0, 0.021, 0.068, 0.070),
    ]), sides=64, relief=face_relief)

    # A shaped beard ends beneath the chin, with temples and hair above it.
    sculpt.loft(curved_sections([
        (1.107, 0, -0.053, 0.038, 0.031), (1.129, 0, -0.059, 0.063, 0.044),
        (1.161, 0, -0.044, 0.084, 0.067), (1.197, 0, -0.033, 0.092, 0.073),
        (1.220, 0, -0.031, 0.091, 0.070), (1.234, 0, -0.027, 0.085, 0.063),
    ]), sides=52, relief=lambda x, y, z, a: (x, z + max(0, -math.sin(a)) * 0.004 * math.cos(19 * a - 8 * y)))
    for side in (-1, 1):
        sculpt.tube([
            ((side * 0.007, 1.237, -0.091), 0.012),
            ((side * 0.028, 1.233, -0.095), 0.016),
            ((side * 0.050, 1.224, -0.079), 0.013),
            ((side * 0.064, 1.211, -0.067), 0.009),
        ])
    sculpt.loft(curved_sections([
        (1.185, 0, 0.056, 0.071, 0.034), (1.222, 0, 0.056, 0.097, 0.048),
        (1.278, 0, 0.050, 0.107, 0.054), (1.337, 0, 0.043, 0.103, 0.074),
        (1.382, 0, 0.021, 0.099, 0.083), (1.424, 0, 0.020, 0.068, 0.070),
    ]), sides=48, relief=lambda x, y, z, a: (x, z + 0.006 * max(0, math.sin(a)) * math.cos(13 * a + 13 * y)))


def build_crown(sculpt):
    # A closed crown with a raised circlet, four arches, an orb, and a cross.
    z = 0.015
    sculpt.loft(curved_sections([
        (1.352, 0, z, 0.094, 0.086), (1.366, 0, z, 0.126, 0.114),
        (1.382, 0, z, 0.128, 0.119), (1.400, 0, z, 0.124, 0.115),
        (1.422, 0, z, 0.132, 0.120), (1.443, 0, z, 0.137, 0.125),
        (1.455, 0, z, 0.127, 0.120), (1.478, 0, z, 0.113, 0.105),
        (1.520, 0, z, 0.089, 0.086), (1.558, 0, z, 0.064, 0.058),
        (1.583, 0, z, 0.031, 0.032), (1.599, 0, z, 0.017, 0.018),
    ]), sides=64)
    for index in range(4):
        angle = math.tau * index / 4 + math.pi / 4
        cos, sin = math.cos(angle), math.sin(angle)
        sculpt.tube([
            ((radius * cos, y, z + radius * sin), width)
            for radius, y, width in [
                (0.114, 1.418, 0.019), (0.124, 1.457, 0.023),
                (0.119, 1.498, 0.021), (0.094, 1.541, 0.020),
                (0.060, 1.579, 0.017), (0.026, 1.602, 0.016),
                (0.003, 1.608, 0.013),
            ]
        ], sides=18)
    for index in range(8):
        angle = math.tau * index / 8
        gem_x, gem_z = 0.120 * math.cos(angle), z + 0.112 * math.sin(angle)
        sculpt.loft([
            (1.381, gem_x, gem_z, 0.013, 0.013),
            (1.398, gem_x, gem_z, 0.023, 0.023),
            (1.418, gem_x, gem_z, 0.014, 0.015),
        ], sides=14)
    sculpt.loft([
        (1.585, 0, z, 0.018, 0.018), (1.604, 0, z, 0.032, 0.030),
        (1.631, 0, z, 0.033, 0.031), (1.652, 0, z, 0.021, 0.021),
    ])

    outline = [
        (-0.015, 1.640), (0.015, 1.640), (0.020, 1.687), (0.057, 1.687),
        (0.063, 1.704), (0.057, 1.724), (0.020, 1.717), (0.019, 1.760),
        (0.011, 1.772), (-0.011, 1.772), (-0.019, 1.760), (-0.020, 1.717),
        (-0.057, 1.724), (-0.063, 1.704), (-0.057, 1.687), (-0.020, 1.687),
    ]
    sculpt.rings([
        [(x * scale, 1.70 + (y - 1.70) * scale, dz + z) for x, y in outline]
        for dz, scale in [(0.025, 0.88), (0.018, 1), (-0.018, 1), (-0.025, 0.88)]
    ])


def build_scepter(sculpt):
    x, z = 0.322, -0.131
    sculpt.loft(curved_sections([
        (0.505, x, z, 0.013, 0.013), (0.527, x, z, 0.025, 0.026),
        (0.554, x, z, 0.025, 0.025), (0.579, x, z, 0.019, 0.019),
        (0.619, x, z, 0.018, 0.018), (0.943, x, z, 0.018, 0.018),
        (0.985, x, z, 0.018, 0.018), (1.079, x, z, 0.020, 0.020),
        (1.104, x, z, 0.038, 0.037), (1.128, x, z, 0.037, 0.038),
        (1.155, x, z, 0.054, 0.052), (1.190, x, z, 0.057, 0.056),
        (1.224, x, z, 0.050, 0.050), (1.253, x, z, 0.036, 0.035),
        (1.273, x, z, 0.018, 0.018), (1.282, x, z, 0.009, 0.009),
    ]), sides=36, relief=lambda px, y, pz, a: (
        px + 0.008 * math.cos(a) * soft_peak(y, 1.199, 0.010),
        pz + 0.008 * math.sin(a) * soft_peak(y, 1.199, 0.010),
    ))


def skin_weights(positions):
    x, y, z = positions.T
    weights = np.zeros((len(x), len(REST)), dtype=np.float32)
    hand = ease(0.271, 0.315, x)
    orb = ease(0.251, 0.266, x) * ease(1.093, 1.126, y)
    scepter = np.maximum(hand, orb) * (1 - ease(-0.087, -0.061, z))
    weights[:, 2] = scepter
    weights[:, 1] = ease(1.050, 1.153, y) * (1 - scepter)
    back_edge = np.interp(y, [0.25, 0.56, 0.85, 1.04], [0.156, 0.152, 0.136, 0.121])
    cape = 0.72 * ease(back_edge, back_edge + 0.043, z) * (1 - ease(0.844, 1.035, y))
    free = 1 - weights[:, 1] - scepter
    weights[:, 11] = free * cape
    free -= weights[:, 11]

    inner = np.interp(y, [0.60, 0.75, 0.94, 1.05], [0.213, 0.198, 0.188, 0.177])
    arm = free * ease(inner, inner + 0.062, abs(x)) * ease(0.526, 0.575, y)
    arm *= (1 - ease(1.017, 1.066, y)) * (1 - ease(0.065, 0.152, z))
    forearm = 1 - ease(0.733, 0.837, y)
    leg = (free - arm) * (1 - ease(0.472, 0.603, y)) * (1 - ease(0.107, 0.163, z))
    shin = 1 - ease(0.247, 0.362, y)
    for shoulder, hip, side in [(3, 7, x < 0), (5, 9, x >= 0)]:
        weights[:, shoulder] = side * arm * (1 - forearm)
        weights[:, shoulder + 1] = side * arm * forearm
        weights[:, hip] = side * leg * (1 - shin)
        weights[:, hip + 1] = side * leg * shin
    weights[:, 0] = np.maximum(0, free - arm - leg)
    ids = np.argsort(-weights, axis=1)[:, :4]
    weights = np.take_along_axis(weights, ids, axis=1)
    return ids, weights / weights.sum(axis=1, keepdims=True)


def build():
    sculpt = Sculpt()
    build_body(sculpt)
    build_cape(sculpt)
    build_head(sculpt)
    build_crown(sculpt)
    build_scepter(sculpt)
    export_figure(sculpt, REST, skin_weights, OUTPUT, 'king', 'Royal king', 0.0048, 0.095, stl_output=STL_OUTPUT)


if __name__ == '__main__':
    build()

