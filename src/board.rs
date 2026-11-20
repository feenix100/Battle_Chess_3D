//! Board finishes use reusable, tileable PBR maps. Color remains an independent
//! tint so appearance controls work with every surface and never rebuild images.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{Appearance, BoardPreset, SceneHandles, hex_rgb, rgb};

const RESOLUTION: usize = 256;
const PRESETS: [BoardPreset; 6] = [
    BoardPreset::Walnut,
    BoardPreset::Marble,
    BoardPreset::Tournament,
    BoardPreset::Slate,
    BoardPreset::Obsidian,
    BoardPreset::Neon,
];

struct Surface {
    color: Handle<Image>,
    normal: Handle<Image>,
    roughness: Handle<Image>,
}

#[derive(Resource)]
pub(crate) struct BoardMaterials {
    surfaces: Vec<Surface>,
    trim: Handle<StandardMaterial>,
    plinth: Handle<StandardMaterial>,
}

impl BoardMaterials {
    pub fn new(images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            surfaces: PRESETS
                .into_iter()
                .map(|preset| surface(images, preset))
                .collect(),
            trim: materials.add(trim_material(BoardPreset::Walnut)),
            plinth: materials.add(StandardMaterial {
                base_color: rgb(hex_rgb(0x171512)),
                perceptual_roughness: 0.40,
                metallic: 0.18,
                ..default()
            }),
        }
    }

    fn maps(&self, preset: BoardPreset) -> &Surface {
        &self.surfaces[PRESETS.iter().position(|&item| item == preset).unwrap()]
    }

    pub fn square_material(&self, appearance: &Appearance, dark: bool) -> StandardMaterial {
        let maps = self.maps(appearance.board_preset);
        let (coat, coat_roughness) = match appearance.board_preset {
            BoardPreset::Walnut => (0.34, 0.28),
            BoardPreset::Marble => (0.42, 0.22),
            BoardPreset::Tournament => (0.0, 0.60),
            BoardPreset::Slate => (0.04, 0.65),
            BoardPreset::Obsidian => (0.60, 0.18),
            BoardPreset::Neon => (0.32, 0.26),
        };
        StandardMaterial {
            base_color: rgb(if dark {
                appearance.dark_square
            } else {
                appearance.light_square
            }),
            base_color_texture: Some(maps.color.clone()),
            normal_map_texture: Some(maps.normal.clone()),
            metallic_roughness_texture: Some(maps.roughness.clone()),
            perceptual_roughness: appearance.board_roughness,
            metallic: appearance.board_metallic,
            reflectance: 0.38,
            clearcoat: coat,
            clearcoat_perceptual_roughness: coat_roughness,
            ..default()
        }
    }

    pub fn frame_material(&self, preset: BoardPreset) -> StandardMaterial {
        let maps = self.maps(preset);
        let color = match preset {
            BoardPreset::Walnut => 0x39251b,
            BoardPreset::Marble => 0x303740,
            BoardPreset::Tournament => 0x25372d,
            BoardPreset::Slate => 0x252d34,
            BoardPreset::Obsidian => 0x20232b,
            BoardPreset::Neon => 0x1a2232,
        };
        StandardMaterial {
            base_color: rgb(hex_rgb(color)),
            base_color_texture: Some(maps.color.clone()),
            normal_map_texture: Some(maps.normal.clone()),
            metallic_roughness_texture: Some(maps.roughness.clone()),
            perceptual_roughness: if preset == BoardPreset::Slate {
                0.67
            } else {
                0.34
            },
            metallic: 0.05,
            clearcoat: 0.40,
            clearcoat_perceptual_roughness: 0.26,
            ..default()
        }
    }

    pub fn update(
        &self,
        appearance: &Appearance,
        handles: &SceneHandles,
        materials: &mut Assets<StandardMaterial>,
    ) {
        for (handle, dark) in [(&handles.light_square, false), (&handles.dark_square, true)] {
            if let Some(mut material) = materials.get_mut(handle) {
                *material = self.square_material(appearance, dark);
            }
        }
        if let Some(mut material) = materials.get_mut(&handles.board_base) {
            *material = self.frame_material(appearance.board_preset);
        }
        if let Some(mut material) = materials.get_mut(&self.trim) {
            *material = trim_material(appearance.board_preset);
        }
    }
}

