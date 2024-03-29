//! A low-level API for original dancing links (DLX) algorithm.

use crate::dlx::callback::{Callback};

/// A single node of [`Matrix`].
#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
struct Node {
    // row, col: 1-based b/c of head node (only internally)
    row: usize,
    col: usize,
    left: usize,
    right: usize,
    up: usize,
    down: usize,
}

/// A sparse matrix representation of an exact cover problem used for DLX algorithm.
#[cfg_attr(test, derive(Debug))]
pub struct Matrix {
    row_cnt: usize,
    col_cnt: usize,
    pool: Vec<Node>, // head: 0, columns: 1..=col_cnt
    col_size: Vec<usize>,

    partial_sol: Vec<usize>,
    col_stack: Vec<usize>,
    row_stack: Vec<usize>,
    task_stack: Vec<usize>,
    abort_requested: bool,
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix {
            row_cnt: 0,
            col_cnt: 0,
            pool: vec![Node::default()],
            col_size: vec![0],
            
            partial_sol: vec![],
            col_stack: vec![],
            row_stack: vec![],
            task_stack: vec![],
            abort_requested: false,
        }
    }
}

// Methods for initializing Matrix
impl Matrix {
    const HEAD: usize = 0;

    pub fn new(col_cnt: usize) -> Matrix {
        let mut mat = Matrix {
            col_cnt,
            col_size: vec![0; col_cnt + 1],
            ..Matrix::default()
        };
        for col_num in 1..=col_cnt {
            let col = mat.create_node(0, col_num);
            mat.insert_right(col - 1, col);
        }
        mat
    }

    pub fn with_rows(col_cnt: usize, rows: &[&[usize]]) -> Matrix {
        let mut mat = Matrix::new(col_cnt);
        for row in rows { mat.add_row(row) }
        mat
    }

    pub fn add_row(&mut self, row: &[usize]) {
        self.row_cnt += 1;
        let row_num = self.row_cnt;
        let mut left_node = 0;

        for &col_num in row {
            assert!(1 <= col_num && col_num <= self.col_cnt); // TODO: write proper validation logic
            let node = self.create_node(row_num, col_num);

            self.insert_down(self.pool[col_num].up, node);
            if left_node != 0 { self.insert_right(left_node, node); }

            self.col_size[col_num] += 1;
            left_node = node;
        }
    }
}

// Main algorithm (dancing links)
impl Matrix {
    pub fn solve(
        &mut self,
        callback: &mut impl Callback<Matrix>,
    ) {
        self.abort_requested = false;
        self.iterative_solve(callback);
    }

    /// A recursive DLX algorithm.
    /// 
    /// It functions as a reference implementation for [`iterative_solve`].
    /// It does not handle all callback functions, so be careful when you want to use it.
    fn _recursive_solve(
        &mut self,
        callback: &mut impl Callback<Matrix>,
    ) {
        // === Task 1 ===
        // Handle callbacks
        if self.pool[Matrix::HEAD].right == Matrix::HEAD {
            callback.on_solution(self.partial_sol.clone(), self);
        }

        callback.on_iteration(self);

        // DLX algorithm
        // =============

        // MRV (minimum remaining values) heuristic
        // Choose a column with minimal branching factor
        let (col, size) = self.choose_best_col();
        if size == 0 { return; } // Dead end
        
        // Select a row to cover the selected column
        self.cover_col(col);

        let mut r = self.pool[col].down;
        while r != col {
            // === Task 2 ===
            let row = self.select_row(r);
            self.partial_sol.push(row);

            // Recurse
            self._recursive_solve(callback);

            // === Task 3 === (including out of while loop)
            self.unselect_row(r);
            self.partial_sol.pop();

            r = self.pool[r].down;
        }

        self.uncover_col(col);
    }

