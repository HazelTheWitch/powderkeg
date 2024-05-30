use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::RwLock;

use crate::{stain::Stainable, PowderkegError};

pub enum TickSuccess {
    Stable,
    Unstable,
}

pub struct TickInput<'g, T: Cell, G: Stainable<Cell = T>> {
    pub origin: IVec2,
    pub grid: &'g mut G,
}

impl<'g, T, G> TickInput<'g, T, G> 
where
    T: Cell,
    G: Stainable<Cell = T>,
{
    pub fn get_this(&self) -> Result<&T, PowderkegError<T>> {
        self.grid.get(self.origin)
    }

    pub fn get_this_mut(&mut self) -> Result<&mut T, PowderkegError<T>> {
        self.grid.get_mut(self.origin)
    }

    pub fn get_state(&self) -> Result<Arc<RwLock<T::State>>, PowderkegError<T>> {
        self.grid.get_state(self.origin)
    }

    pub fn this(&self) -> &T {
        self.grid.at(self.origin)
    }

    pub fn this_mut(&mut self) -> &mut T {
        self.grid.at_mut(self.origin)
    }

    pub fn state(&self) -> Arc<RwLock<T::State>> {
        self.grid.state_at(self.origin)
    }
}

pub trait Cell: Send + Sync + Sized + 'static {
    type State: Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    fn tick<G: Stainable<Cell = Self>>(input: TickInput<'_, Self, G>) -> Result<TickSuccess, PowderkegError<Self>>;
    fn range(&self) -> IRect;
}

pub trait Renderable
where
    Self: Cell,
{
    fn to_color(&self, point: IVec2) -> Color;
}
