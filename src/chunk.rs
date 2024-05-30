use std::{iter, sync::Arc};

use bevy::prelude::*;
use parking_lot::RwLock;
use rand::{distributions::Distribution, Rng};

use crate::{cell::{Cell, Renderable}, grid::Grid, stain::{Stain, Stainable}};

#[derive(Component)]
pub struct Chunk<T: Cell, const N: i32> {
    data: Vec<T>,
    pub(crate) stain: Option<IRect>,
    state: Arc<RwLock<T::State>>,
}

#[derive(Component, Default)]
pub struct ChunkCoords<const N: i32>(pub IVec2);

impl<const N: i32> ChunkCoords<N> {
    pub fn offset(&self) -> IVec2 {
        N * self.0
    }

    pub fn local_to_world(&self, local: IVec2) -> IVec2 {
        self.offset() + local
    }

    pub fn world_to_local(&self, world: IVec2) -> IVec2 {
        world - self.offset()
    }

    pub fn world_to_chunk_and_local(world: IVec2) -> (IVec2, IVec2) {
        (world.div_euclid(IVec2::splat(N)), world.rem_euclid(IVec2::splat(N)))
    }
}

#[derive(Bundle)]
pub struct ChunkBundle<T, const N: i32>
where
    T: Renderable + Send + Sync + 'static,
    T::State: Send + Sync + 'static,
{
    pub chunk: Chunk<T, N>,
    pub coords: ChunkCoords<N>,
    pub transform: TransformBundle,
    pub visibility: VisibilityBundle,
}

impl<T, const N: i32> Default for ChunkBundle<T, N>
where
    T: Renderable + Default + Send + Sync + 'static,
    T::State: Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self { chunk: Default::default(), coords: Default::default(), transform: Default::default(), visibility: Default::default() }
    }
}

impl<T, const N: i32> Chunk<T, N>
where
    T: Cell,
{
    pub fn new(data: Vec<T>, state: T::State) -> Self {
        assert_eq!(data.len(), N as usize * N as usize);

        Self { data, stain: Some(Self::area()), state: Arc::new(RwLock::new(state)) }
    }

    pub const fn area() -> IRect {
        IRect { min: IVec2::splat(0), max: IVec2::splat(N - 1) }
    }

    pub const fn volume() -> usize {
        N as usize * N as usize
    }

    pub fn index(&self, point: IVec2) -> Option<usize> {
        if !Self::area().contains(point) {
            None
        } else {
            Some((N * point.y + point.x) as usize)
        }
    }
}

impl<T, const N: i32> Chunk<T, N> 
where
    T: Cell + Copy,
{
    pub fn full_copied(value: T, state: T::State) -> Self {
        Self::new(vec![value; Self::volume()], state)
    }
}

impl<T, const N: i32> Chunk<T, N>
where
    T: Cell,
{
    pub fn full_random<R: Rng, D: Distribution<T>>(rng: &mut R, distribution: D, state: T::State) -> Self {
        Self::new(rng.sample_iter(distribution).take(Self::volume()).collect(), state)
    }
}

impl<T, const N: i32> Default for Chunk<T, N> 
where
    T: Cell + Default,
    T::State: Default,
{
    fn default() -> Self {
        Self::new(iter::repeat_with(|| T::default()).take(Self::volume()).collect(), T::State::default())
    }
}

impl<T, const N: i32> Grid for Chunk<T, N>
where
    T: Cell,
{
    type Cell = T;

    fn get(&self, point: IVec2) -> Option<&Self::Cell> {
        let index = self.index(point)?;

        self.data.get(index)
    }

    fn get_mut(&mut self, point: IVec2) ->Option<&mut Self::Cell> {
        let index = self.index(point)?;
        
        self.stain_point(point);

        self.data.get_mut(index)
    }

    fn swap(&mut self, first: IVec2, second: IVec2) -> Option<()> {
        let first_index = self.index(first)?;
        let second_index = self.index(second)?;

        self.stain_point(first);
        self.stain_point(second);

        self.data.swap(first_index, second_index);

        Some(())
    }
    
    fn get_state(&self, point: IVec2) -> Option<Arc<RwLock<<T as Cell>::State>>> {
        if Self::area().contains(point) {
            Some(self.state.clone())
        } else {
            None
        }
    }
}

impl<T, const N: i32> Stainable for Chunk<T, N> 
where
    T: Cell,
{
    fn stained(&self) -> Stain {
        match self.stain {
            Some(area) => area.intersect(Self::area()).into(),
            None => Stain::Empty,
        }
    }

    fn clear_stain(&mut self) {
        self.stain = None;
    }

    fn stain(&mut self, area: IRect) {
        match &mut self.stain {
            Some(stain) => *stain = stain.union(area),
            stain @ None => *stain = Some(area),
        }
    }

    fn stain_point(&mut self, point: IVec2) {
        match &mut self.stain {
            Some(stain) => *stain = stain.union_point(point),
            stain @ None => *stain = Some(IRect::from_corners(point, point)),
        }
    }
}
