"""Build the queen's court gown, crown, and skinned rapier rig in Blender."""

import math
import sys
from pathlib import Path

import numpy as np


PROJECT_ROOT = Path(__file__).resolve().parents[2]
OUTPUT = PROJECT_ROOT / 'art' / 'queen'
STL_OUTPUT = PROJECT_ROOT / 'docs' / 'stl'
OUTPUT.mkdir(parents=True, exist_ok=True)
STL_OUTPUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(Path(__file__).resolve().parent))
from figure_mesh import Sculpt, curved_sections, ease, export_figure, soft_peak

# Body, head, rapier, two arms, and two legs in the game's biped joint order.
REST = [
    (0, 0.655, 0.012), (0, 1.147, -0.004), (0.302, 0.663, -0.122),
    (-0.165, 1.077, 0.005), (-0.228, 0.857, 0.019),
    (0.165, 1.077, 0.002), (0.234, 0.865, -0.006),
    (-0.109, 0.621, 0.024), (-0.106, 0.315, 0.004),
    (0.109, 0.627, 0.019), (0.109, 0.321, 0.002),
]


def gown_relief(x, y, z, angle):
    cos, sin = math.cos(angle), math.sin(angle)
    drape = min(1, max(0, (0.76 - y) / 0.47))
    folds = drape * (0.016 * math.cos(7 * angle + y * 3) + 0.008 * math.cos(11 * angle - 3 * y))
    pleats = 0.007 * soft_peak(y, 0.836, 0.113) * math.cos(10 * angle)
    waist_trim = 0.013 * soft_peak(y, 0.809, 0.018)
    embroidery = folds + pleats + waist_trim
    x += cos * embroidery
    z += sin * embroidery

    front = max(0, -sin) ** 2
    bust = 0.041 * soft_peak(y, 0.985, 0.081) * soft_peak(abs(x), 0.074, 0.054)
    neckline = 0.016 * soft_peak(y, 1.052 + 0.47 * abs(x), 0.015)
    sash = 0.025 * soft_peak(x, -0.137 + (1.103 - y) * 0.63, 0.035)
    sash *= min(1, max(0, (y - 0.64) / 0.06))
    return x, z - front * (bust + sash + neckline)


def build_gown(sculpt):
    # A flared skirt gives way to a closely fitted waist and long bodice.
    profile = [
        (0.078, 0, 0.042, 0.232, 0.175), (0.103, 0, 0.043, 0.246, 0.181),
        (0.135, 0, 0.047, 0.248, 0.182), (0.215, 0, 0.045, 0.232, 0.172),
        (0.306, 0, 0.046, 0.207, 0.159), (0.413, 0, 0.051, 0.194, 0.142),
        (0.532, 0, 0.052, 0.202, 0.141), (0.627, 0, 0.037, 0.205, 0.142),
        (0.679, 0, 0.026, 0.182, 0.126), (0.744, 0, 0.017, 0.137, 0.106),
        (0.804, 0, 0.010, 0.117, 0.089), (0.854, 0, 0.006, 0.126, 0.097),
        (0.900, 0, 0.004, 0.143, 0.118), (0.974, 0, 0.004, 0.164, 0.132),
        (1.027, 0, 0.007, 0.163, 0.119), (1.070, 0, 0.009, 0.167, 0.095),
        (1.100, 0, 0.012, 0.153, 0.081), (1.130, 0, 0.010, 0.093, 0.066),
        (1.143, 0, 0.008, 0.061, 0.059),
    ]

    def draped_skirt(x, y, z, angle):
        # The front hem clears the toes while the back gathers into a small train.
        x, z = gown_relief(x, y, z, angle)
        z += 0.048 * soft_peak(y, 0.09, 0.17) * max(0, math.sin(angle)) ** 3
        return x, z

    sculpt.loft(curved_sections(profile), sides=96, relief=draped_skirt)

    for side in (-1, 1):
        x = side * 0.104
        sculpt.loft(curved_sections([
            (0, x, -0.047, 0.038, 0.078), (0.021, x, -0.049, 0.045, 0.088),
            (0.041, x, -0.043, 0.044, 0.081), (0.073, x, -0.023, 0.042, 0.062),
            (0.118, x, 0.001, 0.042, 0.047), (0.164, x, 0.007, 0.043, 0.049),
        ]))

        # Narrow sleeves widen at the elbow and taper into embroidered cuffs.
        grip = side > 0
        front = -0.097 if grip else -0.025
        elbow = -0.008 if grip else 0.017
        sculpt.loft(curved_sections([
            (0.659, side * 0.240, front, 0.035, 0.038),
            (0.680, side * 0.240, front, 0.041, 0.043),
            (0.697, side * 0.239, front * 0.91, 0.040, 0.041),
            (0.760, side * 0.239, front * 0.65, 0.037, 0.037),
            (0.856, side * 0.229, elbow, 0.049, 0.052),
            (0.913, side * 0.212, 0.009, 0.051, 0.050),
            (0.982, side * 0.189, 0.007, 0.053, 0.056),
            (1.048, side * 0.164, 0.005, 0.057, 0.062),
            (1.079, side * 0.153, 0.005, 0.055, 0.053),
            (1.104, side * 0.144, 0.008, 0.046, 0.046),
        ]), sides=36)
        palm = 0.275 if grip else 0.242
        sculpt.loft([
            (0.603, side * palm, front - 0.006, 0.019, 0.020),
            (0.625, side * palm, front - 0.008, 0.026, 0.026),
            (0.655, side * (palm - 0.010), front - 0.005, 0.029, 0.031),
            (0.677, side * 0.246, front, 0.025, 0.028),
            (0.690, side * 0.239, front, 0.023, 0.027),
        ])
        sculpt.tube([
            ((side * 0.235, 0.663, front - 0.016), 0.012),
            ((side * (palm - 0.013), 0.653, front - 0.028), 0.014),
            ((side * (palm + 0.001), 0.636, front - 0.028), 0.011),
        ])


