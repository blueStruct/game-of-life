use std::env;

use ggez::graphics::{DrawMode, Point2};
use ggez::*;
use rand::Rng;
use rayon::prelude::*;

use Cell::*;

#[derive(Clone, PartialEq)]
enum Cell {
    Alive,
    Dead,
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

struct Grid {
    write_buf: CellBuffer,
    read_buf: CellBuffer,
    w: usize,
    h: usize,
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

        for line in &mut grid.write_buf {
            for element in line {
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
        let (next, current) = (&mut self.write_buf, &self.read_buf);

        next.par_iter_mut().enumerate().for_each(|(y, line)| {
            line.par_iter_mut().enumerate().for_each(|(x, cell)| {
                // count the neighbors
                let x = x as i16;
                let y = y as i16;
                let mut neighbors = 0;

                for (ny, nx) in &NEIGHBOR_IDS {
                    let iy = {
                        if y + ny > h - 1 {
                            0
                        } else if y + ny < 0 {
                            h - 1
                        } else {
                            y + ny
                        }
                    };

                    let ix = {
                        if x + nx > w - 1 {
                            0
                        } else if x + nx < 0 {
                            w - 1
                        } else {
                            x + nx
                        }
                    };

                    if current[iy as usize][ix as usize] == Alive {
                        neighbors += 1;
                    };
                }

                // apply rules to cell
                match (&current[y as usize][x as usize], neighbors) {
                    (&Alive, i) if i < 2 || i > 3 => *cell = Dead,
                    (&Alive, 2..=3) => *cell = Alive,
                    (&Dead, 3) => *cell = Alive,
                    (&Dead, _) => *cell = Dead,
                    _ => {}
                }
            })
        });
    }
}

struct MainState {
    grid: Grid,
    cell_size: f32,
}

impl MainState {
    fn new(_ctx: &mut Context, w: usize, h: usize, cell_size: f32) -> GameResult<MainState> {
        let s = MainState {
            grid: Grid::new(w, h),
            cell_size,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 10) {
            self.grid.swap();
            self.grid.step();
        }
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let cell_size = self.cell_size;
        let next = &mut self.grid.write_buf;
        let mut mesh = graphics::MeshBuilder::new();

        graphics::clear(ctx);

        for (y, line) in next.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if *cell == Alive {
                    let y = y as f32;
                    let x = x as f32;
                    let mesh = mesh.polygon(
                        DrawMode::Fill,
                        &[
                            Point2::new(x * cell_size, y * cell_size),
                            Point2::new((x + 1.0) * cell_size, y * cell_size),
                            Point2::new((x + 1.0) * cell_size, (y + 1.0) * cell_size),
                            Point2::new(x * cell_size, (y + 1.0) * cell_size),
                        ],
                    );
                }
            }
        }

        let built_mesh = mesh.build(ctx)?;
        graphics::draw(ctx, &built_mesh, Point2::new(0.0, 0.0), 0.0).unwrap();
        graphics::present(ctx);

        Ok(())
    }
}

pub fn main() {
    let args: Vec<_> = env::args().collect();

    let mut w = 100;
    if args.len() >= 2 {
        w = args[1].parse::<u32>().unwrap();
    }

    let mut h = 100;
    if args.len() >= 3 {
        h = args[2].parse::<u32>().unwrap();
    }

    let mut cell_size = 8;
    if args.len() >= 4 {
        cell_size = args[3].parse::<u32>().unwrap();
    }

    let c = conf::Conf {
        window_mode: conf::WindowMode {
            width: w * cell_size,
            height: h * cell_size,
            ..conf::WindowMode::default()
        },
        window_setup: conf::WindowSetup::default(),
        backend: conf::Backend::default(),
    };

    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx, w as usize, h as usize, cell_size as f32).unwrap();
    event::run(ctx, state).unwrap();
}
