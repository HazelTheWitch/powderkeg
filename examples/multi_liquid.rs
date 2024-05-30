use std::convert::Infallible;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::PresentMode};
use powderkeg::{cell::{Action, Cell, Renderable}, chunk::{Chunk, ChunkBundle, ChunkCoords}, simulation::PowderkegTickRate, viewer::DrawStained, PowderkegPlugin};
use rand::{distributions::{Distribution, Uniform}, seq::IteratorRandom, thread_rng, Rng};

struct LiquidDistribution {
    pub air_chance: f64,
    pub densities: u32,
    pub spread: f32,
}

impl Distribution<MultiLiquidCell> for LiquidDistribution {
    fn sample<R: Rng + ?Sized>(&self, mut rng: &mut R) -> MultiLiquidCell {
        if rng.gen_bool(self.air_chance) {
            MultiLiquidCell::Air
        } else {
            let mut density = (1..=self.densities).choose(&mut rng).unwrap() as f32;

            density += rng.sample(Uniform::new(-self.spread, self.spread));

            density /= self.densities as f32 + 1.0;

            MultiLiquidCell::Liquid { density }
        }
    }
}

#[derive(Default)]
pub enum MultiLiquidCell {
    #[default]
    Air,
    Liquid {
        density: f32,
    },
}

impl MultiLiquidCell {
    pub fn density(&self) -> Option<f32> {
        match self {
            MultiLiquidCell::Air => None,
            MultiLiquidCell::Liquid { density } => Some(*density),
        }
    }
}

pub enum LiquidAction {
    Stable,
    Fall(IVec2),
}

impl Cell for MultiLiquidCell {
    type Action = LiquidAction;
    type Error = Infallible;

    fn tick(&self, origin: IVec2, grid: &impl powderkeg::grid::Grid<Cell = Self>) -> Result<Option<Self::Action>, Self::Error> {
        match self {
            MultiLiquidCell::Air => Ok(None),
            MultiLiquidCell::Liquid { density } => {
                let directions = &[
                    IVec2::new(0, -1),
                    IVec2::new(-1, -1),
                    IVec2::new(1, -1),
                    IVec2::new(-1, 0),
                    IVec2::new(1, 0),
                ];

                for direction in directions {
                    if matches!(grid.map(origin + *direction, |cell| {
                        match cell.density() {
                            Some(other) => *density >= other,
                            None => true,
                        }
                    }), Some(true)) {
                        return Ok(Some(LiquidAction::Fall(*direction)));
                    }
                }

                Ok(None)
            },
        }
    }

    fn range(&self) -> IRect {
        IRect::new(-1, -1, 1, 0)
    }
}

impl Action for LiquidAction {
    type Cell = MultiLiquidCell;
    type State = ();

    fn act(&self, origin: IVec2, grid: &mut impl powderkeg::stain::Stainable<Cell = Self::Cell, State = Self::State>) -> Option<()> {
        match self {
            LiquidAction::Stable => { grid.stain_around(origin, 1); },
            LiquidAction::Fall(offset) => {
                let new = origin + *offset;
                grid.stain_around(origin, 1);
                grid.stain_around(new, 1);

                grid.swap(origin, new);
            },
        }

        Some(())
    }
}

impl Renderable for MultiLiquidCell {
    fn to_color(&self, _: IVec2) -> Color {
        match self {
            MultiLiquidCell::Air => Color::BLACK,
            MultiLiquidCell::Liquid { density } => Color::rgb(*density, *density, *density),
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
        .add_plugins(PowderkegPlugin::<MultiLiquidCell, 64>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(PowderkegTickRate(64.0));

    let mut rng = thread_rng();

    commands
        .spawn(SpatialBundle {
            transform: Transform::default().with_scale(Vec3::splat(2.0)),
            ..Default::default()
        })
        .with_children(|children| {
            for cx in -4..=4 {
                for cy in -2..2 {
                    let chunk = Chunk::full_random(
                        &mut rng,
                        LiquidDistribution { densities: 3, air_chance: 0.1, spread: 0.3 },
                        ()
                    );

                    let chunk_coords = IVec2::new(cx, cy);
        
                    children.spawn((
                        ChunkBundle::<MultiLiquidCell, 64> {
                            chunk,
                            coords: ChunkCoords(chunk_coords),
                            transform: TransformBundle::from_transform(Transform::from_translation(chunk_coords.as_vec2().extend(0.0) * 64.0)),
                            ..default()
                        },
                        DrawStained,
                    ));
                }
            }
        });
}