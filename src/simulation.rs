use std::{marker::PhantomData, mem::swap, sync::Arc};

use bevy::{prelude::*, utils::HashMap};
use crossbeam_channel::unbounded;
use parking_lot::RwLock;

use crate::{cell::{Cell, Renderable, TickInput, TickSuccess}, chunk::{Chunk, ChunkCoords}, grid::Grid, stain::{Stain, Stainable}, PowderkegError, PowderkegSet};

pub(crate) struct PowderkegSimulationPlugin<T: Renderable + Send + Sync + 'static, const N: i32>(PhantomData<T>);

impl<T, const N: i32> Default for PowderkegSimulationPlugin<T, N>
where
    T: Renderable,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, const N: i32> Plugin for PowderkegSimulationPlugin<T, N>
where
    T: Renderable,
{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PowderkegTickRate>()
            .add_systems(Update, simulate_powderkeg::<T, N>.in_set(PowderkegSet::Tick));
    }
}

#[derive(Resource)]
pub struct PowderkegTickRate(pub f32);

impl Default for PowderkegTickRate {
    fn default() -> Self {
        Self(16.0)
    }
}

struct WorldGrid<'c, T, const N: i32>
where
    T: Renderable,
{
    chunks: HashMap<IVec2, &'c mut Chunk<T, N>>,
}

impl<'c, T, const N: i32> Grid for WorldGrid<'c, T, N>
where
    T: Renderable,
{
    type Cell = T;

    fn get(&self, point: IVec2) -> Option<&Self::Cell> {
        let (chunk, local) = ChunkCoords::<N>::world_to_chunk_and_local(point);

        self.chunks.get(&chunk)?.get(local)
    }

    fn get_mut(&mut self, point: IVec2) ->Option<&mut Self::Cell> {
        let (chunk, local) = ChunkCoords::<N>::world_to_chunk_and_local(point);

        self.chunks.get_mut(&chunk)?.get_mut(local)
    }

    fn swap(&mut self, first: IVec2, second: IVec2) -> Option<()> {
        let (first_chunk, first_local) = ChunkCoords::<N>::world_to_chunk_and_local(first);
        let (second_chunk, second_local) = ChunkCoords::<N>::world_to_chunk_and_local(second);

        if first_chunk == second_chunk {
            self.chunks.get_mut(&first_chunk)?.swap(first_local, second_local)
        } else {
            let [first_chunk, second_chunk] = self.chunks.get_many_mut([&first_chunk, &second_chunk])?;

            let first_cell = first_chunk.get_mut(first_local)?;
            let second_cell = second_chunk.get_mut(second_local)?;

            swap(first_cell, second_cell);

            Some(())
        }
    }

    fn get_state(&self, point: IVec2) -> Option<Arc<RwLock<<T as Cell>::State>>> {
        let (chunk, local) = ChunkCoords::<N>::world_to_chunk_and_local(point);

        self.chunks.get(&chunk)?.get_state(local)
    }
}


// TODO: Fix this mess of an implementation
impl<'c, T, const N: i32> Stainable for WorldGrid<'c, T, N>
where
    T: Renderable,
{
    fn stained(&self) -> Stain {
        Stain::from_stains(self.chunks.values().map(|chunk| chunk.stained()))
    }

    fn stain(&mut self, area: IRect) {
        let (min_chunk, _) = ChunkCoords::<N>::world_to_chunk_and_local(area.min);
        let (max_chunk, _) = ChunkCoords::<N>::world_to_chunk_and_local(area.max);
        
        for cx in min_chunk.x..=max_chunk.x {
            for cy in min_chunk.y..=max_chunk.y {
                let chunk_coords = IVec2::new(cx, cy);

                if let Some(chunk) = self.chunks.get_mut(&chunk_coords) {
                    let translated = translate_rect(area, -N * chunk_coords);
                    chunk.stain(translated);
                }
            }
        }
    }

    fn stain_point(&mut self, point: IVec2) {
        let (chunk, local) = ChunkCoords::<N>::world_to_chunk_and_local(point);
        
        if let Some(chunk) = self.chunks.get_mut(&chunk) {
            chunk.stain_point(local);
        }
    }

    fn clear_stain(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.clear_stain();
        }
    }
}

struct SimulationError<T: Cell> {
    pub point: IVec2,
    pub error: PowderkegError<T>,
}

fn simulate_powderkeg<T, const N: i32>(
    mut chunks: Query<(&ChunkCoords<N>, &mut Chunk<T, N>)>,
    tick_rate: Res<PowderkegTickRate>,
    mut ticks: Local<f32>,
    time: Res<Time<Virtual>>,
) where
    T: Renderable,
{
    *ticks += tick_rate.0 * time.delta_seconds();

    if *ticks >= 1.0 {
        let (send_to_tick, recieve_to_tick) = unbounded::<IVec2>();
        let (send_errors, recieve_errors) = unbounded::<SimulationError<T>>();
        let (send_stains, recieve_stains) = unbounded::<IRect>();

        chunks.par_iter_mut().for_each(|(coords, mut chunk)| {
            let area = Chunk::<T, N>::area();

            let stain = chunk.stained();

            chunk.clear_stain();

            stain.apply_randomly(|point| {
                let range = {
                    let cell = chunk.at(point);

                    translate_rect(cell.range(), point)
                };

                if area.contains(range.min) && area.contains(range.max) {
                    let input = TickInput {
                        origin: point,
                        grid: chunk.as_mut(),
                    };

                    match T::tick(input) {
                        Ok(TickSuccess::Unstable) => {
                            chunk.stain_point(point);
                        },
                        Err(error) => {
                            let error = SimulationError { point: coords.local_to_world(point), error };
                            send_errors.send(error).expect("channel unexpectedly closed");
                        },
                        _ => {},
                    }
                } else {
                    send_to_tick.send(coords.local_to_world(point)).expect("channel unexpectedly closed");
                }
            });

            if let Some(stain) = chunk.stain.as_ref() {
                let area = Chunk::<T, N>::area();

                if !(area.contains(stain.min) && area.contains(stain.max)) {
                    send_stains.send(translate_rect(*stain, N * coords.0)).expect("channel unexpectedly closed");
                }
            }
        });

        drop(send_to_tick);
        drop(send_errors);
        drop(send_stains);

        for SimulationError { point, error } in recieve_errors.iter() {
            error!("Error ticking {point}: {error}");
        }
        
        let chunks = chunks
            .iter_mut()
            .map(|(ChunkCoords(coords), chunk)| (*coords, chunk.into_inner()))
            .collect();

        let mut world_grid = WorldGrid {
            chunks,
        };

        for stain in recieve_stains.iter() {
            world_grid.stain(stain);
        }

        for point in recieve_to_tick.iter() {
            // TODO: Add bounds checking for world

            let input = TickInput {
                origin: point,
                grid: &mut world_grid,
            };

            match T::tick(input) {
                Ok(TickSuccess::Unstable) => {
                    world_grid.stain_point(point);
                },
                Err(error) => {
                    error!("Error ticking {point}: {error}");
                },
                _ => {},
            }
        }

        *ticks = f32::clamp(*ticks - 1.0, 0.0, 1.0);
    }   
}

fn translate_rect(rect: IRect, offset: IVec2) -> IRect {
    IRect { min: rect.min + offset, max: rect.max + offset }
}
