//! A polyomino packing problem.

use std::error::Error;
use std::fmt::Display;
use indexmap::{IndexMap, IndexSet};
use crate::problem::{Problem, Value};
use crate::vector::Vector2D;

// Orientation
// ===========

/// An orientation of a piece.
/// 
/// Reflection is applied first, then rotation.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Orientation {
    pub reflection: bool,
    pub rotation: i32, // 0..4
}


// Polyomino
// =========

/// A polyomino piece, possibly with disconnected cells.
/// 
/// The coordinates are normalized upon creation,
/// so it does not contain translation information.
#[derive(PartialEq, Eq, Hash, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Polyomino {
    cells: Vec<Vector2D>,
    size: Vector2D,
    // TODO: add flags to configure the piece (e.g. rotation, reflection, etc.)
}

/// An error returned when an invalid piece is given.
/// 
/// Currently, it only occurs when the given piece is blank.
#[derive(Debug)]
pub struct InvalidPieceError;

impl Display for InvalidPieceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid (blank) piece.")
    }
}

impl Error for InvalidPieceError {}

impl Polyomino {
    /// Creates a new `Polyomino` from a list of cell positions.
    /// 
    /// The coordinates are normalized upon creation,
    /// i.e., the minimums of x/y coordinates are set to 0.
    /// It returns an error if the given list is empty.
    pub fn new(cells: &[Vector2D]) -> Result<Polyomino, InvalidPieceError> {
        if cells.len() == 0 { return Err(InvalidPieceError) }

        let min_x = cells.iter().map(|c| { c.x }).min().unwrap();
        let max_x = cells.iter().map(|c| { c.x }).max().unwrap();
        let min_y = cells.iter().map(|c| { c.y }).min().unwrap();
        let max_y = cells.iter().map(|c| { c.y }).max().unwrap();

        let mut normalized_cells: Vec<_> = cells.iter()
            .map(|&c| { c - Vector2D { x: min_x, y: min_y } })
            .collect();
        normalized_cells.sort();

        Ok(Polyomino {
            cells: normalized_cells,
            size: Vector2D {
                x: max_x - min_x + 1,
                y: max_y - min_y + 1,
            }
        })
    }

    /// Convenience function to create a new `Polyomino` from a bytes array.
    /// 
    /// It uses the inverted y-axis coordinate system.
    /// `#` represents a cell in the piece.
    /// If `array[y][x]` is `#`, then the cell `(x, y)` is a corresponding cell in the piece.
    /// Any other byte represents an empty cell. (usually `.`)
    pub fn from_bytes_array(array: &[&[u8]]) -> Result<Polyomino, InvalidPieceError> {
        let mut cells = Vec::new();

        for y in 0..array.len() {
            for x in 0..array[y].len() {
                if array[y][x] == b'#' {
                    cells.push(Vector2D { x: x as i32, y: y as i32 });
                }
            }
        }

        Polyomino::new(&cells)
    }

    /// Returns the list of cells in the piece.
    pub fn cells(&self) -> &Vec<Vector2D> { &self.cells }
    /// Returns the size of the bounding box.
    pub fn size(&self) -> Vector2D { self.size }
    
    /// Orients the piece according to the given orientation.
    /// Reflection is applied first, then rotation.
    pub fn orient(&self, orientation: Orientation) -> Polyomino {
        match orientation.reflection {
            false => self.rotate(orientation.rotation),
            true => self.reflect().rotate(orientation.rotation),
        }
    }

    /// Reflects the piece in the y axis.
    pub fn reflect(&self) -> Polyomino {
        let reflected: Vec<_> = self.cells.iter()
            .map(|&Vector2D { x, y }| { Vector2D { x: -x, y } })
            .collect();

        Polyomino::new(&reflected).unwrap()
    }

    /// Rotates the piece as specified amount.
    pub fn rotate(&self, rotation: i32) -> Polyomino {
        let rotated: Vec<_> = self.cells.iter()
            .map(|c| { c.rotate(rotation) })
            .collect();

        Polyomino::new(&rotated).unwrap()
    }