fn trim_material(preset: BoardPreset) -> StandardMaterial {
    let color = match preset {
        BoardPreset::Walnut => 0xb49a63,
        BoardPreset::Marble | BoardPreset::Slate => 0xa3adb8,
        BoardPreset::Tournament => 0x8c9a81,
        BoardPreset::Obsidian => 0xb69a60,
        BoardPreset::Neon => 0x60c9d0,
    };
    StandardMaterial {
        base_color: rgb(hex_rgb(color)),
        metallic: 0.72,
        perceptual_roughness: 0.33,
        ..default()
    }
}

pub(crate) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    handles: &SceneHandles,
    materials: &BoardMaterials,
) {
    // Keep the playing surface at y=.06 and its coordinates exactly as before.
    let base = meshes.add(beveled_box(
        Vec3::new(9.12, 0.25, 9.12),
        0.065,
        Vec2::ZERO,
        false,
    ));
    commands.spawn((
        Mesh3d(base),
        MeshMaterial3d(materials.plinth.clone()),
        Transform::from_xyz(0.0, -0.18, 0.0),
    ));
    let long_rail = meshes.add(beveled_box(
        Vec3::new(9.04, 0.13, 0.51),
        0.018,
        Vec2::ZERO,
        true,
    ));
    let short_rail = meshes.add(beveled_box(
        Vec3::new(8.02, 0.13, 0.51),
        0.018,
        Vec2::new(0.0, 0.43),
        true,
    ));
    for side in [-1.0, 1.0] {
        commands.spawn((
            Mesh3d(long_rail.clone()),
            MeshMaterial3d(handles.board_base.clone()),
            Transform::from_xyz(0.0, -0.025, side * 4.265),
        ));
        commands.spawn((
            Mesh3d(short_rail.clone()),
            MeshMaterial3d(handles.board_base.clone()),
            Transform::from_xyz(side * 4.265, -0.025, 0.0)
                .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        ));
    }
    for rank in 0..8 {
        for file in 0..8 {
            let dark = (file + rank) % 2 == 0;
            let offset = Vec2::new(hash(file, rank) * 0.78, hash(rank + 11, file + 31) * 0.82);
            let tile = meshes.add(beveled_box(
                Vec3::new(0.994, 0.12, 0.994),
                0.006,
                offset,
                dark,
            ));
            commands.spawn((
                Mesh3d(tile),
                MeshMaterial3d(if dark {
                    handles.dark_square.clone()
                } else {
                    handles.light_square.clone()
                }),
                Transform::from_xyz(file as f32 - 3.5, 0.0, 3.5 - rank as f32),
            ));
        }
    }
    // A narrow metal inlay separates the veneer from the frame. The second
    // hairline catches light on the outside edge without brightening the board.
    for (extent, width, y) in [(4.035, 0.014, 0.045), (4.475, 0.010, 0.042)] {
        for side in [-1.0, 1.0] {
            for turn in [false, true] {
                commands.spawn((
                    Mesh3d(handles.cube.clone()),
                    MeshMaterial3d(materials.trim.clone()),
                    Transform::from_xyz(
                        if turn { side * extent } else { 0.0 },
                        y,
                        if turn { 0.0 } else { side * extent },
                    )
                    .with_scale(if turn {
                        Vec3::new(width, 0.008, extent * 2.0)
                    } else {
                        Vec3::new(extent * 2.0, 0.008, width)
                    }),
                ));
            }
        }
    }
}

fn hash(x: u32, y: u32) -> f32 {
    let mut value = x.wrapping_mul(0x9e37_79b9) ^ y.wrapping_mul(0x85eb_ca6b) ^ 0xa3c5_9ac3;
    value = (value ^ (value >> 16)).wrapping_mul(0x7feb_352d);
    value = (value ^ (value >> 15)).wrapping_mul(0x846c_a68b);
    ((value ^ (value >> 16)) & 0x00ff_ffff) as f32 / 16_777_215.0
}