    /// An iterative DLX algorithm.
    fn iterative_solve(&mut self, callback: &mut impl Callback<Matrix>) {
        self.task_stack.push(1);

        while !self.task_stack.is_empty() {
            match self.task_stack.pop().unwrap() {
                1 => {
                    // Handle callbacks
                    if self.pool[Matrix::HEAD].right == Matrix::HEAD {
                        callback.on_solution(self.partial_sol.clone(), self);
                    }

                    callback.on_iteration(self);

                    if self.abort_requested {
                        callback.on_abort(self);
                        return
                    }

                    // MRV (minimum remaining values) heuristic
                    // Choose a column with minimal branching factor
                    let (col, size) = self.choose_best_col();
                    if size == 0 { continue; } // Dead end

                    // Select a row to cover the selected column
                    self.cover_col(col);

                    let r = self.pool[col].down;
                    // End of chunk
                    self.col_stack.push(col);
                    self.row_stack.push(r);
                    self.task_stack.push(2);
                }
                2 => {
                    // Restore variables
                    let r = *self.row_stack.last().unwrap();
                    
                    let row = self.select_row(r);
                    self.partial_sol.push(row);
                    
                    // End of chunk
                    self.task_stack.push(3);
                    self.task_stack.push(1);
                }
                3 => {
                    // Restore variables
                    let col = *self.col_stack.last().unwrap();
                    let mut r = self.row_stack.pop().unwrap();
                    
                    self.unselect_row(r);
                    self.partial_sol.pop();
                    
                    r = self.pool[r].down;
                    // End of chunk
                    if r == col {
                        // Out of while loop
                        self.uncover_col(col);
                        self.col_stack.pop();
                    } else {
                        self.row_stack.push(r);
                        self.task_stack.push(2);
                    }
                }
                _ => { panic!("Unexpected implementation error"); }
            }
        }

        callback.on_finish()
    }
}

// Helper methods
impl Matrix {
    pub fn abort(&mut self) {
        self.abort_requested = true;
    }

    fn create_node(&mut self, row: usize, col: usize) -> usize {
        let idx = self.pool.len();
        self.pool.push(Node {
            row,
            col,
            left: idx,
            right: idx,
            up: idx,
            down: idx,
        });
        idx
    }

    fn insert_right(&mut self, at: usize, node: usize) {
        let right = self.pool[at].right;
        self.pool[node].right = right;
        self.pool[right].left = node;
        self.pool[node].left = at;
        self.pool[at].right = node;
    }

    fn insert_down(&mut self, at: usize, node: usize) {
        let down = self.pool[at].down;
        self.pool[node].down = down;
        self.pool[down].up = node;
        self.pool[node].up = at;
        self.pool[at].down = node;
    }

    #[inline]
    fn cover_col(&mut self, col: usize) {
        let Node { left, right, .. } = self.pool[col];
        self.pool[left].right = right;
        self.pool[right].left = left;

        let mut i = self.pool[col].down;
        while i != col {
            let mut j = self.pool[i].right;
            while j != i {
                let Node { col: c, up, down, .. } = self.pool[j];
                self.pool[up].down = down;
                self.pool[down].up = up;

                self.col_size[c] -= 1;
                j = self.pool[j].right;
            }

            i = self.pool[i].down;
        }
    }

    #[inline]
    fn uncover_col(&mut self, col: usize) {
        let mut i = self.pool[col].up;
        while i != col {
            let mut j = self.pool[i].left;
            while j != i {
                let Node { col: c, up, down, .. } = self.pool[j];
                self.pool[up].down = j;
                self.pool[down].up = j;

                self.col_size[c] += 1;
                j = self.pool[j].left;
            }

            i = self.pool[i].up;
        }

        let Node { left, right, .. } = self.pool[col];
        self.pool[left].right = col;
        self.pool[right].left = col;
    }

    #[inline]
    fn select_row(&mut self, r: usize) -> usize {
        let mut j = self.pool[r].right;
        while j != r {
            self.cover_col(self.pool[j].col);
            j = self.pool[j].right;
        }
        // Returns index of selected row (containing node r)
        self.pool[r].row
    }

    #[inline]
    fn unselect_row(&mut self, r: usize) {
        let mut j = self.pool[r].left;
        while j != r {
            self.uncover_col(self.pool[j].col);
            j = self.pool[j].left;
        }
    }

    #[inline]
    fn choose_best_col(&self) -> (usize, usize) {
        let mut col = self.pool[Matrix::HEAD].right;
        let mut size = self.col_size[col];
        
        let mut j = col;
        while j != Matrix::HEAD {
            if self.col_size[j] < size {
                col = j;
                size = self.col_size[j];
            }
            j = self.pool[j].right;
        }
        (col, size)
    }
}


#[cfg(test)]
mod tests {
    // use super::*;
}
