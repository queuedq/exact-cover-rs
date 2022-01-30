use std::time::Instant;
use exact_cover::vector::Vector2D;
use exact_cover::problems::polyomino::{Polyomino, PolyominoPacking, Board, CompoundName};
use exact_cover::{Solver, SolverEvent};

fn parse_piece(string: &[&[u8]]) -> Polyomino {
    let mut cells = Vec::new();

    for y in 0..string.len() {
        for x in 0..string[y].len() {
            if string[y][x] == b'#' {
                cells.push(Vector2D { x: x as i32, y: y as i32 });
            }
        }
    }

    Polyomino::new(&cells).ok().unwrap()
}

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
    let board_cells: Vec<Vec<bool>> = ([
        b"........",
        b"........",
        b"........",
        b"...##...",
        b"...##...",
        b"........",
        b"........",
        b"........",
    ]).iter().map(|s| {
        s.iter().map(|c| { *c == b'.' }).collect()
    }).collect();
    let board = Board::new(board_cells);

    let pento_f = parse_piece(&[
        b".##",
        b"##.",
        b".#.",
    ]);
    let pento_i = parse_piece(&[
        b"#####",
    ]);
    let pento_l = parse_piece(&[
        b"####",
        b"#...",
    ]);
    let pento_n = parse_piece(&[
        b".###",
        b"##..",
    ]);
    let pento_p = parse_piece(&[
        b"###",
        b".##",
    ]);
    let pento_t = parse_piece(&[
        b"###",
        b".#.",
        b".#.",
    ]);
    let pento_u = parse_piece(&[
        b"#.#",
        b"###",
    ]);
    let pento_v = parse_piece(&[
        b"#..",
        b"#..",
        b"###",
    ]);
    let pento_w = parse_piece(&[
        b"#..",
        b"##.",
        b".##",
    ]);
    let pento_x = parse_piece(&[
        b".#.",
        b"###",
        b".#.",
    ]);
    let pento_y = parse_piece(&[
        b"####",
        b".#..",
    ]);
    let pento_z = parse_piece(&[
        b"##.",
        b".#.",
        b".##",
    ]);
    
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

    // TODO: exclude rotations/reflections
    println!(
        "Found {:?} solutions, w/ rotations/reflections. ({:?}s)",
        sol.len(),
        elapsed_time.as_millis() as f64 / 1000.
    );
}

