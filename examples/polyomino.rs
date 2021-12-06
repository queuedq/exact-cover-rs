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
        b"##...",
        b"#....",
        b"#....",
        b"#....",
        b".....",
    ]).iter().map(|s| {
        s.iter().map(|c| { *c == b'.' }).collect()
    }).collect();
    let board = Board::new(board_cells);

    let tet_i = parse_piece(&[
        b"####",
    ]);
    let tet_o = parse_piece(&[
        b"##",
        b"##",
    ]);
    let tet_t = parse_piece(&[
        b"###",
        b".#.",
    ]);
    let tet_l = parse_piece(&[
        b"#..",
        b"###",
    ]);
    let tet_s = parse_piece(&[
        b".##",
        b"##.",
    ]);
    
    let mut prob = PolyominoPacking::new();
    prob.set_board(board);
    prob.add_piece("I", tet_i);
    prob.add_piece("O", tet_o);
    prob.add_piece("T", tet_t);
    prob.add_piece("L", tet_l);
    prob.add_piece("S", tet_s);
    let gen_prob = prob.generate_problem();

    let mut solver = Solver::new(gen_prob);
    solver.run().ok();

    let sol: Vec<_> = solver.filter_map(|e| match e {
        SolverEvent::SolutionFound(s) => Some(s),
        _ => None,
    }).collect();

    println!("{:?}", sol.len());
    for solution in sol {
        println!();
        print_sol(&prob, &solution);
    }
}
