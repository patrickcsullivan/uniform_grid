use itertools::Itertools;

use crate::{
    bounding_box::BoundingBox,
    f32::{max_f32, min_f32},
    offset3::Offset3,
    point_object::PointObject,
    spiral_cells::{self, SpiralCell},
};

/// The uniform grid is a 3-dimensional grid of cube-shaped cells that covers a
/// finite region in infinite 3-dimensional space. Each cell is a container for
/// points that are positioned inside the space covered by the cell.
pub struct UniformGrid<T>
where
    T: PointObject,
{
    point_objs: Vec<T>,

    /// A flat vector that contains one element for each cell in the
    /// 3-dimensional grid. Each element contains a count of the number of
    /// points that are bucketed into that cell.
    cell_point_counts: Vec<usize>,

    /// A flat vector that contains one element for each cell in the
    /// 3-dimensional grid. Each element contains a vector of the points that
    /// are bucketed into that cell. Each point is represented by a tuple
    /// containing the point's position in 3-dimensional space and the point's
    /// index in `point_objs`.
    cell_point_positions: Vec<Vec<([f32; 3], usize)>>,

    /// The minimum position in space that is covered by the uniform grid.
    min_position: [f32; 3],

    // The width in space that is covered by each cube-shaped cell in the uniform grid.
    cell_width: f32,

    /// The number of cells in each dimension of the uniform grid.
    grid_dimensions: (usize, usize, usize),

    /// Vector of `SpiralCell`s that indicate which cells to check when
    /// searching for nearest neighbors outward from some center cell.
    spiral_cells: Vec<SpiralCell>,
}

