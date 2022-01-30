use std::time::Instant;
use exact_cover::vector::Vector2D;
use exact_cover::problems::polyomino::{Polyomino, PolyominoPacking, Board, CompoundName};
use exact_cover::{Solver, SolverEvent};

fn print_sol(prob: &PolyominoPacking<&str>, sol: &Vec<CompoundName<&str>>) {
    let mut buff = Vec::new();
    
    for y in 0..prob.board().size().y {
        let row: Vec<char> = prob.board().cells()[y as usize].iter()
            .map(|c| { if *c { '.' } else { ' ' } })
            .collect();
        buff.push(row);
    }

    for &(name, o, t) in sol {
        let cells = prob.pieces()[name].orient(o).translated_cells(t);
        for Vector2D { x, y } in cells {
            buff[y as usize][x as usize] = name.chars().nth(0).unwrap();
        }
    }

    for y in 0..buff.len() {
        for x in 0..buff[y].len() {
            print!("{} ", buff[y][x]);
        }
        println!();
    }
}

fn main() {
    let board = Board::from_bytes_array(&[
        b"##...",
        b"#....",
        b"#....",
        b"#....",
        b".....",
    ]);

    let tet_i = Polyomino::from_bytes_array(&[
        b"####",
    ]).unwrap();
    let tet_o = Polyomino::from_bytes_array(&[
        b"##",
        b"##",
    ]).unwrap();
    let tet_t = Polyomino::from_bytes_array(&[
        b"###",
        b".#.",
    ]).unwrap();
    let tet_l = Polyomino::from_bytes_array(&[
        b"#..",
        b"###",
    ]).unwrap();
    let tet_s = Polyomino::from_bytes_array(&[
        b".##",
        b"##.",
    ]).unwrap();
    
    let mut prob = PolyominoPacking::default();
    *prob.board_mut() = board;
    prob.add_piece("I", tet_i);
    prob.add_piece("O", tet_o);
    prob.add_piece("T", tet_t);
    prob.add_piece("L", tet_l);
    prob.add_piece("S", tet_s);
    
    println!("Generating the problem...");
    let gen_prob = prob.generate_problem();
    let mut solver = Solver::new(gen_prob);
    
    println!("Solving the problem...");
    let start_time = Instant::now();
    solver.run().ok();

    let sol: Vec<_> = solver.filter_map(|e| match e {
        SolverEvent::SolutionFound(s) => Some(s),
        _ => None,
    }).collect();
    let elapsed_time = start_time.elapsed();

    println!("Done!");
    for solution in &sol {
        println!();
        print_sol(&prob, &solution);
    }

    println!(
        "Found {:?} solutions, w/ rotations/reflections. ({:?}s)",
        sol.len(),
        elapsed_time.as_millis() as f64 / 1000.
    );
}
