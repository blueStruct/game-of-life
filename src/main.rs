use ggez::graphics::{DrawMode, DrawParam, BLACK, WHITE};
use ggez::nalgebra as na;
use ggez::*;
use rand::Rng;
use rayon::prelude::*;

use Cell::*;

pub fn main() {
    let args: Vec<f32> = std::env::args()
        .skip(1)
        .map(|x| x.parse::<f32>().expect("not a number"))
        .collect();

    let w = args[0];
    let h = args[1];
    let cell_size = args[2];

    let (mut ctx, mut event_loop) = ContextBuilder::new("game of life", "markus")
        .window_mode(conf::WindowMode {
            width: w * cell_size,
            height: h * cell_size,
            ..conf::WindowMode::default()
        })
        .build()
        .unwrap();

    let mut game = Game::new(&mut ctx, w as usize, h as usize, cell_size as f32).unwrap();
    event::run(&mut ctx, &mut event_loop, &mut game).unwrap();
}

const NEIGHBOR_IDS: [(i16, i16); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

type CellBuffer = Vec<Vec<Cell>>;

struct Game {
    grid: Grid,
    cell_size: f32,
}

struct Grid {
    write_buf: CellBuffer,
    read_buf: CellBuffer,
    w: usize,
    h: usize,
}

#[derive(Clone, PartialEq)]
enum Cell {
    Dead,
    Alive,
}

impl Game {
    fn new(_ctx: &mut Context, w: usize, h: usize, cell_size: f32) -> GameResult<Game> {
        let s = Game {
            grid: Grid::new(w, h),
            cell_size,
        };
        Ok(s)
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 10) {
            self.grid.swap();
            self.grid.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, BLACK);

        let cell_size = self.cell_size;
        let next = &mut self.grid.write_buf;
        let mut mesh = graphics::MeshBuilder::new();

        for (y, row) in next.iter().enumerate() {
            let y1 = y as f32 * cell_size;
            let y2 = y1 + cell_size;

            for (x, cell) in row.iter().enumerate() {
                if *cell == Alive {
                    let x1 = x as f32 * cell_size;
                    let x2 = x1 + cell_size;

                    mesh.polygon(
                        DrawMode::fill(),
                        &[
                            na::Point2::new(x1, y1),
                            na::Point2::new(x2, y1),
                            na::Point2::new(x2, y2),
                            na::Point2::new(x1, y2),
                        ],
                        WHITE,
                    )?;
                }
            }
        }

        let built_mesh = mesh.build(ctx)?;
        graphics::draw(ctx, &built_mesh, DrawParam::new()).unwrap();
        graphics::present(ctx)?;
        timer::yield_now();

        Ok(())
    }
}

impl Grid {
    fn new(w: usize, h: usize) -> Self {
        let mut grid = Grid {
            write_buf: vec![vec![Dead; w]; h],
            read_buf: vec![vec![Dead; w]; h],
            w,
            h,
        };

        let mut rng = rand::thread_rng();

        for row in &mut grid.write_buf {
            for element in row {
                if rng.gen::<f32>() < 0.33 {
                    *element = Alive;
                }
            }
        }

        grid
    }

    fn swap(&mut self) {
        std::mem::swap(&mut self.write_buf, &mut self.read_buf);
    }

    fn step(&mut self) {
        let (h, w) = (self.h as i16, self.w as i16);
        let (write_buf, read_buf) = (&mut self.write_buf, &self.read_buf);

        write_buf.par_iter_mut().enumerate().for_each(|(y, row)| {
            let y1 = y as i16;

            row.iter_mut().enumerate().for_each(|(x, cell)| {
                // count the neighbors
                let x1 = x as i16;
                let mut neighbors = 0;

                for (dy, dx) in &NEIGHBOR_IDS {
                    let y2 = (y1 + dy + h) % h;
                    let x2 = (x1 + dx + w) % w;

                    if read_buf[y2 as usize][x2 as usize] == Alive {
                        neighbors += 1;
                    };
                }

                // apply rules to cell
                match (&read_buf[y][x], neighbors) {
                    (&Dead, 3) => *cell = Alive,
                    (&Dead, _) => *cell = Dead,
                    (&Alive, i) if i < 2 || i > 3 => *cell = Dead,
                    (&Alive, 2..=3) => *cell = Alive,
                    _ => {}
                }
            })
        });
    }
}
