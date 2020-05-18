extern crate exact_cover;
use exact_cover::{Matrix};

fn main() {
    // First problem
    let mut mat = Matrix::new(
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
    println!("First problem: {:?}", solutions);

    // Second problem
    mat = Matrix::new(
        4,
        vec![vec![1], vec![2], vec![3], vec![4], vec![1, 3], vec![2, 4]],
    );
    let solutions = mat.search();
    println!("Second problem: {:?}", solutions);
}
