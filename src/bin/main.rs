#![allow(unused)]
use either::Either;
use ggez::graphics::{self, Color};
use ggez::*;

use spatial_hash::SpatialHash;

use std::time;

const S: f32 = 2395.;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CollisionMode {
    /// No Collisions
    None,
    /// Balls only hit other balls
    Balls,
    /// Balls only hit other pegs
    Pegs,
    /// Balls hit other balls and pegs
    BallsAndPegs,
}

pub type Vec2 = [f32; 2];
fn add([x, y]: Vec2, [a, b]: Vec2) -> Vec2 {
    [x + a, y + b]
}

fn sub([x, y]: Vec2, [a, b]: Vec2) -> Vec2 {
    [x - a, y - b]
}

fn kmul(k: f32, [x, y]: Vec2) -> Vec2 {
    [k * x, k * y]
}

fn modulo([x, y]: Vec2, [a, b]: Vec2) -> Vec2 {
    [x % a, y % b]
}

fn sqr(x: f32) -> f32 {
    x * x
}

fn dist_sqr([x, y]: Vec2, [a, b]: Vec2) -> f32 {
    sqr(x - a) + sqr(y - b)
}

fn len(v: Vec2) -> f32 {
    dist_sqr(v, [0.; 2])
}

fn normalize(a @ [x, y]: Vec2) -> Vec2 {
    let d = len(a).sqrt();
    [x / d, y / d]
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Circle {
    origin: [f32; 2],
    radius: f32,
    vel: [f32; 2],
}

impl Circle {
    fn new(x: f32, y: f32) -> Self {
        Circle {
            origin: [x, y],
            radius: 10.,
            vel: [0., 0.],
        }
    }
    fn x(&self) -> f32 {
        self.origin[0]
    }
    fn y(&self) -> f32 {
        self.origin[1]
    }
}

const RAD: f32 = 0.05;

#[derive(Debug)]
struct State {
    balls: Vec<Circle>,
    pegs: Vec<Circle>,

    peg_checks: usize,

    dt: time::Duration,
    substeps: usize,

    mode: CollisionMode,
    frame: usize,

    static_spatial_hash: Option<SpatialHash<usize>>,

    // text buffers
    fps_text: graphics::Text,
    peg_text: graphics::Text,
}

impl State {
    fn new() -> Self {
        State {
            pegs: vec![],
            balls: vec![],
            dt: time::Duration::new(0, 0),

            peg_checks: 0,

            substeps: 5,
            mode: CollisionMode::Pegs,
            //mode: CollisionMode::None,
            static_spatial_hash: None,

            frame: 0,

            fps_text: graphics::Text::new("..."),
            peg_text: graphics::Text::new("..."),
        }
    }

    fn set_pegs(&mut self, rows: usize, cols: usize, h: f32, w: f32) {
        self.pegs.clear();

        let rh = h / (rows as f32);
        let rw = w / (cols as f32);

        for i in 0..rows + 1 {
            let offset = if i % 2 == 0 { rw / 2. } else { 0. };
            let extra = if i % 2 == 0 { 0 } else { 1 };
            for j in 0..cols + extra {
                let x = rw * (j as f32) + offset;
                let y = rh * (i as f32) + rw / 2.;
                self.pegs.push(Circle::new(x, y + 12.));
            }
        }
    }
    fn make_peg_spatial_hash(&mut self) {
        //let mut sh = SpatialHash::tri_h(20.1);
        //let mut sh = SpatialHash::hex(20.1);
        let mut sh = SpatialHash::cube(20.1);
        for (i, p) in self.pegs.iter().enumerate() {
            sh.add(p.origin[0], p.origin[1], i);
        }
        self.static_spatial_hash = Some(sh);
    }

    fn satisfy_constraints(&mut self, dt: f32) {
        for b in &mut self.balls {
            if b.origin[0] < 0. {
                b.origin[0] = S - b.origin[0];
            } else if b.origin[0] > S {
                b.origin[0] %= S;
            }
        }
        if self.mode == CollisionMode::None {
            return;
        }
        if matches!(
            self.mode,
            CollisionMode::Balls | CollisionMode::BallsAndPegs
        ) {
            let nb = self.balls.len();
            for i in 0..nb {
                for j in i + 1..nb {
                    let b2 = self.balls[j];
                    let b = &mut self.balls[i];
                    let d2 = dist_sqr(b.origin, b2.origin);
                    if d2 < 400. {
                        let d = d2.sqrt();
                        let delta = normalize(sub(b.origin, b2.origin));
                        let delta = kmul(20. - d + 1e-8, delta);
                        b.origin = add(b.origin, delta);
                        b.vel = add(b.vel, kmul(1. / dt, delta));
                    }
                }
            }
        }

        if matches!(self.mode, CollisionMode::Pegs | CollisionMode::BallsAndPegs) {
            self.peg_checks = 0;
            // brute-force
            for b in &mut self.balls {
                let iter = if let Some(sh) = &self.static_spatial_hash {
                    Either::Left(sh.query(b.x(), b.y()).map(|&i| &self.pegs[i]))
                } else {
                    Either::Right(self.pegs.iter())
                };
                for p in iter {
                    let d2 = dist_sqr(b.origin, p.origin);
                    if d2 < 400. {
                        let d = d2.sqrt();
                        let delta = normalize(sub(b.origin, p.origin));
                        let delta = kmul(20. - d, delta);
                        b.origin = add(b.origin, delta);
                        b.vel = add(b.vel, kmul(1. / dt, delta));
                    }
                    self.peg_checks += 1;
                }
            }
        }
        self.peg_text = graphics::Text::new(graphics::TextFragment {
            text: format!("Checks: {}", self.peg_checks),
            scale: Some(graphics::PxScale { x: 24., y: 24. }),
            font: None,
            color: Some(graphics::Color::BLACK),
        });
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // fps is 1/dt where dt is in seconds.
        self.dt = ctx.time.delta();

        self.frame += 1;
        if self.frame % 50 == 0 {
            self.fps_text = graphics::Text::new(graphics::TextFragment {
                text: format!("FPS: {}", 1. / self.dt.as_secs_f32()),
                scale: Some(graphics::PxScale { x: 24., y: 24. }),
                font: None,
                color: Some(graphics::Color::BLACK),
            });
        }

        let dt = 1. / (self.substeps as f32);
        let t = self.dt.as_secs_f32().sin() * 3.;

        for _ in 0..self.substeps {
            for (i, v) in &mut self.balls.iter_mut().enumerate() {
                v.vel = add(
                    v.vel,
                    [(t + 0.1 + i as f32).cos() * 0.01, 0.2 + 0.05 * t.sin()],
                );
                v.origin = add(v.origin, kmul(dt, v.vel));

                v.vel[0] = v.vel[0].abs().min(15.).copysign(v.vel[0]);
                v.vel[1] = v.vel[1].min(15.);

                v.origin[1] = v.origin[1] % ctx.gfx.size().1;
            }

            self.satisfy_constraints(dt);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);
        let mut mb = graphics::MeshBuilder::new();

        for c in &self.balls {
            mb.circle(
                //ctx,
                graphics::DrawMode::fill(),
                mint::Point2 {
                    x: c.origin[0],
                    y: c.origin[1],
                },
                c.radius,
                0.1,
                graphics::Color::RED,
            )?;
        }

        for c in &self.pegs {
            mb.circle(
                graphics::DrawMode::fill(),
                mint::Point2 {
                    x: c.origin[0],
                    y: c.origin[1],
                },
                c.radius,
                0.1,
                graphics::Color::BLUE,
            )?;
        }

        canvas.draw(
            &graphics::Mesh::from_data(ctx, mb.build()),
            graphics::DrawParam::default(),
        );

        canvas.draw(
            &self.fps_text,
            graphics::DrawParam::default().dest([0., 0.]),
        );

        canvas.draw(
            &self.peg_text,
            graphics::DrawParam::default().dest([400., 0.]),
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() {
    let mut state = State::new();

    state.set_pegs(64, 60, S, S);
    state.make_peg_spatial_hash();
    let (mut ctx, event_loop) = ContextBuilder::new("Pachinko", "julianknodt")
        .build()
        .unwrap();

    ctx.gfx.set_drawable_size(S, S).unwrap();
    ctx.gfx.set_resizable(true).unwrap();
    ctx.gfx.set_window_title("Pachinko");

    let num_balls = 5000;
    for i in 0..num_balls {
        state.balls.push(Circle::new(
            (i * 13 % (S as usize)) as f32,
            20. + (i as f32 * 2.5) % 30.,
        ));
    }
    event::run(ctx, event_loop, state);
}