    /// Returns possible orientations of the piece without duplication.
    pub fn unique_orientations(&self) -> Vec<Orientation> {
        let mut pieces = IndexSet::new();
        let mut res = Vec::new();
        
        for reflection in [false, true] {
            for rotation in 0..4 {
                let o = Orientation { reflection, rotation };
                let piece = self.orient(o);
                if !pieces.contains(&piece) {
                    pieces.insert(piece);
                    res.push(o);
                }
            }
        }
        res
    }

    /// Returns the list of cells after translation.
    pub fn translated_cells(&self, trans: Vector2D) -> Vec<Vector2D> {
        self.cells.iter()
            .map(|&c| { c + trans })
            .collect()
    }
}


// Board
// ========

/// A board to fit the pieces in.
#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Board {
    cells: Vec<Vec<bool>>,
    size: Vector2D,
}

impl Board {
    /// Creates a new board from a 2D boolean list.
    /// 
    /// It uses the inverted y-axis coordinate system.
    /// `true` corresponds to an empty cell where pieces can fit in.
    /// If `cells[y][x]` is true, then the cell `(x, y)` is a corresponding empty cell in the board.
    pub fn new(cells: Vec<Vec<bool>>) -> Board {
        // TODO: validate parameter
        assert_eq!(cells.len() > 0, true);
        
        Board {
            size: Vector2D {
                y: cells.len() as i32,
                x: cells[0].len() as i32,
            },
            cells,
        }
    }

    /// Convenience function to create a new `Board` from a bytes array.
    /// 
    /// It uses the inverted y-axis coordinate system.
    /// `.` represents an empty cell to fit pieces in.
    /// Any other byte represents an unused cell. (usually `#`)
    pub fn from_bytes_array(array: &[&[u8]]) -> Board {
        let cells: Vec<Vec<bool>> = array.iter().map(|&s| {
            s.iter().map(|&c| { c == b'.' }).collect()
        }).collect();

        Board::new(cells)
    }

    /// Returns a 2D boolean list representing this board.
    /// 
    /// It uses the inverted y-axis coordinate system.
    /// `true` corresponds to an empty cell where pieces can fit in.
    /// If `cells[y][x]` is true, then the cell `(x, y)` is a corresponding empty cell in the board.
    pub fn cells(&self) -> &Vec<Vec<bool>> { &self.cells }
    /// Returns the size of the board.
    pub fn size(&self) -> Vector2D { self.size }

    /// Returns whether the given piece can fit in the board
    /// with specified orientation and translation.
    pub fn piece_fits(
        &self,
        piece: &Polyomino,
        orien: Orientation,
        trans: Vector2D,
    ) -> bool {
        for c in piece.orient(orien).translated_cells(trans) {
            let Vector2D { x, y } = c;
            if self.out_of_bounds(c) { return false }
            if !self.cells[y as usize][x as usize] { return false }
        }
        true
    }

    fn out_of_bounds(&self, Vector2D { x, y }: Vector2D) -> bool {
        x < 0 || x >= self.size.x || y < 0 || y >= self.size.y
    }
}


// Problem
// =========

/// An identifier of a piece placed in a specified orientation and translation.
/// It is used as a subset name of [`Problem`] instance.
pub type CompoundName<N> = (N, Orientation, Vector2D);

/// An exact cover constraint for polyomino packing problem.
#[derive(PartialEq, Eq, Clone, Hash)]
#[cfg_attr(test, derive(Debug))]
pub enum CompoundConstraint<N> {
    Piece(N),
    Cell(Vector2D),
}

/// A polyomino packing problem.
#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct PolyominoPacking<N: Value> {
    board: Board,
    pieces: IndexMap<N, Polyomino>,
}

impl<N: Value> PolyominoPacking<N> {
    // TODO: hide IndexMap/IndexSet from API
    /// Returns a reference to the board.
    pub fn board(&self) -> &Board { &self.board }
    /// Returns a mutable reference to the board.
    pub fn board_mut(&mut self) -> &mut Board { &mut self.board }
    /// Returns a reference to the pieces.
    pub fn pieces(&self) -> &IndexMap<N, Polyomino> { &self.pieces }
    /// Returns a mutable reference to the pieces.
    pub fn pieces_mut(&mut self) -> &mut IndexMap<N, Polyomino> { &mut self.pieces }