fn noise(u: f32, v: f32, fx: u32, fy: u32) -> f32 {
    let x = u * fx as f32;
    let y = v * fy as f32;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let a = x - x.floor();
    let b = y - y.floor();
    let a = a * a * (3.0 - 2.0 * a);
    let b = b * b * (3.0 - 2.0 * b);
    let sample = |dx: i32, dy: i32| {
        hash(
            (ix + dx).rem_euclid(fx as i32) as u32,
            (iy + dy).rem_euclid(fy as i32) as u32,
        )
    };
    sample(0, 0)
        .lerp(sample(1, 0), a)
        .lerp(sample(0, 1).lerp(sample(1, 1), a), b)
}

fn cloudy(u: f32, v: f32) -> f32 {
    noise(u, v, 3, 3) * 0.57 + noise(u, v, 7, 7) * 0.28 + noise(u, v, 17, 17) * 0.15
}

/// Returns neutral albedo, micro-height and relative roughness.
fn finish(preset: BoardPreset, u: f32, v: f32) -> (Vec3, f32, f32) {
    let cloud = cloudy(u, v);
    let fine = noise(u, v, 127, 127);
    let (value, height, roughness) = match preset {
        BoardPreset::Walnut => {
            let warp = noise(u, v, 3, 2) * 2.4 + noise(u, v, 7, 3) * 0.48;
            let ring = (0.5 + 0.5 * ((u * 27.0 + warp) * TAU).sin()).powi(12);
            let fibers = noise(u, v, 103, 7);
            (
                0.86 + 0.10 * cloud + 0.035 * fibers - 0.18 * ring,
                fibers * 0.016 - ring * 0.012,
                0.79 + fibers * 0.20,
            )
        }
        BoardPreset::Marble => {
            let flow = (u * 2.0 + v * 3.0 + cloud * 1.8 + noise(u, v, 13, 13) * 0.12) * TAU;
            let vein = (-flow.sin().abs() * 32.0).exp();
            (
                0.98 - 0.23 * vein - 0.06 * cloud,
                cloud * 0.008 - vein * 0.003,
                0.80 + cloud * 0.14,
            )
        }
        BoardPreset::Tournament => {
            let weave = (u * 128.0 * TAU).cos() * (v * 128.0 * TAU).cos();
            (
                0.96 + fine * 0.035,
                fine * 0.006 + weave * 0.001,
                0.93 + fine * 0.06,
            )
        }
        BoardPreset::Slate => (
            0.83 + cloud * 0.13 + fine * 0.04,
            cloud * 0.05 + fine * 0.027,
            0.87 + fine * 0.12,
        ),
        BoardPreset::Obsidian => {
            let flow = (u * 3.0 - v * 2.0 + cloud) * TAU;
            let seam = (-flow.sin().abs() * 48.0).exp();
            (
                0.94 - seam * 0.10 + cloud * 0.04,
                cloud * 0.004,
                0.80 + cloud * 0.18,
            )
        }
        BoardPreset::Neon => {
            let line = |coordinate: f32| (((coordinate * 8.0).fract() - 0.5).abs() * 2.0).powi(36);
            let grid = line(u).max(line(v));
            (
                0.96 - grid * 0.055 + fine * 0.015,
                grid * 0.009,
                0.82 + grid * 0.15,
            )
        }
    };
    let tint = if preset == BoardPreset::Walnut {
        Vec3::new(1.0, 0.992, 0.975)
    } else {
        Vec3::ONE
    };
    (Vec3::splat(value) * tint, height, roughness)
}

