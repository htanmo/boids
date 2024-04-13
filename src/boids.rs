use crate::{HEIGHT, WIDTH};

use raylib::{
    ffi::{atan2f, fmod, GetTime},
    prelude::*,
};

const LOCAL_FLOCK_SIZE: usize = 128;

#[derive(Debug)]
struct LocalFlock {
    flock: Vec<Boid>,
    size: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Boid {
    origin: Vector2,
    positions: [Vector2; 3],
    velocity: Vector2,
    rotation: f32,
    angular_vel: f32,
    last_update: f64,
}

impl Boid {
    pub fn new(origin: Vector2, velocity: Vector2, rotation: f32, angular_vel: f32) -> Self {
        let mut positions: [Vector2; 3] = [Vector2::zero(); 3];
        positions[0] = Vector2::new(0.0, -5.0);
        positions[1] = Vector2::new(-5.0, 5.0);
        positions[2] = Vector2::new(5.0, 5.0);

        let last_update = unsafe { GetTime() };

        let mut boid = Boid {
            origin,
            positions,
            velocity,
            angular_vel,
            rotation: 0.0,
            last_update,
        };

        boid.rotate(rotation);

        boid
    }

    fn get_local_flock(&self, flock: &Vec<Boid>, flocksize: usize) -> LocalFlock {
        let perception_radius = 50.0;
        let mut local_flock = LocalFlock {
            flock: Vec::new(),
            size: 0,
        };

        for i in 0..flocksize {
            if flock[i].positions != self.positions {
                let distance = flock[i].origin.distance_to(self.origin);
                if distance < perception_radius {
                    local_flock.flock.push(flock[i]);
                    local_flock.size += 1;

                    if local_flock.size > LOCAL_FLOCK_SIZE {
                        break;
                    }
                }
            }
        }
        local_flock
    }

    fn get_rotation(&self, other: Vector2) -> f32 {
        let delta = Vector2::new(self.origin.x - other.x, self.origin.y - other.y);
        unsafe { atan2f(-delta.x, delta.y) }
    }

    fn get_cohesion(&self, local_flock: &LocalFlock) -> f32 {
        if local_flock.size == 0 {
            return self.rotation;
        }

        let mut mean = Vector2::zero();

        for i in 0..local_flock.size {
            mean.x = mean.x + local_flock.flock[i].origin.x;
            mean.y = mean.y + local_flock.flock[i].origin.y;
        }

        mean = mean / local_flock.size as f32;

        return self.get_rotation(mean);
    }

    fn get_alignment(&self, local_flock: &LocalFlock) -> f32 {
        if local_flock.size == 0 {
            return self.rotation;
        }
        let mut total_rotation = 0.0;
        for i in 0..local_flock.size {
            total_rotation += local_flock.flock[i].rotation;
        }
        total_rotation / local_flock.size as f32
    }

    fn get_seperation(&self, local_flock: &LocalFlock) -> f32 {
        if local_flock.size == 0 {
            return self.rotation;
        }

        let mut distances = Vec::new();
        for i in 0..local_flock.size {
            distances.push(self.origin.distance_to(local_flock.flock[i].origin));
        }

        let mut closest_local_distance = 0.0;
        let mut closest_local_rotation = 0.0;

        for i in 0..local_flock.size {
            if distances[i] < closest_local_distance || closest_local_rotation == 0.0 {
                closest_local_rotation = self.get_rotation(local_flock.flock[i].origin);
                closest_local_distance = distances[i];
            }
        }

        if closest_local_distance > 5.0 {
            return self.rotation;
        }
        return unsafe {
            fmod(
                closest_local_rotation as f64 + std::f64::consts::PI,
                2.0 * std::f64::consts::PI,
            ) as f32
        };
    }

    pub fn update(&mut self, flock: &Vec<Boid>, flocksize: usize) {
        let now = unsafe { GetTime() };
        let delta_time = now - self.last_update;

        let local_flock = self.get_local_flock(flock, flocksize);
        let mut closest_boid = -1.0;
        for i in 0..local_flock.size {
            let distance = self.origin.distance_to(local_flock.flock[i].origin);

            if distance < closest_boid || closest_boid == -1.0 {
                closest_boid = distance;
            }
        }

        let alignment = self.get_alignment(&local_flock);
        let cohesion = self.get_cohesion(&local_flock);
        let seperation = self.get_seperation(&local_flock);
        let mut target_rotation = alignment - self.rotation;

        if f32::abs(self.rotation - alignment) > 0.0 && closest_boid > 0.0 {
            if closest_boid >= 30.0 {
                target_rotation = cohesion - self.rotation;
            }
            if closest_boid <= 10.0 {
                target_rotation = seperation - self.rotation;
            }
            let val = (target_rotation < 0.0) as i32;
            target_rotation = unsafe { fmod(target_rotation as f64, std::f64::consts::PI) } as f32
                + val as f32 * 2.0 * std::f32::consts::PI;

            if target_rotation > std::f32::consts::PI {
                target_rotation = unsafe {
                    fmod(
                        target_rotation as f64 + std::f64::consts::PI,
                        2.0 * std::f64::consts::PI,
                    ) as f32
                } - std::f32::consts::PI;
            }

            let max_rotation = self.angular_vel * delta_time as f32;

            if target_rotation > max_rotation {
                target_rotation = max_rotation;
            }
            if target_rotation < max_rotation {
                target_rotation = -max_rotation;
            }
        }

        self.rotate(target_rotation);

        let velocity = Vector2::new(
            self.rotation.sin() * self.velocity.x,
            self.rotation.cos() * self.velocity.y,
        );

        self.origin = Vector2::new(
            self.origin.x + velocity.x * delta_time as f32,
            self.origin.y + velocity.y * delta_time as f32,
        );

        self.origin.x =
            unsafe { fmod(self.origin.x as f64 + std::f64::consts::PI, WIDTH as f64) as f32 }
                - std::f32::consts::PI;
        self.origin.y =
            unsafe { fmod(self.origin.y as f64 + std::f64::consts::PI, HEIGHT as f64) as f32 }
                - std::f32::consts::PI;

        self.last_update = now;
    }

    fn rotate(&mut self, theta: f32) {
        let rotation_delta = self.rotation + theta;

        for i in 0..3 {
            let x = self.positions[i].x;
            let y = self.positions[i].y;

            self.positions[i].x = theta.cos() * x - theta.sin() * y;
            self.positions[i].y = theta.sin() * x + theta.cos() * y;
        }

        self.rotation = unsafe { fmod(rotation_delta as f64, 2.0 * std::f64::consts::PI) as f32 };
    }

    pub fn draw(&self, ctx: &mut RaylibDrawHandle) {
        let mut screenpos: [Vector2; 3] = [self.positions[0], self.positions[1], self.positions[2]];
        for i in 0..3 {
            screenpos[i] = Vector2::new(
                screenpos[i].x + self.origin.x,
                screenpos[i].y + self.origin.y,
            );
        }
        ctx.draw_triangle(screenpos[0], screenpos[1], screenpos[2], Color::PINK);
        ctx.draw_triangle_lines(screenpos[0], screenpos[1], screenpos[2], Color::BLACK);
    }
}
