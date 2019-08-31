extern crate rand;
use super::Dimension;
#[derive(Clone, PartialEq, Default)]
pub struct Particle {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub radius: f64,
    pub mass: f64,
}
impl Particle {
    // Convenience function for testing.
    /// Generates a particle with random properties.
    pub fn random_particle() -> Particle {
        return Particle {
            vx: rand::random::<f64>(),
            vy: rand::random::<f64>(),
            vz: rand::random::<f64>(),
            x: rand::random::<f64>(),
            y: rand::random::<f64>(),
            z: rand::random::<f64>(),
            radius: rand::random::<f64>(),
            mass: rand::random::<f64>(),
        };
    }
    // used in writing output
    /// Returns the particle as a string with space separated values.
    pub fn as_string(&self) -> String {
        return format!(
            "{} {} {} {} {} {} {} {}",
            self.x, self.y, self.z, self.vx, self.vy, self.vz, self.mass, self.radius
        );
    }
    /// Adds an acceleration to the velocity of the particle.
    pub fn add_acceleration(&mut self, acc: (f64, f64, f64)) {
        self.vx = self.vx + acc.0;
        self.vy = self.vy + acc.1;
        self.vz = self.vz + acc.2;
    }
    /// Adds the current velocity to the position. Takes in a duration of time.
    pub fn time_advance(&mut self, time_step: f64) {
        self.x = self.x + (self.vx * time_step);
        self.y = self.y + (self.vy * time_step);
        self.z = self.z + (self.vz * time_step);
    }
    pub fn distance_squared(&self, other: &Particle) -> f64 {
        // sqrt((x2 - x1) + (y2 - y1) + (z2 - z1))
        // all dist variables  are squared
        let x_dist = (other.x - self.x) * (other.x - self.x);
        let y_dist = (other.y - self.y) * (other.y - self.y);
        let z_dist = (other.z - self.z) * (other.z - self.z);
        let distance = x_dist + y_dist + z_dist;
        return distance;
    }
    /// Returns the distance between the two particles
    pub fn distance(&self, other: &Particle) -> f64 {
        // sqrt((x2 - x1) + (y2 - y1) + (z2 - z1))
        // all dist variables  are squared
        let x_dist = (other.x - self.x) * (other.x - self.x);
        let y_dist = (other.y - self.y) * (other.y - self.y);
        let z_dist = (other.z - self.z) * (other.z - self.z);
        let distance = f64::sqrt(x_dist + y_dist + z_dist);
        return distance;
    }
    /// Returns the distance between two particles as an (x:f64,y:f64,z:f64) tuple.
    pub fn distance_vector(&self, other: &Particle) -> (f64, f64, f64) {
        let x_dist = (other.x - self.x) * (other.x - self.x);
        let y_dist = (other.y - self.y) * (other.y - self.y);
        let z_dist = (other.z - self.z) * (other.z - self.z);
        return (x_dist, y_dist, z_dist);
    }
    /// Returns a particle with all 0.0 values.
    pub fn new() -> Particle {
        Particle::default()
    }
    pub fn get_dim(&self, dim: &Dimension) -> &f64 {
        match dim {
            &Dimension::X => &self.x,
            &Dimension::Y => &self.y,
            &Dimension::Z => &self.z,
        }
    }
}