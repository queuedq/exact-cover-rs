#[derive(Debug, Default)]
struct Node {
    // row, col: 1-based b/c of head node (only internally)
    row: usize,
    col: usize,
    left: usize,
    right: usize,
    up: usize,
    down: usize,
}

#[derive(Debug)]
pub struct Matrix {
    row_cnt: usize,
    col_cnt: usize,
    pool: Vec<Node>, // head: 0, columns: 1..col_size
    col_size: Vec<usize>,
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix {
            row_cnt: 0,
            col_cnt: 0,
            pool: vec![Node::default()],
            col_size: vec![0],
        }
    }
}

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

    pub fn with_rows(col_cnt: usize, rows: Vec<Vec<usize>>) -> Matrix {
        let mut mat = Matrix::new(col_cnt);
        for row in &rows { mat.add_row(row) }
        mat
    }

    pub fn add_row(&mut self, row: &Vec<usize>) {
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

    pub fn search(&mut self) -> Vec<Vec<usize>> {
        let mut current_sol = vec![];
        let mut sols = vec![];
        self.recursive_search(&mut current_sol, &mut sols);
        sols
    }
}

// helper methods
impl Matrix {
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

    fn recursive_search(&mut self, current_sol: &mut Vec<usize>, sols: &mut Vec<Vec<usize>>) {
        if self.pool[Matrix::HEAD].right == Matrix::HEAD {
            sols.push(current_sol.clone());
        }

        let col = self.pool[Matrix::HEAD].right; // TODO: select better column
        self.cover_col(col);

        let mut r = self.pool[col].down;
        while r != col {
            current_sol.push(self.pool[r].row);
            let mut j = self.pool[r].right;
            while j != r {
                self.cover_col(self.pool[j].col);
                j = self.pool[j].right;
            }

            self.recursive_search(current_sol, sols);

            current_sol.pop();
            j = self.pool[r].left;
            while j != r {
                self.uncover_col(self.pool[j].col);
                j = self.pool[j].left;
            }

            r = self.pool[r].down;
        }

        self.uncover_col(col);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_search_should_solve_exact_cover() {
        let mut mat = Matrix::with_rows(
            7,
            vec![
                vec![3, 5, 6],
                vec![1, 4, 7],
                vec![2, 3, 6],
                vec![1, 4],
                vec![2, 7],
                vec![4, 5, 7],
            ],
        );
        let solutions = mat.search();
        assert_eq!(solutions.len(), 1);
    }

    #[test]
    fn matrix_search_should_find_multiple_solutions() {
        let mut mat = Matrix::with_rows(
            4,
            vec![vec![1], vec![2], vec![3], vec![4], vec![1, 3], vec![2, 4]],
        );
        let solutions = mat.search();
        assert_eq!(solutions.len(), 4);
    }
}
