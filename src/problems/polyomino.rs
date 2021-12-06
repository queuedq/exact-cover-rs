use indexmap::{IndexMap, IndexSet};
use crate::problem::{Problem, Value};
use crate::vector::Vector2D;

/// Orientation of a piece.
/// 
/// Reflection is applied first, then rotation.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Orientation {
    pub reflection: bool,
    pub rotation: i32, // 0..4
}


// Polyomino

/// The coordinates are normalized, i.e., the minimums of x or y coordinates are 0.
#[derive(PartialEq, Eq, Hash, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Polyomino {
    cells: Vec<Vector2D>,
    size: Vector2D,
    // TODO: add flags to configure the piece (e.g. rotation, reflection, etc.)
}

pub struct InvalidPiece;

impl Polyomino {
    pub fn new(cells: &[Vector2D]) -> Result<Polyomino, InvalidPiece> {
        if cells.len() == 0 { return Err(InvalidPiece) }

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

        Polyomino::new(&reflected).ok().unwrap()
    }

    /// Rotates the piece as specified amount.
    pub fn rotate(&self, rotation: i32) -> Polyomino {
        let rotated: Vec<_> = self.cells.iter()
            .map(|c| { c.rotate(rotation) })
            .collect();

        Polyomino::new(&rotated).ok().unwrap()
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

    pub fn translated_cells(&self, trans: Vector2D) -> Vec<Vector2D> {
        self.cells.iter()
            .map(|&c| { c + trans })
            .collect()
    }

    pub fn cells(&self) -> &Vec<Vector2D> { &self.cells }
    pub fn size(&self) -> Vector2D { self.size }
}


// Board

#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Board {
    cells: Vec<Vec<bool>>,
    size: Vector2D,
}

impl Board {
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

    pub fn cells(&self) -> &Vec<Vec<bool>> { &self.cells }
    pub fn size(&self) -> Vector2D { self.size }
}


// Problem

pub type CompoundName<N> = (N, Orientation, Vector2D);

#[derive(PartialEq, Eq, Clone, Hash)]
#[cfg_attr(test, derive(Debug))]
pub enum CompoundConstraint<N> {
    Piece(N),
    Cell(Vector2D),
}

#[cfg_attr(test, derive(Debug))]
pub struct PolyominoPacking<N: Value> {
    board: Board,
    pieces: IndexMap<N, Polyomino>,
}

impl<N: Value> PolyominoPacking<N> {
    pub fn new() -> PolyominoPacking<N> {
        PolyominoPacking {
            board: Default::default(),
            pieces: Default:: default(),
        }
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board;
    }
    
    pub fn add_piece(&mut self, name: N, piece: Polyomino) {
        self.pieces.insert(name, piece);
    }

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

    pub fn generate_subset(
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

    pub fn board(&self) -> &Board { &self.board }
    pub fn pieces(&self) -> &IndexMap<N, Polyomino> { &self.pieces }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;
    use crate::{Solver, SolverEvent};

    fn parse_piece(string: &[&str]) -> Polyomino {
        let mut cells = Vec::new();

        for y in 0..string.len() {
            for x in 0..string[y].len() {
                if string[y].chars().nth(x).unwrap() == '#' {
                    cells.push(Vector2D { x: x as i32, y: y as i32 });
                }
            }
        }

        Polyomino::new(&cells).ok().unwrap()
    }

    fn compare_unique_orientations(piece: &Polyomino, expected: &[(bool, i32)]) {
        assert_eq!(
            piece.unique_orientations().into_iter().collect::<HashSet<_>>(),
            expected.iter()
                .map(|&(f, r)| { Orientation { reflection: f, rotation: r } })
                .collect::<HashSet<_>>()
        )
    }

    #[test]
    fn unique_orientations_can_be_found_1() {
        let tetro_l = parse_piece(&[".#.", ".#.", ".##"]);
        compare_unique_orientations(&tetro_l, &[
            (false, 0), (false, 1), (false, 2), (false, 3),
            (true, 0), (true, 1), (true, 2), (true, 3),
        ]);

        let tetro_s = parse_piece(&["...", ".##", "##."]);
        compare_unique_orientations(&tetro_s, &[
            (false, 0), (false, 1), (true, 0), (true, 1),
        ]);

        let tetro_o = parse_piece(&["...", ".##", ".##"]);
        compare_unique_orientations(&tetro_o, &[
            (false, 0),
        ]);

        let pento_w = parse_piece(&["..#", ".##", "##."]);
        compare_unique_orientations(&pento_w, &[
            (false, 0), (false, 1), (false, 2), (false, 3),
        ]);
    }

    #[test]
    fn problem_can_be_solved() {
        let board_cells: Vec<Vec<bool>> = ([
            "...",
            "...",
            "...",
        ]).iter().map(|s| {
            s.chars().map(|c| { c == '.' }).collect()
        }).collect();
        let board = Board::new(board_cells);

        let p1 = parse_piece(&[
            "###",
            "#.#",
        ]);
        let p2 = parse_piece(&[
            "###",
            ".#.",
        ]);
        
        let mut prob = PolyominoPacking::new();
        prob.set_board(board);
        prob.add_piece("1", p1);
        prob.add_piece("2", p2);
        let gen_prob = prob.generate_problem();
        
        let mut solver = Solver::new(gen_prob);
        solver.run().ok();

        let sol: Vec<_> = solver.filter_map(|e| match e {
            SolverEvent::SolutionFound(s) => Some(s),
            _ => None,
        }).collect();

        assert_eq!(sol.len(), 4);
    }
}
