pub mod grid;
pub mod chunk;
pub mod stain;
pub mod cell;
pub mod simulation;
pub mod viewer;
pub mod area;

use std::marker::PhantomData;

use bevy::prelude::*;
use cell::{Cell, Renderable};
use simulation::PowderkegSimulationPlugin;
use thiserror::Error;
use viewer::PowderkegViewPlugin;

#[derive(Debug, Error)]
pub enum PowderkegError<T: Cell> {
    #[error(transparent)]
    Cell(T::Error),
    #[error("chunk local {0} out of bounds")]
    LocalOutOfBounds(IVec2),
    #[error("chunk at {0} out of bounds")]
    ChunkOutOfBounds(IVec2),
    #[error("chunks not found when swapping {first} -> {second}")]
    SwapOutOfBounds {
        first: IVec2,
        second: IVec2,
    },
}

pub struct PowderkegPlugin<T, const N: i32>(PhantomData<T>);

impl<T, const N: i32> Default for PowderkegPlugin<T, N>
where
    T: Renderable,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, const N: i32> Plugin for PowderkegPlugin<T, N>
where
    T: Renderable,
{
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PowderkegViewPlugin::<T, N>::default())
            .add_plugins(PowderkegSimulationPlugin::<T, N>::default())
            .configure_sets(Update, (PowderkegSet::Tick, PowderkegSet::Render).chain()); 
    }
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PowderkegSet {
    Tick,
    Render,
}
