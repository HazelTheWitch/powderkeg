use std::convert::Infallible;

use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::*, window::{PresentMode, PrimaryWindow}};
use powderkeg::{cell::{Action, Cell, Renderable}, chunk::{Chunk, ChunkBundle, ChunkCoords}, grid::Grid, simulation::PowderkegTickRate, stain::Stainable, viewer::DrawStained, PowderkegPlugin, PowderkegSet};
use rand::{rngs::ThreadRng, thread_rng, Rng};

const CHUNK_SIZE: i32 = 64;

#[derive(Clone, Copy, Default)]
pub enum SimpleSand {
    Sand,
    Stone,
    #[default]
    Air,
}

impl Cell for SimpleSand {
    type Error = Infallible;

    fn tick(&self, origin: IVec2, grid: &impl Grid<Cell = Self>) -> Result<Option<Self::Action>, Self::Error> {
        match self {
            SimpleSand::Sand => {
                let mut rng = thread_rng();

                if matches!(grid.map(origin + IVec2::new(0, -1), |cell| matches!(cell, Self::Air)), Some(true)) {
                    if rng.gen_bool(0.1) {
                        return Ok(Some(SimpleAction::Stable));
                    } else {
                        return Ok(Some(SimpleAction::Fall(Direction::Down)));
                    }
                }

                let directions = if rng.gen_bool(0.5) {
                    &[
                        (Direction::Left, IVec2::new(-1, -1)),
                        (Direction::Right, IVec2::new(1, -1)),
                    ]
                } else {
                    &[
                        (Direction::Right, IVec2::new(1, -1)),
                        (Direction::Left, IVec2::new(-1, -1)),
                    ]
                };

                for (direction, offset) in directions.iter() {
                    if matches!(grid.map(origin + *offset, |cell| matches!(cell, Self::Air)), Some(true)) {
                        return Ok(Some(SimpleAction::Fall(*direction)));
                    }
                }

                Ok(None)
            },
            SimpleSand::Stone => Ok(None),
            SimpleSand::Air => Ok(None),
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

#[derive(Clone, Copy)]
pub enum Direction {
    Down,
    Left,
    Right,
}

pub enum SimpleAction {
    Stable,
    Fall(Direction),
}

impl Action for SimpleAction {
    type Cell = SimpleSand;
    type State = ();

    fn act(&self, origin: IVec2, grid: &mut impl Stainable<Cell = Self::Cell>) -> Option<()> {
        match self {
            SimpleAction::Fall(direction) => {
                grid.stain_around(origin, 2);
                match direction {
                    Direction::Down => grid.swap(origin, origin + IVec2::new(0, -1))?,
                    Direction::Left => grid.swap(origin, origin + IVec2::new(-1, -1))?,
                    Direction::Right => grid.swap(origin, origin + IVec2::new(1, -1))?,
                }
            },
            SimpleAction::Stable => {
                grid.stain_around(origin, 1);
            },
        }

        Some(())
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
        .add_systems(Update, paint_sand.before(PowderkegSet::Sync))
        .run();
}

fn setup(
    mut commands: Commands,
) {
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
                    let mut center_chunk = Chunk::default();

                    let mut rng = thread_rng();
        
                    for x in 0..CHUNK_SIZE {
                        for y in 0..CHUNK_SIZE {
                            if rng.gen_bool(0.5) {
                                center_chunk.replace(IVec2::new(x, y), SimpleSand::Sand);
                            }
                        }
                    }

                    let chunk_coords = IVec2::new(cx, cy);
        
                    children.spawn((
                        ChunkBundle::<SimpleSand, CHUNK_SIZE> {
                            chunk: center_chunk,
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
                    chunk.map_mut(IVec2::new(x, y), |old| *old = cell);
                }
            }

            chunk.stain_around(local, 5);
        }
    }
}
