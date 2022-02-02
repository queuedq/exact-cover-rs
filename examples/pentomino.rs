use std::error::Error;
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

fn main() -> Result<(), Box<dyn Error>> {
    let board = Board::from_bytes_array(&[
        b"........",
        b"........",
        b"........",
        b"...##...",
        b"...##...",
        b"........",
        b"........",
        b"........",
    ]);

    let pento_f = Polyomino::from_bytes_array(&[
        b".##",
        b"##.",
        b".#.",
    ])?;
    let pento_i = Polyomino::from_bytes_array(&[
        b"#####",
    ])?;
    let pento_l = Polyomino::from_bytes_array(&[
        b"####",
        b"#...",
    ])?;
    let pento_n = Polyomino::from_bytes_array(&[
        b".###",
        b"##..",
    ])?;
    let pento_p = Polyomino::from_bytes_array(&[
        b"###",
        b".##",
    ])?;
    let pento_t = Polyomino::from_bytes_array(&[
        b"###",
        b".#.",
        b".#.",
    ])?;
    let pento_u = Polyomino::from_bytes_array(&[
        b"#.#",
        b"###",
    ])?;
    let pento_v = Polyomino::from_bytes_array(&[
        b"#..",
        b"#..",
        b"###",
    ])?;
    let pento_w = Polyomino::from_bytes_array(&[
        b"#..",
        b"##.",
        b".##",
    ])?;
    let pento_x = Polyomino::from_bytes_array(&[
        b".#.",
        b"###",
        b".#.",
    ])?;
    let pento_y = Polyomino::from_bytes_array(&[
        b"####",
        b".#..",
    ])?;
    let pento_z = Polyomino::from_bytes_array(&[
        b"##.",
        b".#.",
        b".##",
    ])?;
    
    let mut prob = PolyominoPacking::default();
    *prob.board_mut() = board;
    prob.add_piece("F", pento_f);
    prob.add_piece("I", pento_i);
    prob.add_piece("L", pento_l);
    prob.add_piece("N", pento_n);
    prob.add_piece("P", pento_p);
    prob.add_piece("T", pento_t);
    prob.add_piece("U", pento_u);
    prob.add_piece("V", pento_v);
    prob.add_piece("W", pento_w);
    prob.add_piece("X", pento_x);
    prob.add_piece("Y", pento_y);
    prob.add_piece("Z", pento_z);
    
    println!("Generating the problem...");
    let gen_prob = prob.generate_problem();
    let mut solver = Solver::new(gen_prob);
    
    println!("Solving the problem...");
    let start_time = Instant::now();
    let mut solutions = vec![];
    solver.run();
    
    for event in solver {
        if let SolverEvent::SolutionFound(sol) = event {
            print_sol(&prob, &sol);
            println!();
            solutions.push(sol);
        }
    }

    // This does not measure the exact time because printing the solutions takes up a nonnegligible fraction.
    // To measure the exact time, print the solutions after this line.
    let elapsed_time = start_time.elapsed();

    println!(
        "Found {:?} solutions, w/ rotations/reflections. ({:?}s)",
        solutions.len(),
        elapsed_time.as_millis() as f64 / 1000.
    );

    Ok(())
}