def face_relief(x, y, z, angle):
    front = max(0, -math.sin(angle)) ** 6
    eyes = 0.013 * soft_peak(abs(x), 0.040, 0.023) * soft_peak(y, 1.333, 0.012)
    brow = 0.012 * soft_peak(abs(x), 0.041, 0.032) * soft_peak(y, 1.349, 0.009)
    nose = 0.033 * soft_peak(x, 0, 0.018) * soft_peak(y, 1.309, 0.024)
    cheeks = 0.012 * soft_peak(abs(x), 0.056, 0.025) * soft_peak(y, 1.313, 0.028)
    lips = (0.011 * soft_peak(y, 1.278, 0.010) - 0.005 * soft_peak(y, 1.278, 0.003))
    return x, z + front * (eyes - brow - nose - cheeks - lips * soft_peak(x, 0, 0.033))


def build_head(sculpt):
    sculpt.loft(curved_sections([
        (1.096, 0, 0.005, 0.062, 0.062), (1.142, 0, 0.005, 0.049, 0.050),
        (1.200, 0, 0.001, 0.046, 0.047), (1.247, 0, -0.005, 0.049, 0.053),
    ]))

    sculpt.loft(curved_sections([
        (1.224, 0, -0.024, 0.023, 0.042), (1.241, 0, -0.024, 0.040, 0.057),
        (1.272, 0, -0.019, 0.064, 0.066), (1.307, 0, -0.006, 0.080, 0.073),
        (1.345, 0, -0.001, 0.080, 0.074), (1.376, 0, 0.002, 0.083, 0.077),
        (1.412, 0, 0.011, 0.080, 0.078), (1.450, 0, 0.016, 0.062, 0.061),
    ]), sides=64, relief=face_relief)

    # Sculpt the swept hair as continuous waves running to the upper back.
    sculpt.loft(curved_sections([
        (1.033, 0, 0.112, 0.030, 0.028), (1.070, 0.005, 0.108, 0.066, 0.040),
        (1.126, 0.011, 0.102, 0.089, 0.052), (1.219, 0.002, 0.074, 0.097, 0.059),
        (1.305, -0.010, 0.056, 0.103, 0.068), (1.384, 0, 0.024, 0.105, 0.092),
        (1.430, 0, 0.017, 0.095, 0.098), (1.466, 0, 0.013, 0.071, 0.075),
    ]), sides=56, relief=lambda x, y, z, a: (x, z + max(0, math.sin(a)) * 0.009 * math.cos(13 * a + 7 * y)))
    for side in (-1, 1):
        sculpt.tube([
            ((side * 0.028, 1.428, -0.060), 0.023),
            ((side * 0.060, 1.396, -0.057), 0.030),
            ((side * 0.083, 1.361, -0.032), 0.026),
            ((side * 0.091, 1.304, 0.006), 0.023),
            ((side * 0.077, 1.259, 0.009), 0.021),
            ((side * 0.090, 1.211, 0.020), 0.017),
            ((side * 0.086, 1.180, 0.020), 0.011),
        ])