    /// Adds a piece to the problem.
    /// 
    /// If the piece name already exists,
    /// it updates the piece of that name with the given new piece.
    pub fn add_piece(&mut self, name: N, piece: Polyomino) {
        self.pieces.insert(name, piece);
    }

    /// Generates an exact cover problem instance ([`Problem`]).
    pub fn generate_problem(&self) -> Problem<CompoundName<N>, CompoundConstraint<N>> {
        let mut prob = Problem::<CompoundName<N>, CompoundConstraint<N>>::default();

        // Piece constraints
        for (name, _) in &self.pieces {
            prob.add_constraint(CompoundConstraint::Piece(name.clone()));
        }

        // Cell contraints
        for y in 0..self.board.size.y {
            for x in 0..self.board.size.x {
                if self.board.cells[y as usize][x as usize] {
                    prob.add_constraint(CompoundConstraint::Cell(Vector2D { x, y }));
                }
            }
        }

        // Subsets
        for (name, piece) in &self.pieces {
            for o in piece.unique_orientations() {
                let p = piece.orient(o);
                for y in 0..=(self.board.size.y - p.size.y) {
                    for x in 0..=(self.board.size.x - p.size.x) {
                        let t = Vector2D { x, y };
                        if !self.board.piece_fits(&piece, o, t) { continue }
                        
                        let compound_name = (name.clone(), o, t);
                        let subset = Self::generate_subset(name.clone(), &p, t);
                        
                        prob.add_subset(compound_name, subset);
                    }
                }
            }
        }

        prob
    }

    fn generate_subset(
        name: N,
        oriented_piece: &Polyomino,
        trans: Vector2D,
    ) -> Vec<CompoundConstraint<N>> {
        let mut subset = Vec::new();
        subset.push(CompoundConstraint::Piece(name));
        subset.extend(
            oriented_piece.translated_cells(trans).iter()
                .map(|&c| { CompoundConstraint::Cell(c) })
        );
        subset
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;
    use crate::{Solver, SolverEvent};

    fn compare_unique_orientations(piece: &Polyomino, expected: &[(bool, i32)]) {
        assert_eq!(
            piece.unique_orientations().into_iter().collect::<HashSet<_>>(),
            expected.iter()
                .map(|&(f, r)| { Orientation { reflection: f, rotation: r } })
                .collect::<HashSet<_>>()
        )
    }

    #[test]
    fn unique_orientations_can_be_found() {
        let tetro_l = Polyomino::from_bytes_array(&[b".#.", b".#.", b".##"]).unwrap();
        compare_unique_orientations(&tetro_l, &[
            (false, 0), (false, 1), (false, 2), (false, 3),
            (true, 0), (true, 1), (true, 2), (true, 3),
        ]);

        let tetro_s = Polyomino::from_bytes_array(&[b"...", b".##", b"##."]).unwrap();
        compare_unique_orientations(&tetro_s, &[
            (false, 0), (false, 1), (true, 0), (true, 1),
        ]);

        let tetro_o = Polyomino::from_bytes_array(&[b"...", b".##", b".##"]).unwrap();
        compare_unique_orientations(&tetro_o, &[
            (false, 0),
        ]);

        let pento_w = Polyomino::from_bytes_array(&[b"..#", b".##", b"##."]).unwrap();
        compare_unique_orientations(&pento_w, &[
            (false, 0), (false, 1), (false, 2), (false, 3),
        ]);
    }

    #[test]
    fn problem_can_be_solved() -> Result<(), Box<dyn Error>> {
        let board = Board::from_bytes_array(&[
            b"...",
            b"...",
            b"...",
        ]);

        let p1 = Polyomino::from_bytes_array(&[
            b"###",
            b"#.#",
        ])?;
        let p2 = Polyomino::from_bytes_array(&[
            b"###",
            b".#.",
        ])?;
        
        let mut prob = PolyominoPacking::default();
        *prob.board_mut() = board;
        prob.add_piece("1", p1);
        prob.add_piece("2", p2);
        let gen_prob = prob.generate_problem();
        
        let mut solver = Solver::new(gen_prob);
        let mut solutions = vec![];
        solver.run();

        for event in solver {
            if let SolverEvent::SolutionFound(sol) = event {
                solutions.push(sol);
            }
        }

        assert_eq!(solutions.len(), 4);

        Ok(())
    }
}