impl<T> UniformGrid<T>
where
    T: PointObject,
{
    pub fn new(points: Vec<T>, scale: usize, spiral_cells: Vec<SpiralCell>) -> Self {
        // The maximum number of cells that the grid will be able to contain.
        let max_cell_count = points.len() * scale;

        let bb = BoundingBox::new(&points);

        // For simplicity we assume that we're constructing a uniform grid that has the
        // same number of cells in each dimension. To save space, we should allow
        // different widths in each dimension.
        let cube_bb_width = max_f32(bb.x_width, max_f32(bb.y_width, bb.z_width));
        // The max number of cells we can have in a single dimension while staying under
        // the max cell count.
        let cube_grid_width = (max_cell_count as f32).cbrt() as usize;
        let grid_dimensions = (cube_grid_width, cube_grid_width, cube_grid_width);

        // Make each cell slightly larger than is necessary to fit perfectly within the
        // bounding box so that points on a maximum face of the bounding box can fit
        // into a cell.
        let cell_width = cube_bb_width * 1.01 / cube_grid_width as f32;

        let cell_count = grid_dimensions.0 * grid_dimensions.1 * grid_dimensions.2;
        let mut cell_point_counts: Vec<usize> = vec![0; cell_count];
        for point in &points {
            let cell_index =
                point_into_index1(point.position(), bb.min, cell_width, grid_dimensions).unwrap();
            cell_point_counts[cell_index] += 1;
        }

        // Pre-allocate the necessary space for the vector in each cell so that the
        // vectors don't need to get re-allocated as new points are added.
        let mut cell_point_positions = cell_point_counts
            .iter()
            .map(|&count| Vec::with_capacity(count))
            .collect_vec();

        for (point_index, point) in points.iter().enumerate() {
            let cell_index =
                point_into_index1(point.position(), bb.min, cell_width, grid_dimensions).unwrap();
            cell_point_positions[cell_index].push((point.position(), point_index));
        }

        Self {
            point_objs: points,
            cell_point_counts,
            cell_point_positions,
            min_position: bb.min,
            cell_width,
            grid_dimensions,
            spiral_cells,
        }
    }

    /// Finds the point in the uniform grid that is closest to the given query
    /// point.
    ///
    /// Distance between points is Euclidean distance.
    pub fn nearest_neighbor(&self, query_point: [f32; 3]) -> Option<(&T, f32)> {
        let query_cell_offset = self.point_into_offset(query_point);
        self.nearest_neighbor_in_query_cell(query_point, query_cell_offset)
            .or_else(|| self.nearest_neighbor_spiral_search(query_point, query_cell_offset))
            .or_else(|| self.nearest_neighbor_brute_force(query_point))
            .map(|sr| {
                (
                    &self.point_objs[sr.point_object_index],
                    sr.distance2_to_query,
                )
            })
    }

    fn nearest_neighbor_in_query_cell(
        &self,
        query_point: [f32; 3],
        query_cell_offset: Offset3,
    ) -> Option<SearchResult> {
        self.offset_into_index1(query_cell_offset)
            .filter(|&query_cell_index| self.cell_point_counts[query_cell_index] > 0)
            .map(|query_cell_index| {
                // We know there is at least one point in the cell so this is ok.
                let nearest_in_query_cell =
                    nearest(query_point, &self.cell_point_positions[query_cell_index]).unwrap();

                let dist_to_wall =
                    self.nearest_wall_dist(nearest_in_query_cell.position, query_cell_offset);
                if dist_to_wall * dist_to_wall > nearest_in_query_cell.distance2_to_query {
                    // The neighbor is closer than any of the cell walls, so no need to search in
                    // other cells.
                    nearest_in_query_cell
                } else {
                    // Check the neighboring cells for points that might be closer.
                    let maybe_nearest_in_neighbor_cells = self.nearest_in_cell_offsets(
                        query_point,
                        query_cell_offset,
                        neighbor_offsets(),
                    );

                    if let Some(nearest_in_neighbor_cells) = maybe_nearest_in_neighbor_cells {
                        if nearest_in_query_cell.distance2_to_query
                            <= nearest_in_neighbor_cells.distance2_to_query
                        {
                            nearest_in_query_cell
                        } else {
                            nearest_in_neighbor_cells
                        }
                    } else {
                        nearest_in_query_cell
                    }
                }
            })
    }

    fn nearest_neighbor_spiral_search(
        &self,
        query_point: [f32; 3],
        query_cell_offset: Offset3,
    ) -> Option<SearchResult> {
        // Use the sprial cells to spiral out and check points in each batch of cells
        // that are equidistanct from the center cell until...
        // - a first point is found in some cell, and then that cell's stop cell is
        //   reached
        // - or all spiral cells are exhausted
        let mut maybe_stop_cell_index1: Option<usize> = None;
        let mut maybe_nearest_so_far: Option<SearchResult> = None;

        // Skip the first spiral cell, which is always (0, 0, 0), since that cell is
        // checked before attempting spiral search.
        for (spiral_cell_index1, spiral_cell) in self.spiral_cells.iter().enumerate().skip(1) {
            // Terminate after the stop cell is checked.
            if let Some(stop_cell_index1) = maybe_stop_cell_index1 {
                if spiral_cell_index1 > stop_cell_index1 {
                    break;
                }
            }

            // Look for the nearest point in the next batch of cells that are equidistant
            // from the center cell.
            let maybe_nearest_in_spiral_cell = self.nearest_in_cell_offsets(
                query_point,
                query_cell_offset,
                spiral_cells::offset_variations(spiral_cell.offset),
            );

            if let Some(nearest_in_spiral_cell) = maybe_nearest_in_spiral_cell {
                // A point has been found, so we don't need to search past the stop cell.
                if maybe_stop_cell_index1.is_none() {
                    maybe_stop_cell_index1 = Some(spiral_cell.stop_cell_index1);
                }

                // Check if the point that's found is the new nearest neighbor.
                let is_new_nearest = match &maybe_nearest_so_far {
                    None => true,
                    Some(nearest_so_far) => {
                        nearest_in_spiral_cell.distance2_to_query
                            < nearest_so_far.distance2_to_query
                    }
                };

                // Update the nearest neighbor.
                if is_new_nearest {
                    maybe_nearest_so_far = Some(nearest_in_spiral_cell)
                }
            }
        }

        maybe_nearest_so_far
    }

    fn nearest_neighbor_brute_force(&self, query_point: [f32; 3]) -> Option<SearchResult> {
        nearest(query_point, self.cell_point_positions.iter().flatten())
    }

    /// Returns the distance between the point and the nearest wall of the cell
    /// that contains the point.
    ///
    /// The 3-dimensional offset, `cell_offset`, is relative to the uniform
    /// grid's "origin cell" at `(0, 0, 0)`.
    fn nearest_wall_dist(&self, point: [f32; 3], cell_offset: Offset3) -> f32 {
        let dist_to_x_wall = min_f32(
            point[0] - (cell_offset.x as f32 * self.cell_width),
            (cell_offset.x + 1) as f32 * self.cell_width - point[0],
        );
        let dist_to_y_wall = min_f32(
            point[1] - (cell_offset.y as f32 * self.cell_width),
            (cell_offset.y + 1) as f32 * self.cell_width - point[1],
        );
        let dist_to_z_wall = min_f32(
            point[2] - (cell_offset.z as f32 * self.cell_width),
            (cell_offset.z + 1) as f32 * self.cell_width - point[1],
        );
        min_f32(dist_to_x_wall, min_f32(dist_to_y_wall, dist_to_z_wall))
    }

    /// Returns the 3-dimensional offset of the cell in which the point would be
    /// bucketed.
    ///
    /// The 3-dimensional offset is relative to the uniform grid's "origin cell"
    /// at `(0, 0, 0)`. The uniform grid has a finite width in each
    /// dimension, so the offset may refer to a "cell" that doesn't actually
    /// exist. This will happen if the given point lies outside the region
    /// of space that is covered by the uniform grid.
    fn point_into_offset(&self, point: [f32; 3]) -> Offset3 {
        point_into_offset(point, self.min_position, self.cell_width)
    }

    /// Converts the 3-dimensional offset of a cell in the uniform grid into an
    /// index into the 1-dimensional vector that stores the cells of the uniform
    /// grid.
    ///
    /// The 3-dimensional offset is relative to the uniform grid's "origin cell"
    /// at `(0, 0, 0)`.
    ///
    /// This returns `None` if the offset refers to a "cell" that doesn't exist
    /// because it is outside the finite bounds of the uniform grid.
    fn offset_into_index1(&self, offset_from_origin: Offset3) -> Option<usize> {
        offset_from_origin.into_grid_index1(self.grid_dimensions)
    }

    /// Checks each of the cells that are identified by the offsets from the
    /// center cell, and return the point in those cells that is nearest to the
    /// query point.
    fn nearest_in_cell_offsets(
        &self,
        query_point: [f32; 3],
        center_cell_offset: Offset3,
        cell_offsets: Vec<Offset3>,
    ) -> Option<SearchResult> {
        let points = cell_offsets
            .iter()
            .filter_map(|o| self.offset_into_index1(center_cell_offset + o))
            .flat_map(|i| &self.cell_point_positions[i]);
        nearest(query_point, points)
    }
}

