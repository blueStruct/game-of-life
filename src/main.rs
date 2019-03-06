use ggez::*;
use ggez::graphics::{DrawMode, Point2};
use Cell::*;
use rand::Rng;
use std::env;


#[derive(Clone, PartialEq)]
enum Cell {
    Alive,
    Dead,
}

type CellBuffer = Vec<Vec<Cell>>;

struct Grid {
    swapped: bool,
    buffer0: CellBuffer,
    buffer1: CellBuffer,
    w: usize,
    h: usize,
}

impl Grid {
    fn new(w: usize, h: usize) -> Self {
        let mut grid = Grid {
            buffer0: vec![vec![Dead; w]; h],
            buffer1: vec![vec![Dead; w]; h],
            swapped: false,
            w, h,
        };
        
        let mut rng = rand::thread_rng();

        for line in &mut grid.buffer0 {
            for element in line {
                if rng.gen::<f32>() < 0.33 {
                    *element = Alive;
                }
            }
        }

        grid
    }

    fn swap(&mut self) {
        self.swapped = !self.swapped;
    }

    fn get_buffers(&mut self) -> (&mut CellBuffer, &CellBuffer) {
        if !self.swapped {
            (&mut self.buffer0, &self.buffer1)
        } else {
            (&mut self.buffer1, &self.buffer0)
        }
    }

    fn step(&mut self) {
        let (h, w) = (self.h as i16, self.w as i16);
        let (next, current) = self.get_buffers();
        let mut neighbor_cells: Vec<(i16, i16)> = Vec::new();

        for &x in &[-1, 1, 0] {
            for &y in &[-1, 1, 0] {
                neighbor_cells.push((x, y));
            }
        }
        neighbor_cells.pop();

        for (y, line) in next.iter_mut().enumerate() {
            for (x, cell) in line.iter_mut().enumerate() {
                // count the neighbors
                let x = x as i16;
                let y = y as i16;
                let mut neighbors = 0;
                for (nx, ny) in &neighbor_cells {
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
                    (&Alive, 2 ..= 3) => *cell = Alive, 
                    (&Dead, 3) => *cell = Alive,
                    (&Dead, _) => *cell = Dead,
                    _ => {},
                }
            }
        }
    }
}

struct MainState {
    grid: Grid,
    cell_size: f32,
}

impl MainState {
    fn new(_ctx: &mut Context, w: usize, h: usize, cell_size: f32) -> GameResult<MainState> {
        let s = MainState { grid: Grid::new(w, h), cell_size };
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

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let cell_size = self.cell_size;
        let (_, next) = self.grid.get_buffers();
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
                            Point2::new(x*cell_size, y*cell_size),
                            Point2::new((x+1.0)*cell_size, y*cell_size),
                            Point2::new((x+1.0)*cell_size, (y+1.0)*cell_size),
                            Point2::new(x*cell_size, (y+1.0)*cell_size),
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
            .. conf::WindowMode::default()
        },
        window_setup: conf::WindowSetup::default(),
        backend: conf::Backend::default(),
    };

    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx, w as usize, h as usize, cell_size as f32).unwrap();
    event::run(ctx, state).unwrap();
}