fn surface(images: &mut Assets<Image>, preset: BoardPreset) -> Surface {
    let mut colors = Vec::with_capacity(RESOLUTION * RESOLUTION * 4);
    let mut heights = Vec::with_capacity(RESOLUTION * RESOLUTION);
    let mut roughness = Vec::with_capacity(RESOLUTION * RESOLUTION * 4);
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    for y in 0..RESOLUTION {
        for x in 0..RESOLUTION {
            let (color, height, rough) = finish(
                preset,
                x as f32 / RESOLUTION as f32,
                y as f32 / RESOLUTION as f32,
            );
            colors.extend([byte(color.x), byte(color.y), byte(color.z), 255]);
            heights.push(height);
            // glTF convention: green roughness; blue metallic multiplier.
            roughness.extend([255, byte(rough), 255, 255]);
        }
    }
    let mut normals = Vec::with_capacity(colors.len());
    for y in 0..RESOLUTION {
        for x in 0..RESOLUTION {
            let left = heights[y * RESOLUTION + (x + RESOLUTION - 1) % RESOLUTION];
            let right = heights[y * RESOLUTION + (x + 1) % RESOLUTION];
            let up = heights[((y + RESOLUTION - 1) % RESOLUTION) * RESOLUTION + x];
            let down = heights[((y + 1) % RESOLUTION) * RESOLUTION + x];
            let normal = Vec3::new((left - right) * 3.5, (up - down) * 3.5, 1.0).normalize();
            normals.extend([
                byte(normal.x * 0.5 + 0.5),
                byte(normal.y * 0.5 + 0.5),
                byte(normal.z * 0.5 + 0.5),
                255,
            ]);
        }
    }
    Surface {
        color: images.add(texture(colors, true)),
        normal: images.add(texture(normals, false)),
        roughness: images.add(texture(roughness, false)),
    }
}

fn texture(data: Vec<u8>, srgb: bool) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: RESOLUTION as u32,
            height: RESOLUTION as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data.clone(),
        if srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::default(),
    );
    // Mipmaps and anisotropic filtering keep fine grain stable while orbiting.
    let mut mip = data;
    let mut size = RESOLUTION;
    let mut chain = mip.clone();
    while size > 1 {
        let smaller = size / 2;
        let mut next = Vec::with_capacity(smaller * smaller * 4);
        for y in 0..smaller {
            for x in 0..smaller {
                for channel in 0..4 {
                    let mut sum = 0_u32;
                    for dy in 0..2 {
                        for dx in 0..2 {
                            sum += mip[((y * 2 + dy) * size + x * 2 + dx) * 4 + channel] as u32;
                        }
                    }
                    next.push((sum / 4) as u8);
                }
            }
        }
        chain.extend_from_slice(&next);
        mip = next;
        size = smaller;
        image.texture_descriptor.mip_level_count += 1;
    }
    image.data = Some(chain);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        anisotropy_clamp: 8,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn beveled_box(size: Vec3, bevel: f32, offset: Vec2, turn: bool) -> Mesh {
    let half = size * 0.5;
    let outline = |inset: f32, corner: f32, y: f32| {
        let x = half.x - inset;
        let z = half.z - inset;
        [
            Vec3::new(-x + corner, y, -z),
            Vec3::new(-x, y, -z + corner),
            Vec3::new(-x, y, z - corner),
            Vec3::new(-x + corner, y, z),
            Vec3::new(x - corner, y, z),
            Vec3::new(x, y, z - corner),
            Vec3::new(x, y, -z + corner),
            Vec3::new(x - corner, y, -z),
        ]
    };
    let layers = [
        outline(bevel, bevel * 0.4, half.y),
        outline(0.0, bevel, half.y - bevel),
        outline(0.0, bevel, -half.y + bevel),
        outline(bevel, bevel * 0.4, -half.y),
    ];
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut triangle = |a: Vec3, b: Vec3, c: Vec3| {
        let normal = (b - a).cross(c - a).normalize();
        for p in [a, b, c] {
            let base = if normal.y.abs() > 0.25 {
                Vec2::new(p.x, p.z)
            } else if normal.x.abs() > normal.z.abs() {
                Vec2::new(p.z, p.y)
            } else {
                Vec2::new(p.x, p.y)
            };
            let uv = if turn {
                Vec2::new(-base.y, base.x)
            } else {
                base
            };
            indices.push(positions.len() as u32);
            positions.push(p.to_array());
            normals.push(normal.to_array());
            uvs.push((uv + offset).to_array());
        }
    };
    for i in 0..8 {
        let next = (i + 1) % 8;
        triangle(Vec3::new(0.0, half.y, 0.0), layers[0][i], layers[0][next]);
        triangle(Vec3::new(0.0, -half.y, 0.0), layers[3][next], layers[3][i]);
        for pair in layers.windows(2) {
            triangle(pair[0][i], pair[1][i], pair[0][next]);
            triangle(pair[0][next], pair[1][i], pair[1][next]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices));
    mesh.generate_tangents()
        .expect("Board surface needs tangent-space UVs");
    mesh
}
