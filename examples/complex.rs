use std::convert::Infallible;

use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::*, window::{PresentMode, PrimaryWindow}};
use powderkeg::{cell::{Action, Cell, Renderable}, chunk::{Chunk, ChunkBundle, ChunkCoords}, simulation::PowderkegTickRate, viewer::DrawStained, PowderkegPlugin, PowderkegSet, stain::Stainable, grid::Grid};
use rand::prelude::*;

const CHUNK_SIZE: i32 = 64;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum DynamicCell {
    #[default]
    Air,
    Gunpowder,
    Fire,
    Smoke,
    Stone,
}

impl DynamicCell {
    pub fn solid(&self) -> bool {
        match self {
            DynamicCell::Stone | DynamicCell::Gunpowder => true,
            _ => false,
        }
    }

    pub fn flammable(&self) -> bool {
        match self {
            DynamicCell::Fire => true,
            _ => false
        }
    }
}

pub enum DynamicAction {
    Fall(IVec2),
    SetTo(DynamicCell),
    Explode(i32),
}

impl Cell for DynamicCell {
    type Action = DynamicAction;
    type Error = Infallible;

    fn tick(&self, origin: IVec2, grid: &impl Grid<Cell = Self>) -> Result<Option<Self::Action>, Self::Error> {
        let mut rng = thread_rng();

        let dynamic_action = match self {
            DynamicCell::Air => Ok(None),
            DynamicCell::Gunpowder => {
                for y in (origin.y - 1)..=(origin.y + 1) {
                    for x in (origin.x - 1)..=(origin.x + 1) {
                        if matches!(grid.map_cell(IVec2::new(x, y), Self::flammable), Some(true)) && rng.gen_bool(0.4) {
                            return Ok(Some(DynamicAction::Explode(2)));
                        }
                    }
                }

                let offsets = [IVec2::new(0, -1), IVec2::new(-1, -1), IVec2::new(1, -1)];

                for offset in offsets {
                    if !matches!(grid.map_cell(origin + offset, Self::solid), Some(true)) {
                        return Ok(Some(DynamicAction::Fall(offset)));
                    }
                }

                Ok(None)
            },
            DynamicCell::Fire => {
                if rng.gen_bool(0.01) {
                    return Ok(Some(DynamicAction::SetTo(Self::Smoke)));
                }

                let offsets = &mut [IVec2::new(0, 1), IVec2::new(-1, 1), IVec2::new(1, 1), IVec2::new(0, -1)];

                offsets.shuffle(&mut rng);

                for offset in offsets {
                    if !matches!(grid.map_cell(origin + *offset, Self::solid), Some(true)) {
                        return Ok(Some(DynamicAction::Fall(*offset)));
                    }
                }

                Ok(None)
            },
            DynamicCell::Smoke => {
                if rng.gen_bool(0.01) {
                    return Ok(Some(DynamicAction::SetTo(Self::Air)));
                }

                let offsets = &mut [IVec2::new(0, 1), IVec2::new(-1, 1), IVec2::new(1, 1), IVec2::new(0, -1)];

                offsets.shuffle(&mut rng);

                for offset in offsets {
                    if !matches!(grid.map_cell(origin + *offset, Self::solid), Some(true)) {
                        return Ok(Some(DynamicAction::Fall(*offset)));
                    }
                }

                Ok(None)
            },
            DynamicCell::Stone => Ok(None),
        };
        dynamic_action
    }

    fn range(&self) -> IRect {
        IRect::from_center_half_size(IVec2::ZERO, IVec2::splat(4))
    }
}

impl Renderable for DynamicCell {
    fn to_color(&self, _: IVec2) -> Color {
        match self {
            DynamicCell::Air => Color::BLACK,
            DynamicCell::Gunpowder => Color::GRAY,
            DynamicCell::Fire => Color::RED,
            DynamicCell::Smoke => Color::DARK_GRAY,
            DynamicCell::Stone => Color::BEIGE,
        }
    }
}

impl Action for DynamicAction {
    type Cell = DynamicCell;
    type State = ();

    fn act(&self, origin: IVec2, grid: &mut impl Stainable<Cell = Self::Cell, State = Self::State>) -> Option<()> {
        let mut rng = thread_rng();

        match self {
            DynamicAction::Fall(offset) => {
                grid.stain_around(origin, 2);
                grid.stain_around(origin + *offset, 1);

                grid.swap(origin, origin + *offset)
            },
            DynamicAction::SetTo(cell) => {
                grid.stain_around(origin, 1);
                grid.replace(origin, *cell)?;

                Some(())
            },
            DynamicAction::Explode(radius) => {
                for y in (origin.y - radius)..=(origin.y + radius) {
                    for x in (origin.x - radius)..=(origin.x + radius) {
                        let point = IVec2::new(x, y);

                        if let Some(DynamicCell::Air) = grid.get(point) {
                            if rng.gen_bool(0.5) {
                                grid.replace(point, DynamicCell::Fire);
                                grid.stain_around(point, 1);
                            }
                        }
                    }
                }

                grid.stain_around(origin, 1);
                grid.replace(origin, DynamicCell::Fire)?;

                Some(())
            },
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
        .add_plugins(PowderkegPlugin::<DynamicCell, CHUNK_SIZE>::default())
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
                    let chunk_coords = IVec2::new(cx, cy);
        
                    children.spawn((
                        ChunkBundle::<DynamicCell, CHUNK_SIZE> {
                            chunk: Chunk::default(),
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
    mut chunks: Query<(&mut Chunk<DynamicCell, CHUNK_SIZE>, &GlobalTransform)>,
) {
    let (camera, camera_transform) = cameras.single();

    let Some(position) = windows.single().cursor_position().and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor)) else {
        return;
    };

    let cell = if buttons.pressed(MouseButton::Left) {
        DynamicCell::Gunpowder
    } else if buttons.pressed(MouseButton::Right) {
        DynamicCell::Fire
    } else if buttons.pressed(MouseButton::Middle) {
        DynamicCell::Stone
    } else {
        return;
    };

    for (mut chunk, transform) in chunks.iter_mut() {
        let local = transform.affine().inverse().transform_point3(position.extend(0.0)).truncate().as_ivec2() + IVec2::splat(CHUNK_SIZE / 2);

        let local_rect = IRect::from_corners(local - 3, local + 3);

        if !Chunk::<DynamicCell, CHUNK_SIZE>::area().intersect(local_rect).is_empty() {
            for x in (local.x - 3)..=(local.x + 3) {
                for y in (local.y - 3)..=(local.y + 3) {
                    chunk.map_cell_mut(IVec2::new(x, y), |old| *old = cell);
                }
            }

            chunk.stain_around(local, 5);
        }
    }
}
