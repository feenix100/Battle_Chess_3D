//! Short corner fountains, driven by the same clock as the surrender poses.

use bevy::{light::NotShadowCaster, prelude::*};

use crate::characters::CharacterScene;

const SPRAY_SECONDS: f32 = 3.0;
const PULSE_SECONDS: f32 = 0.06;
const GRAVITY: f32 = 7.6;
const CORNERS: [(f32, f32); 4] = [(-4.05, -4.05), (4.05, -4.05), (-4.05, 4.05), (4.05, 4.05)];

#[derive(Resource, Default)]
pub(crate) struct FireworkAssets {
    streak: Handle<Mesh>,
    colors: Vec<Handle<StandardMaterial>>,
}

impl FireworkAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let colors = [
            Color::srgb(1.0, 0.73, 0.13),
            Color::srgb(1.0, 0.95, 0.65),
            Color::srgb(0.16, 0.90, 1.0),
            Color::srgb(1.0, 0.24, 0.56),
        ]
        .into_iter()
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 4.0,
                unlit: true,
                ..default()
            })
        })
        .collect();
        Self {
            streak: meshes.add(Cuboid::default()),
            colors,
        }
    }
}

#[derive(Component)]
pub(crate) struct FireworkSpark {
    origin: Vec3,
    velocity: Vec3,
    born: f32,
    lifetime: f32,
    size: f32,
}

impl FireworkSpark {
    fn sample(&self, now: f32) -> Option<Transform> {
        let age = now - self.born;
        if age < 0.0 || age >= self.lifetime {
            return None;
        }
        let velocity = self.velocity - Vec3::Y * (GRAVITY * age);
        let fade = ((self.lifetime - age) / 0.28).clamp(0.0, 1.0).sqrt();
        Some(
            Transform::from_translation(
                self.origin + self.velocity * age - Vec3::Y * (0.5 * GRAVITY * age * age),
            )
            .with_rotation(Quat::from_rotation_arc(
                Vec3::Y,
                velocity.normalize_or_zero(),
            ))
            .with_scale(Vec3::new(
                self.size * fade,
                (0.055 + velocity.length() * 0.025) * fade,
                self.size * fade,
            )),
        )
    }
}

#[derive(Default)]
pub(crate) struct Fountain {
    next_pulse: u32,
    last_elapsed: Option<f32>,
}

pub(crate) fn animate_fireworks(
    mut commands: Commands,
    scene: Res<CharacterScene>,
    assets: Option<Res<FireworkAssets>>,
    mut fountain: Local<Fountain>,
    mut sparks: Query<(Entity, &FireworkSpark, &mut Transform)>,
) {
    let elapsed = scene.checkmate.as_ref().map(|state| state.elapsed);
    let reset =
        elapsed.is_none_or(|now| now < 0.0 || fountain.last_elapsed.is_some_and(|last| now < last));
    if reset {
        for (entity, _, _) in &sparks {
            commands.entity(entity).despawn();
        }
        *fountain = Fountain::default();
        if elapsed.is_none_or(|now| now < 0.0) {
            return;
        }
    }
    let now = elapsed.unwrap();
    fountain.last_elapsed = Some(now);
    for (entity, spark, mut transform) in &mut sparks {
        if let Some(pose) = spark.sample(now) {
            *transform = pose;
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some(assets) = assets.filter(|assets| !assets.colors.is_empty()) else {
        return;
    };
    // Fixed timestamps make the spray consistent at different frame rates.
    while (fountain.next_pulse as f32 * PULSE_SECONDS) < SPRAY_SECONDS {
        let pulse = fountain.next_pulse;
        let born = pulse as f32 * PULSE_SECONDS;
        if born > now {
            break;
        }
        fountain.next_pulse += 1;
        for (corner, (x, z)) in CORNERS.into_iter().enumerate() {
            let outward = Vec3::new(x, 0.0, z).normalize();
            let sideways = Vec3::new(-outward.z, 0.0, outward.x);
            for jet in 0..4 {
                let seed = pulse * 31 + corner as u32 * 17 + jet * 47;
                let phase = (seed as f32 * 2.399_963).sin();
                let spark = FireworkSpark {
                    origin: Vec3::new(x, 0.14, z),
                    velocity: Vec3::Y * (4.0 + (seed % 9) as f32 * 0.17)
                        + outward * (0.35 + (jet as f32 * 0.41))
                        + sideways * (phase * 1.25),
                    born,
                    lifetime: 0.70 + (seed % 7) as f32 * 0.045,
                    size: 0.023 + (seed % 3) as f32 * 0.005,
                };
                if let Some(transform) = spark.sample(now) {
                    commands.spawn((
                        Mesh3d(assets.streak.clone()),
                        MeshMaterial3d(
                            assets.colors[(seed as usize / 3) % assets.colors.len()].clone(),
                        ),
                        transform,
                        NotShadowCaster,
                        spark,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::characters::CheckmateCelebration;
    use shakmaty::Color as ChessColor;

    #[test]
    fn four_corner_fountains_wait_spray_once_expire_and_reset() {
        let mut app = App::new();
        let mut meshes = Assets::default();
        let mut materials = Assets::default();
        app.insert_resource(FireworkAssets::new(&mut meshes, &mut materials))
            .init_resource::<CharacterScene>()
            .add_systems(Update, animate_fireworks);
        let set_time = |app: &mut App, elapsed| {
            app.world_mut().resource_mut::<CharacterScene>().checkmate =
                Some(CheckmateCelebration {
                    loser: ChessColor::Black,
                    elapsed,
                });
            app.update();
        };
        set_time(&mut app, -0.2);
        assert_eq!(
            app.world_mut()
                .query::<&FireworkSpark>()
                .iter(app.world())
                .count(),
            0
        );
        set_time(&mut app, 0.0);
        let world = app.world_mut();
        let mut count = [0; 4];
        for spark in world.query::<&FireworkSpark>().iter(world) {
            let corner = CORNERS
                .iter()
                .position(|&(x, z)| spark.origin == Vec3::new(x, 0.14, z))
                .unwrap();
            count[corner] += 1;
        }
        assert_eq!(count, [4; 4]);
        // Repeated updates at the same time never emit twice.
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&FireworkSpark>()
                .iter(app.world())
                .count(),
            16
        );
        for tick in 1..=85 {
            set_time(&mut app, tick as f32 * 0.05);
        }
        assert_eq!(
            app.world_mut()
                .query::<&FireworkSpark>()
                .iter(app.world())
                .count(),
            0
        );
        set_time(&mut app, 5.0);
        assert_eq!(
            app.world_mut()
                .query::<&FireworkSpark>()
                .iter(app.world())
                .count(),
            0
        );
        // New Game clears active particles and allows the next game's celebration.
        app.world_mut().resource_mut::<CharacterScene>().checkmate = None;
        app.update();
        set_time(&mut app, 0.12);
        assert!(
            app.world_mut()
                .query::<&FireworkSpark>()
                .iter(app.world())
                .count()
                > 16
        );
        app.world_mut().resource_mut::<CharacterScene>().checkmate = None;
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&FireworkSpark>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
