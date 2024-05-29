use bevy::prelude::*;

use crate::{grid::Grid, stain::Stainable};

pub trait Cell {
    type Action: Action<Cell = Self>;
    type Error: std::error::Error;

    fn tick(&self, origin: IVec2, grid: &impl Grid<Cell = Self>) -> Result<Option<Self::Action>, Self::Error>;
    fn range(&self) -> IRect;
}

pub trait Action {
    type Cell: Cell<Action = Self>;
    type State;

    fn act(&self, origin: IVec2, grid: &mut impl Stainable<Cell = Self::Cell, State = Self::State>) -> Option<()>;
}

pub trait Renderable: Cell {
    fn to_color(&self, point: IVec2) -> Color;
}