def build_crown(sculpt):
    sculpt.loft(curved_sections([
        (1.407, 0, 0.013, 0.095, 0.086), (1.419, 0, 0.011, 0.122, 0.111),
        (1.437, 0, 0.012, 0.123, 0.115), (1.451, 0, 0.012, 0.119, 0.112),
        (1.470, 0, 0.012, 0.125, 0.119), (1.486, 0, 0.012, 0.130, 0.122),
        (1.499, 0, 0.012, 0.126, 0.119),
    ]), sides=72)

    for point in range(8):
        # Alternate tall and short fleur-de-lis points with rounded finials.
        angle = point * math.tau / 8 - math.pi / 2
        cos, sin = math.cos(angle), math.sin(angle)
        height = 0.187 if point % 2 == 0 else 0.139
        rings = []
        for fraction, radius, width, depth in [
            (0, 0.104, 0.047, 0.028), (0.39, 0.125, 0.041, 0.025),
            (0.55, 0.136, 0.031, 0.023), (0.69, 0.140, 0.038, 0.023),
            (0.79, 0.146, 0.017, 0.019), (0.94, 0.151, 0.018, 0.013),
            (1.0, 0.152, 0.011, 0.010),
        ]:
            ring = []
            for j in range(20):
                theta = math.tau * j / 20
                radial = radius + depth * math.cos(theta)
                tangent = width * math.sin(theta)
                ring.append((radial * cos - tangent * sin,
                             1.445 + height * fraction,
                             0.012 + radial * sin + tangent * cos))
            rings.append(ring)
        sculpt.rings(rings)
        center_x, center_z = 0.152 * cos, 0.012 + 0.152 * sin
        top = 1.445 + height
        pearl = [
            (-0.022, 0.011), (-0.014, 0.023), (0, 0.027), (0.014, 0.023), (0.025, 0.009),
        ]
        sculpt.loft([(top + offset, center_x, center_z, size, size) for offset, size in pearl], sides=18)

        gem_x, gem_z = 0.116 * cos, 0.012 + 0.111 * sin
        sculpt.loft([
            (1.442, gem_x, gem_z, 0.010, 0.011),
            (1.455, gem_x, gem_z, 0.021, 0.021),
            (1.470, gem_x, gem_z, 0.012, 0.012),
        ], sides=12)


def build_rapier(sculpt):
    x, z = 0.302, -0.122
    # The basket guard and flared pommel keep the slender blade readable on the board.
    sculpt.loft([
        (0.563, x, z, 0.010, 0.011), (0.579, x, z, 0.023, 0.022),
        (0.598, x, z, 0.022, 0.022), (0.617, x, z, 0.016, 0.017),
        (0.698, x + 0.004, z, 0.017, 0.017), (0.714, x + 0.004, z, 0.043, 0.032),
        (0.736, x + 0.007, z, 0.049, 0.035), (0.749, x + 0.008, z, 0.036, 0.027),
        (0.768, x + 0.009, z, 0.021, 0.017),
    ])
    sculpt.loft([
        (0.754, x + 0.009, z, 0.024, 0.017),
        (0.806, x + 0.012, z, 0.023, 0.015),
        (1.022, x + 0.030, z, 0.020, 0.014),
        (1.256, x + 0.047, z, 0.015, 0.012),
        (1.423, x + 0.063, z, 0.010, 0.009),
        (1.463, x + 0.067, z, 0.004, 0.004),
    ], sides=12)


def skin_weights(positions):
    x, y, z = positions.T
    weights = np.zeros((len(x), len(REST)), dtype=np.float32)
    blade = ease(0.257, 0.285, x) * (1 - ease(-0.088, -0.069, z))
    weights[:, 2] = blade
    weights[:, 1] = ease(1.090, 1.188, y) * (1 - blade)
    free = 1 - weights[:, 1] - blade

    inner_arm = np.interp(y, [0.62, 0.73, 0.86, 0.96, 1.06, 1.1],
                          [0.207, 0.188, 0.175, 0.158, 0.145, 0.148])
    arm = (free * ease(inner_arm, inner_arm + 0.057, abs(x)) *
           ease(0.565, 0.615, y) * (1 - ease(1.082, 1.124, y)))
    forearm = 1 - ease(0.810, 0.900, y)

    legs = (free - arm) * np.interp(y, [0.08, 0.13, 0.24, 0.40, 0.59, 0.67],
                                  [1, 0.91, 0.62, 0.30, 0.07, 0])
    shin = 1 - ease(0.220, 0.370, y)
    right_leg = ease(-0.10, 0.10, x)
    for shoulder, hip, arm_side, leg_side in [
        (3, 7, x < 0, 1 - right_leg), (5, 9, x >= 0, right_leg)
    ]:
        weights[:, shoulder] = arm * arm_side * (1 - forearm)
        weights[:, shoulder + 1] = arm * arm_side * forearm
        weights[:, hip] = legs * leg_side * (1 - shin)
        weights[:, hip + 1] = legs * leg_side * shin
    weights[:, 0] = np.maximum(0, free - arm - legs)
    ids = np.argsort(-weights, axis=1)[:, :4]
    weights = np.take_along_axis(weights, ids, axis=1)
    return ids, weights / weights.sum(axis=1, keepdims=True)


def build():
    sculpt = Sculpt()
    build_gown(sculpt)
    build_head(sculpt)
    build_crown(sculpt)
    build_rapier(sculpt)
    export_figure(sculpt, REST, skin_weights, OUTPUT, 'queen', 'Royal queen', 0.0048, 0.093, stl_output=STL_OUTPUT)


if __name__ == '__main__':
    build()