struct SearchResult {
    pub position: [f32; 3],
    pub point_object_index: usize,
    pub distance2_to_query: f32,
}

fn neighbor_offsets() -> Vec<Offset3> {
    vec![
        Offset3::new(-1, -1, -1),
        Offset3::new(0, -1, -1),
        Offset3::new(1, -1, -1),
        Offset3::new(-1, 0, -1),
        Offset3::new(0, 0, -1),
        Offset3::new(1, 0, -1),
        Offset3::new(-1, 1, -1),
        Offset3::new(0, 1, -1),
        Offset3::new(1, 1, -1),
        Offset3::new(-1, -1, 0),
        Offset3::new(0, -1, 0),
        Offset3::new(1, -1, 0),
        Offset3::new(-1, 0, 0),
        Offset3::new(1, 0, 0),
        Offset3::new(-1, 1, 0),
        Offset3::new(0, 1, 0),
        Offset3::new(1, 1, 0),
        Offset3::new(-1, -1, 1),
        Offset3::new(0, -1, 1),
        Offset3::new(1, -1, 1),
        Offset3::new(-1, 0, 1),
        Offset3::new(0, 0, 1),
        Offset3::new(1, 0, 1),
        Offset3::new(-1, 1, 1),
        Offset3::new(0, 1, 1),
        Offset3::new(1, 1, 1),
    ]
}

fn point_into_offset(point: [f32; 3], min_point: [f32; 3], cell_width: f32) -> Offset3 {
    let relative_pos = [
        point[0] - min_point[0],
        point[1] - min_point[1],
        point[2] - min_point[2],
    ];
    let x = (relative_pos[0] / cell_width) as i64;
    let y = (relative_pos[1] / cell_width) as i64;
    let z = (relative_pos[2] / cell_width) as i64;
    Offset3::new(x, y, z)
}

fn point_into_index1(
    point: [f32; 3],
    min_point: [f32; 3],
    cell_width: f32,
    grid_size: (usize, usize, usize),
) -> Option<usize> {
    point_into_offset(point, min_point, cell_width).into_grid_index1(grid_size)
}

fn nearest<'a, I>(query_point: [f32; 3], points: I) -> Option<SearchResult>
where
    I: IntoIterator<Item = &'a ([f32; 3], usize)>,
{
    points
        .into_iter()
        .map(|(p, p_obj_idx)| SearchResult {
            position: *p,
            point_object_index: *p_obj_idx,
            distance2_to_query: dist2(query_point, *p),
        })
        .min_by(|sr1, sr2| {
            sr1.distance2_to_query
                .partial_cmp(&sr2.distance2_to_query)
                .unwrap()
        })
}

fn dist2(p: [f32; 3], q: [f32; 3]) -> f32 {
    let x = q[0] - p[0];
    let y = q[1] - p[1];
    let z = q[2] - p[2];
    x * x + y * y + z * z
}
