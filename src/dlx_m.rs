//! A low-level API for dancing links (DLX) algorithm.
//! 
//! If you are looking for a [`Problem`](crate::problem::Problem) solver API,
//! see the [`solver`](crate::solver) module.

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
    
    // column multiplicity range
    min: Vec<usize>,
    max: Vec<usize>,
    weight: Vec<usize>,

    partial_sol: Vec<usize>,
    col_stack: Vec<usize>,
    row_stack: Vec<usize>,
    task_stack: Vec<usize>,
    abort_requested: bool,
}

/// An interface of callback objects to pass to the DLX algorithm.
pub trait Callback {
    fn on_solution(&mut self, _sol: Vec<usize>, _mat: &mut Matrix) {}
    fn on_iteration(&mut self, _mat: &mut Matrix) {}
    fn on_abort(&mut self, _mat: &mut Matrix) {}
    fn on_finish(&mut self) {}
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix {
            row_cnt: 0,
            col_cnt: 0,
            pool: vec![Node::default()],
            col_size: vec![0],

            min: vec![0],
            max: vec![0],
            weight: vec![0],

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
        // Set multiplicity to [1, 1] by default
        let mut col_mul_default = vec![1; col_cnt + 1];
        col_mul_default[0] = 0;

        let mut mat = Matrix {
            col_cnt,
            col_size: vec![0; col_cnt + 1],
            min: col_mul_default.clone(),
            max: col_mul_default.clone(),
            weight: vec![0; col_cnt + 1],
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

    pub fn set_multiplicity(&mut self, col: usize, min: usize, max: usize) {
        self.min[col] = min;
        self.max[col] = max;
    }
}

// Main algorithm (dancing links)
impl Matrix {
    pub fn solve(
        &mut self,
        callback: &mut impl Callback,
    ) {
        self.abort_requested = false;
        self._recursive_solve(callback);
    }

    /// A recursive DLX algorithm.
    /// 
    /// It functions as a reference implementation for [`iterative_solve`].
    /// It does not handle all callback functions, so be careful when you want to use it.
    fn _recursive_solve(
        &mut self,
        callback: &mut impl Callback,
    ) {
        // Dancing links with multiplicity (Algorithm M)
        // ================
        // [CHOOSE-COLUMN] In each recursion level, choose a single column c.
        // [TRY-ROWS] Try each row r in column c and then recurse.
        // [COVER-FULL] If column c becomes full after selecting any row, cover it -- to disable it.
        // [TWEAK-ROW] Otherwise, just hide the rows above row r -- to force the row order.
        // [NO-SELECT] If c is already fulfilled, also recurse without selecting any row at all.
        // [UNDO] Finally, undo all modifications and backtrack.
        //
        // At most one row is selected in each recursion level.

        // === Task 1 ===
        // Handle callbacks
        if self.pool[Matrix::HEAD].right == Matrix::HEAD {
            callback.on_solution(self.partial_sol.clone(), self);
        }
        callback.on_iteration(self);

        // [CHOOSE-COLUMN] MRV (minimum remaining values) heuristic:
        // choose a column with minimal branching factor.
        //
        // After selecting a row in the previous recursion level,
        // some columns can become unfulfillable. (See `select_row` to check the details.)
        // So `choose_best_col` prioritizes unfulfillable columns for early return.
        //
        // Also, it deprioritizes already-fulfilled columns as well,
        // because it is more effective to increase the number of fulfilled columns directly.
        // 
        // We don't have any fulfilled columns remaining in the matrix,
        // because every column is covered as soon as it is fulfilled.
        let c = self.choose_best_col(); // TODO-A: modify find best column logic
        if !self.col_fulfillable(c) { return; }
        
        // [COVER-FULL] If column c becomes full after selecting any row, cover it in advance.
        self.weight[c] += 1; // will select a row
        let mut covered = false;
        if self.col_full(c) {
            self.cover_col(c);
            covered = true;
        }

        // [TRY-ROWS]
        let first = self.pool[c].down; // to untweak rows later (UNDO)
        let mut r = first;
        while r != c {
            // === Task 2 ===
            // TODO: (pruning) Break early if there exist unfulfillable columns after tweaking (and before selecting a row).
            // be careful to skip NO-SELECT step if exited early.
            
            if !covered { // If covered, rows are already hidden.
                // [TWEAK-ROW]
                self.tweak_row(r);
            }
            self.select_row(r);
            self.partial_sol.push(self.pool[r].row);
            
            // If column c becomes unfulfillable after selecting a row, don't recurse.
            // TODO: (optimization) Compare performance with/without the condition below.
            if self.col_fulfillable(r) {
                self._recursive_solve(callback);
            }

            // === Task 3 === (including out of while loop)
            // TODO: Modify task 3 range
            self.unselect_row(r);
            self.partial_sol.pop();
            r = self.pool[r].down;
        }

        // TODO: Assign task numbers to the lines below for iterative implementation

        // [NO-SELECT] If column c was already fulfilled, not selecting any row is also an option.
        self.weight[c] -= 1;
        if self.col_fulfilled(c) {
            // All rows are already hidden, so just hide the column from the column list.
            let Node { left, right, .. } = self.pool[c];
            self.pool[left].right = right;
            self.pool[right].left = left;

            self._recursive_solve(callback);

            self.pool[left].right = c;
            self.pool[right].left = c;
        }

        // [UNDO] Undo all modifications
        if covered {
            self.uncover_col(c);
        } else {
            self.untweak_rows(first);
        }
    }

    /// An iterative DLX algorithm.
    /// TODO: Fix it to match the new recursive algorithm.
    fn _iterative_solve(&mut self, callback: &mut impl Callback) {
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
                    
                    self.select_row(r);
                    self.partial_sol.push(self.pool[r].row);
                    
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

    // ======== Level 4 ========

    /// Selects (already hidden) row r by selecting each node j in the row.
    /// 
    /// It doesn't add the weight to the current column,
    /// because the current column's weight is handled in the main algorithm.
    /// 
    /// Be aware that the current column's fulfillability may change after calling this function.
    /// Selecting some node j in row r can make j's column covered,
    /// and in turn hide other rows which are also in the current column c.
    #[inline]
    fn select_row(&mut self, r: usize) {
        let mut j = self.pool[r].right;
        while j != r {
            self.select_node(j);
            j = self.pool[j].right;
        }
    }

    /// [Level 3] Unselects row r.
    #[inline]
    fn unselect_row(&mut self, r: usize) {
        let mut j = self.pool[r].left;
        while j != r {
            self.unselect_node(self.pool[j].col);
            j = self.pool[j].left;
        }
    }

    /// Selects (already hidden) node j.
    /// Subroutine of `select_row`.
    #[inline]
    fn select_node(&mut self, j: usize) {
        let c = self.pool[j].col;
        self.weight[c] += 1;
        // If column c is full after selecting node j, cover the column
        if self.col_full(c) {
            self.cover_col(c);
        }
    }

    /// Unselects node j.
    #[inline]
    fn unselect_node(&mut self, j: usize) {
        let c = self.pool[j].col;
        if self.col_full(c) {
            self.uncover_col(c);
        }
        self.weight[c] -= 1;
    }
    
    // ======== Level 3 ========

    /// Covers column c by hiding all its rows.
    /// It effectively disables the use of column c at all.
    #[inline]
    fn cover_col(&mut self, c: usize) {
        // remove c from column list
        let Node { left, right, .. } = self.pool[c];
        self.pool[left].right = right;
        self.pool[right].left = left;

        // hide rows
        let mut r = self.pool[c].down;
        while r != c {
            self.hide_row(r);
            r = self.pool[r].down;
        }
    }

    /// Uncovers column c.
    #[inline]
    fn uncover_col(&mut self, c: usize) {
        let mut r = self.pool[c].up;
        while r != c {
            self.unhide_row(r);
            r = self.pool[r].up;
        }

        let Node { left, right, .. } = self.pool[c];
        self.pool[left].right = c;
        self.pool[right].left = c;
    }

    /// Hides row r completely (i.e. from the current column as well).
    /// It should be called only when r is the first node in the column.
    /// The name "tweak" is from Knuth's TAOCP fascicle 5.
    #[inline]
    fn tweak_row(&mut self, r: usize) {
        self.hide_row(r);
        let Node { col: c, down: d, .. } = self.pool[r];
        self.pool[c].down = d;
        self.pool[d].down = c;
    }

    /// Untweaks all rows starting from r.
    /// It takes advantage from the non-obvious fact
    /// that unhiding rows can be done in the same order as hiding.
    #[inline]
    fn untweak_rows(&mut self, mut r: usize) {
        let c = self.pool[r].col;
        while r != c {
            self.unhide_row(r);
            let Node { up: u, down: d, .. } = self.pool[r];
            self.pool[u].down = r;
            self.pool[d].down = r;
            r = d;
        }
    }

    // ======== Level 2 ========

    /// Hides row r from other columns by hiding each node j in the row.
    /// It doesn't hide node r from its column,
    /// so call it when the column is covered or you have to manually hide node r.
    #[inline]
    fn hide_row(&mut self, r: usize) {
        let mut j = self.pool[r].right;
        while j != r {
            self.hide_node(j);
            j = self.pool[j].right;
        }
    }

    /// Unhides row r.
    #[inline]
    fn unhide_row(&mut self, r: usize) {
        let mut j = self.pool[r].left;
        while j != r {
            self.unhide_node(j);
            j = self.pool[j].left;
        }
    }

    // ======== Level 1 ========
    
    /// Hides node j by connecting its up/down nodes.
    #[inline]
    fn hide_node(&mut self, j: usize) {
        let Node { col, up, down, .. } = self.pool[j];
        self.pool[up].down = down;
        self.pool[down].up = up;
        self.col_size[col] -= 1;
    }
    
    /// Unhides node j.
    #[inline]
    fn unhide_node(&mut self, j: usize) {
        let Node { col, up, down, .. } = self.pool[j];
        self.pool[up].down = j;
        self.pool[down].up = j;
        self.col_size[col] += 1;
    }
    
    // ======== Level 0 ========

    /// Chooses the column with the lowest `col_size`. (MRV Heuristic).
    #[inline]
    fn choose_best_col(&self) -> usize {
        let mut best_col = self.pool[Matrix::HEAD].right;
        let mut best_size = self.col_size[best_col];
        
        let mut c = best_col;
        while c != Matrix::HEAD {
            if self.col_size[c] < best_size {
                best_col = c;
                best_size = self.col_size[c];
            }
            c = self.pool[c].right;
        }
        best_col
    }

    /// Returns whether column c is selected within the multiplicity range.
    #[inline]
    fn col_fulfilled(&self, c: usize) -> bool {
        let Matrix { weight, min, max, .. } = self;
        return min[c] <= weight[c] && weight[c] <= max[c];
    }

    /// Returns whether column c is fully selected.
    #[inline]
    fn col_full(&self, c: usize) -> bool {
        return self.weight[c] == self.max[c];
    }
    
    /// Returns whether it is possible to select column c within the multiplicity range.
    #[inline]
    fn col_fulfillable(&self, c: usize) -> bool {
        let Matrix { weight, min, max, col_size, .. } = self;
        if weight[c] > max[c] { return false; }
        if weight[c] + col_size[c] < min[c] { return false; }
        return true;
    }
}


#[cfg(test)]
mod tests {
    // use super::*;
}
