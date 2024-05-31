use std::convert::Infallible;

use bevy::{prelude::*, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, math::IVec2, render::color::Color, window::{PresentMode, PrimaryWindow}};
use powderkeg::{cell::{Cell, Renderable, TickInput, TickSuccess}, chunk::{Chunk, ChunkBundle, ChunkCoords}, grid::Grid, simulation::PowderkegTickRate, stain::Stainable, viewer::DrawStained, PowderkegError, PowderkegPlugin, PowderkegSet};
use rand::{distributions::{Distribution, Uniform}, rngs::SmallRng, thread_rng, Rng, SeedableRng};

const CHUNK_SIZE: i32 = 64;

#[derive(Clone, Copy, Default)]
pub enum SimpleSand {
    Sand,
    Stone,
    #[default]
    Air,
}

pub struct SimpleSandDistribution {
    pub sand_weight: f32,
    pub stone_weight: f32,
    pub air_weight: f32,
}

impl Distribution<SimpleSand> for SimpleSandDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SimpleSand {
        let total_weights = self.sand_weight + self.stone_weight + self.air_weight;

        let mut t = Uniform::new(0.0, total_weights).sample(rng);

        if t <= self.sand_weight {
            return SimpleSand::Sand;
        }

        t -= self.sand_weight;

        if t <= self.stone_weight {
            return SimpleSand::Stone;
        }

        return SimpleSand::Air;
    }
}

#[derive(Deref, DerefMut)]
pub struct SimpleState(pub SmallRng);

impl Default for SimpleState {
    fn default() -> Self {
        Self(SmallRng::from_entropy())
    }
}

impl Cell for SimpleSand {
    type Error = Infallible;
    type State = SimpleState;

    fn tick<G: Stainable<Cell = Self>>(input: TickInput<'_, Self, G>) -> Result<TickSuccess, PowderkegError<Self>> {
        match input.this() {
            SimpleSand::Sand => {
                let mut rng = input.state().write_arc();

                if input.grid.map_cell(input.origin + IVec2::new(0, -1), |cell| matches!(cell, Self::Air))? {
                    input.grid.stain_around(input.origin, 3);
                    if rng.gen_bool(0.1) {
                        return Ok(TickSuccess::Unstable)
                    } else {
                        input.grid.swap(input.origin, input.origin + IVec2::new(0, -1))?;
                        return Ok(TickSuccess::Unstable);
                    }
                }

                let directions = if rng.gen_bool(0.5) {
                    &[
                        IVec2::new(-1, -1),
                        IVec2::new(1, -1),
                    ]
                } else {
                    &[
                        IVec2::new(1, -1),
                        IVec2::new(-1, -1),
                    ]
                };

                for offset in directions.iter() {
                    if input.grid.map_cell(input.origin + *offset, |cell| matches!(cell, Self::Air))? {
                        input.grid.swap(input.origin, input.origin + *offset)?;
                        input.grid.stain_around(input.origin, 1);
                        return Ok(TickSuccess::Unstable);
                    }
                }

                Ok(TickSuccess::Stable)
            },
            SimpleSand::Stone => Ok(TickSuccess::Stable),
            SimpleSand::Air => Ok(TickSuccess::Stable),
        }
    }

    fn range(&self) -> IRect {
        IRect::new(-1, -1, 1, 0)
    }
}

impl Renderable for SimpleSand {
    fn to_color(&self, _: IVec2) -> Color {
        match self {
            SimpleSand::Sand => Color::BEIGE,
            SimpleSand::Stone => Color::GRAY,
            SimpleSand::Air => Color::BLACK,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::Immediate,
                        title: String::from("Powderkeg Simple Example"),
                        ..default()
                    }),
                    ..default()
                })
        )
        .add_plugins(PowderkegPlugin::<SimpleSand, CHUNK_SIZE>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_title)
        .add_systems(Update, paint_sand.before(PowderkegSet::Tick))
        .run();
}

fn setup(
    mut commands: Commands,
) {
    let mut rng = thread_rng();

    let distribution = SimpleSandDistribution {
        sand_weight: 1.0,
        stone_weight: 0.0,
        air_weight: 1.0,
    };

    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(PowderkegTickRate(64.0));

    commands
        .spawn(SpatialBundle {
            transform: Transform::default().with_scale(Vec3::splat(2.0)),
            ..Default::default()
        })
        .with_children(|children| {
            for cx in -3..=3 {
                for cy in -3..=3 {
                    let state = SimpleState(SmallRng::from_rng(&mut rng).unwrap());
                    let chunk = Chunk::full_random(&mut rng, &distribution, state);

                    let chunk_coords = IVec2::new(cx, cy);
        
                    children.spawn((
                        ChunkBundle::<SimpleSand, CHUNK_SIZE> {
                            chunk,
                            coords: ChunkCoords(chunk_coords),
                            transform: TransformBundle::from_transform(Transform::from_translation(chunk_coords.as_vec2().extend(0.0) * CHUNK_SIZE as f32)),
                            ..default()
                        },
                        DrawStained,
                    ));
                }
            }
        });
}

fn update_title(
    diagnostics: Res<DiagnosticsStore>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps) = fps.smoothed() {
                window.title = format!("Powderkeg Simple Example ({:.0} fps)", fps);
            }
        }
    }
}

fn paint_sand(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut chunks: Query<(&mut Chunk<SimpleSand, CHUNK_SIZE>, &GlobalTransform)>,
) {
    let (camera, camera_transform) = cameras.single();

    let Some(position) = windows.single().cursor_position().and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor)) else {
        return;
    };

    let cell = if buttons.pressed(MouseButton::Left) {
        SimpleSand::Sand
    } else if buttons.pressed(MouseButton::Right) {
        SimpleSand::Air
    } else if buttons.pressed(MouseButton::Middle) {
        SimpleSand::Stone
    } else {
        return;
    };

    for (mut chunk, transform) in chunks.iter_mut() {
        let local = transform.affine().inverse().transform_point3(position.extend(0.0)).truncate().as_ivec2() + IVec2::splat(CHUNK_SIZE / 2);

        let local_rect = IRect::from_corners(local - 3, local + 3);

        if !Chunk::<SimpleSand, CHUNK_SIZE>::area().intersect(local_rect).is_empty() {
            for x in (local.x - 3)..=(local.x + 3) {
                for y in (local.y - 3)..=(local.y + 3) {
                    chunk.map_cell_mut(IVec2::new(x, y), |old| *old = cell).ok();
                }
            }

            chunk.stain_around(local, 5);
        }
    }
}
