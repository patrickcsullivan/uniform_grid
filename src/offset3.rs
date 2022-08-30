use std::ops::Add;

use serde::{Deserialize, Serialize};

/// Offset into a 3-dimensional grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Offset3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Offset3 {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Converts the 3-dimensional offset into a 1-dimensional index.
    ///
    /// The 3-dimensional offset is an offset from the grid's "origin cell" at
    /// `(0, 0, 0)`. The grid has has a finite size; `grid_size` specifies the
    /// width of the grid, in number of cells, in each dimension. The returned
    /// 1-dimensional index is a an index into a flat vector that contains
    /// the cells of a grid.
    ///
    /// If the offset references a cell that is outside the bounds of the grid,
    /// then this will return `None`.
    pub fn into_grid_index1(self, grid_size: (usize, usize, usize)) -> Option<usize> {
        if self.x >= 0
            && (self.x as usize) < grid_size.0
            && self.y >= 0
            && (self.y as usize) < grid_size.1
            && self.z >= 0
            && (self.z as usize) < grid_size.2
        {
            Some(
                (self.x as usize)
                    + (self.y as usize) * grid_size.0
                    + (self.z as usize) * grid_size.0 * grid_size.1,
            )
        } else {
            None
        }
    }

    /// Converts a 1-dimensional index into a 3-dimensional offset.
    ///
    /// The given 1-dimensional index is a an index into a flat vector that
    /// contains the cells of a grid. The returned 3-dimensional offset is an
    /// offset from the grid's "origin cell" at `(0, 0, 0)`.
    pub fn from_grid_index1(i: usize, grid_width_x: usize, grid_width_y: usize) -> Self {
        let x = i % grid_width_x;
        let y = (i / grid_width_x) % grid_width_y;
        let z = i / (grid_width_x * grid_width_y);
        Self::new(x as i64, y as i64, z as i64)
    }
}

impl Add for Offset3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<&Self> for Offset3 {
    type Output = Self;

    fn add(self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
