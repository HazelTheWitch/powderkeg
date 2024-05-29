pub mod grid;
pub mod chunk;
pub mod stain;
pub mod cell;
pub mod simulation;
pub mod viewer;

use std::marker::PhantomData;

use bevy::prelude::*;
use cell::{Action, Renderable};
use simulation::PowderkegSimulationPlugin;
use viewer::PowderkegViewPlugin;

pub struct PowderkegPlugin<T, const N: i32>(PhantomData<T>);

impl<T, const N: i32> Default for PowderkegPlugin<T, N>
where
    T: Renderable + Send + Sync + 'static,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, const N: i32> Plugin for PowderkegPlugin<T, N>
where
    T: Renderable + Send + Sync + 'static,
    <T::Action as Action>::State: Send + Sync + 'static,
    T::Action: Send + Sync + 'static,
    T::Error: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PowderkegViewPlugin::<T, N>::default())
            .add_plugins(PowderkegSimulationPlugin::<T, N>::default())
            .configure_sets(Update, (PowderkegSet::Sync, PowderkegSet::Tick, PowderkegSet::Render).chain()); 
    }
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PowderkegSet {
    Sync,
    Tick,
    Render,
}
