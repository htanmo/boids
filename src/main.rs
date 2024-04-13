mod boids;

use boids::Boid;

use raylib::prelude::*;

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;
const FPS: u32 = 60;
const NBOIDS: usize = 100;

fn main() {
    #[cfg(not(debug_assertions))]
    raylib::set_trace_log(TraceLogLevel::LOG_NONE);

    let (mut rl, thread) = raylib::init().size(WIDTH, HEIGHT).title("Boids").build();

    rl.set_target_fps(FPS);

    let mut flock: Vec<Boid> = Vec::new();

    for _ in 0..NBOIDS {
        let x: f64 = get_random_value(0, WIDTH);
        let y: f64 = get_random_value(0, HEIGHT);
        let r: f64 = get_random_value(0, 6);
        let boid = Boid::new(
            Vector2::new(x as f32, y as f32),
            Vector2::new(20.0, 20.0),
            r as f32,
            1.0,
        );
        flock.push(boid);
    }

    while !rl.window_should_close() {
        let mut boids = flock.to_vec();
        for boid in flock.iter_mut() {
            boid.update(&mut boids, NBOIDS);
        }

        let mut ctx = rl.begin_drawing(&thread);

        for boid in flock.iter() {
            boid.draw(&mut ctx);
        }

        ctx.clear_background(Color::WHITE);
    }
}
