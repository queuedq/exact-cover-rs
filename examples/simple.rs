extern crate exact_cover;
use exact_cover::{Matrix};

fn main() {
    // First problem
    let mut mat = Matrix::with_rows(
        7,
        &[
            &[3, 5, 6],
            &[1, 4, 7],
            &[2, 3, 6],
            &[1, 4],
            &[2, 7],
            &[4, 5, 7],
        ],
    );
    let solutions = mat.search();
    println!("First problem: {:?}", solutions);

    // Second problem
    mat = Matrix::with_rows(
        4,
        &[&[1], &[2], &[3], &[4], &[1, 3], &[2, 4]],
    );
    let solutions = mat.search();
    println!("Second problem: {:?}", solutions);
}
