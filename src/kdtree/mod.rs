
// TODO list
// speed check compare the mutated accel value vs the recursive addition
// function that takes the acceleration on a particle and applies it
// function that puts all of the new particles into a new kdtree
pub mod particle;
pub mod io;
pub mod utilities;
pub mod node;
pub mod dimension;
use kdtree::utilities::*;
use kdtree::particle::Particle;
use kdtree::dimension::Dimension;
use kdtree::node::{Node, Properties};
extern crate rand;
const MAX_PTS: i32 = 3;
const THETA: f64 = 0.2;
const TIME_STEP: f64 = 0.2;

/// The main struct. Contains a root node and the total number of particles. Sort of a wrapper for
/// the recursive node structure.
pub struct KDTree {
    pub root: Node, // The root Node.
    pub number_of_particles: usize, // The number of particles in the tree.
}
// impl KDTree {
// pub fn display_tree(&self) {
// self.root.display_tree();
// }
// }
//
pub fn new_kdtree(pts: &mut Vec<Particle>) -> KDTree {
    let size_of_vec = pts.len();
    KDTree {
        root: new_root_node(pts),
        number_of_particles: size_of_vec,
    }
}
/// Returns a boolean representing whether or node the node is within the THETA range
/// of the particle.


fn theta_exceeded(particle: &Particle, node: &Node) -> bool {
    // 1) distance from particle to COM of that node
    // 2) if 1) * THETA > size (max diff) then
    let node_as_particle = node.to_particle();
    let dist = particle.distance_squared(&node_as_particle);
    let max_dist = node.max_distance();
    (dist) * (THETA * THETA) > (max_dist * max_dist)
}

/// Given a particle and a node, particle and other, returns the acceleration that other is
/// exerting on particle.
fn get_gravitational_acceleration_node(particle: &Particle, other: &Node) -> (f64, f64, f64) {
    let node_as_particle = other.to_particle();
    let d_magnitude = particle.distance(&node_as_particle);
    let d_vector = particle.distance_vector(&node_as_particle);
    let d_over_d_cubed = (d_vector.0 / d_magnitude * d_magnitude,
                          d_vector.1 / d_magnitude * d_magnitude,
                          d_vector.2 / d_magnitude * d_magnitude);
(d_over_d_cubed.0 * node_as_particle.mass,
                        d_over_d_cubed.1 * node_as_particle.mass,
                        d_over_d_cubed.2 * node_as_particle.mass)
}
/// Given two particles, particle and other, returns the acceleration that other is exerting on
/// particle.
fn get_gravitational_acceleration_particle(particle: &Particle,
                                           other: &Particle)
                                           -> (f64, f64, f64) {
    let d_magnitude = particle.distance(other);
    let d_vector = particle.distance_vector(other);
    let d_over_d_cubed = (d_vector.0 / d_magnitude * d_magnitude,
                          d_vector.1 / d_magnitude * d_magnitude,
                          d_vector.2 / d_magnitude * d_magnitude);
    (d_over_d_cubed.0 * other.mass,
                        d_over_d_cubed.1 * other.mass,
                        d_over_d_cubed.2 * other.mass)

}
/// This function creates a vector of all particles from the tree and applies gravity to them.
/// Returns a new `KDTree`.
// of note: The c++ implementation of this just stores a vector of
// accelerations and matches up the
// indexes with the indexes of the particles, and then applies them. That way
// some memory is saved.
// I am not sure if this will be necessary or very practical in the rust
// implementation (I would
// have to implement indexing in my kdtree struct).
pub fn tree_after_gravity(node: &Node) -> KDTree {
    // TODO currently there is a time when the particles are stored twice.
    // Store only accelerations perhaps?
    // perhaps store an iterator?
    let mut post_gravity_particle_vec: Vec<Particle> = traverse_tree_helper(node);
    for i in &mut post_gravity_particle_vec {
        particle_after_gravity(node, i)
    }
    new_kdtree(&mut post_gravity_particle_vec)
}
/// Takes in a particle and a node and returns the particle with the gravity from the node and all
/// subnodes applied to it.
fn particle_after_gravity(node: &Node, particle: &mut Particle) {
    let acceleration = particle_gravity(node, particle);
    let movement =
        (acceleration.0 * TIME_STEP, acceleration.1 * TIME_STEP, acceleration.2 * TIME_STEP);
    particle.add_acceleration(movement);
    particle.time_advance(TIME_STEP);
}
/// Returns the acceleration of a particle  after it has had gravity from the tree applied to it.
// In this function, we approximate some particles if they exceed a certain
// critera specified in
// "exceeds_THETA()". If we reach a node and it is a leaf, then we
// automatically get the
// acceleration from every particle in that node, but if we reach a node that
// is not a leaf and
// exceeds_THETA() returns true, then we treat the node as one giant particle
// and get the
// acceleration from it.
fn particle_gravity(node: &Node, particle: &Particle) -> (f64, f64, f64) {
    let mut acceleration = (0.0, 0.0, 0.0);
    if theta_exceeded(particle, node) {
        // TODO do we want this here?
        let tmp_accel = get_gravitational_acceleration_node(particle, node);
                        acceleration.0 += tmp_accel.0;
                        acceleration.1 += tmp_accel.1;
                        acceleration.2 += tmp_accel.2;
    } else {
        match *node {
            Node::Leaf { ref points, .. } => {
                // if the node is a leaf
                for i in points {
                    let tmp_accel = get_gravitational_acceleration_particle(particle, i);
                        acceleration.0 += tmp_accel.0;
                        acceleration.1 += tmp_accel.1;
                        acceleration.2 += tmp_accel.2;
                }
            }
            Node::Interior { left: Some(ref left), .. } => {
                // if the node is an interior node
                if theta_exceeded(particle, node) {
                    let tmp_accel = get_gravitational_acceleration_node(particle, node);
                        acceleration.0 += tmp_accel.0;
                        acceleration.1 += tmp_accel.1;
                        acceleration.2 += tmp_accel.2;
                } else {
                    match **left {
                        Node::Interior { .. } | Node::Leaf { .. } => {
                            let tmp_accel = particle_gravity(left, particle);
                        acceleration.0 += tmp_accel.0;
                        acceleration.1 += tmp_accel.1;
                        acceleration.2 += tmp_accel.2;
                        }
                    }
                }
            }
            Node::Interior { right: Some(ref right), .. } => {
                match **right {
                    Node::Interior { .. } | Node::Leaf{ .. }=> {
                        let tmp_accel = particle_gravity(right, particle);
                        acceleration.0 += tmp_accel.0;
                        acceleration.1 += tmp_accel.1;
                        acceleration.2 += tmp_accel.2;
                    }
                }
            }
            _ => {}
        }

    }

    (acceleration.0 + acceleration.0,
            acceleration.1 + acceleration.1,
            acceleration.2 + acceleration.2)
}

// if node.points.is_some() {
// If the node is a leaf
// for i in node.points.as_ref().expect("unexpected null node 1") {
// recurse through particles
// let tmp_accel = get_gravitational_acceleration_particle(particle, i);
// acceleration.0 = acceleration.0 + tmp_accel.0;
// acceleration.1 = acceleration.1 + tmp_accel.1;
// acceleration.2 = acceleration.2 + tmp_accel.2;
// }
// } else if theta_exceeded(&particle, &node) {
// otherwise, check if THETA is exceeded.
// let tmp_accel = get_gravitational_acceleration_node(&particle, &node);
// acceleration.0 = acceleration.0 + tmp_accel.0; // if THETA was exceeded, then
// acceleration.1 = acceleration.1 + tmp_accel.1; // get the force from the
// node's
// acceleration.2 = acceleration.2 + tmp_accel.2; // COM and mass
// } else {
// let tmp_accel = particle_gravity(&node, &particle);
// acceleration.0 = acceleration.0 + tmp_accel.0; // otherwise recurse
// acceleration.1 = acceleration.1 + tmp_accel.1;
// acceleration.2 = acceleration.2 + tmp_accel.2;
// }
// }
//
// None => (),
// }
// match node.right {
// Some(ref node) => {
// if node.points.is_some() {
// same logic as above
// for i in node.points.as_ref().expect("unexpected null node 2") {
// let tmp_accel = get_gravitational_acceleration_particle(particle, i);
// acceleration.0 = acceleration.0 + tmp_accel.0;
// acceleration.1 = acceleration.1 + tmp_accel.1;
// acceleration.2 = acceleration.2 + tmp_accel.2;
// }
// } else if theta_exceeded(&particle, &node) {
// TODO
// let tmp_accel = get_gravitational_acceleration_node(&particle, &node);
// acceleration.0 = acceleration.0 + tmp_accel.0;
// acceleration.1 = acceleration.1 + tmp_accel.1;
// acceleration.2 = acceleration.2 + tmp_accel.2;
// } else {
// let tmp_accel = particle_gravity(&node, &particle);
// acceleration.0 = acceleration.0 + tmp_accel.0;
// acceleration.1 = acceleration.1 + tmp_accel.1;
// acceleration.2 = acceleration.2 + tmp_accel.2;
// }
// }
// None => (),
// }
//
/// Takes in a mutable slice of particles and creates a recursive 3d tree structure.
fn new_root_node(pts: &mut [Particle]) -> Node {
    // Start and end are probably 0 and pts.len(), respectively.
    let start = 0 as usize;
    let end = pts.len();
    let length_of_points = pts.len() as i32;
    let (xmax, xmin) = max_min_x(pts);
    let (ymax, ymin) = max_min_y(pts);
    let (zmax, zmin) = max_min_z(pts);
    let xdistance = (xmax - xmin).abs();
    let ydistance = (ymax - ymin).abs();
    let zdistance = (zmax - zmin).abs();
    if length_of_points <= MAX_PTS {
        // Here we calculate the center of mass and total mass for each axis and store
        // it as a three-tuple.
        let mut totalmass = 0.0;
        let mut max_radius = 0.0;
        let (mut x_total, mut y_total, mut z_total) = (0.0, 0.0, 0.0);
        for point in pts.iter() {
            x_total += point.x * point.mass; // add up the vector and weight it by mass
            y_total += point.y * point.mass;
            z_total += point.z * point.mass;
            totalmass += point.mass;
            if point.radius > max_radius {
                max_radius = point.radius;
            }
        }
        let (xmax, xmin, ymax, ymin, zmax, zmin) = max_min_xyz(pts);
        let com =
            (x_total / totalmass as f64, y_total / totalmass as f64, z_total / totalmass as f64);
        let props = Properties {
            x_max: xmax,
            x_min: xmin,
            z_max: zmax,
            z_min: zmin,
            y_max: ymax,
            y_min: ymin,
            center_of_mass: com,
            total_mass: totalmass,
            r_max: max_radius,
        };
        Node::Leaf {
            properties: props,
            points: pts.to_vec(),
        }
    } else {
        //        let mut root_node = Node::new_interior();
        let split_index;
        let mid = (start + end) / 2 as usize;
        let (split_dimension, split_value) = if zdistance > ydistance && zdistance > xdistance {
            // "If the z distance is the greatest"
            // split on Z
            let (split_value, tmp) = find_median_z(pts, start, end, mid);
            split_index = tmp;
            (Dimension::Z, split_value)
        } else if ydistance > xdistance && ydistance > zdistance {
            // "If the x distance is the greatest"
            // split on Y
            let (split_value, tmp) = find_median_y(pts, start, end, mid);
            split_index = tmp;
            (Dimension::Y, split_value)
        } else {
            // "If the y distance is the greatest"
            // split on X
            let (split_value, tmp) = find_median_x(pts, start, end, mid);
            split_index = tmp;
            (Dimension::X, split_value)
        };
        let (mut lower_vec, mut upper_vec) = pts.split_at_mut(split_index);
        let left = Some(Box::new(new_root_node(&mut lower_vec)));
        let right = Some(Box::new(new_root_node(&mut upper_vec)));
        // The center of mass is a recursive definition. This finds the average COM for
        // each node.
        let left_mass = left.as_ref()
            .expect("unexpected null node #3")
            .properties()
            .total_mass;
        let right_mass = right.as_ref()
            .expect("unexpected null node #4")
            .properties()
            .total_mass;
        let (left_x, left_y, left_z) = left.as_ref()
            .expect("unexpected null node #5")
            .properties()
            .center_of_mass;
        let (right_x, right_y, right_z) = right.as_ref()
            .expect("unexpected null node #6")
            .properties()
            .center_of_mass;
        let total_mass = left_mass + right_mass;
        let (center_x, center_y, center_z) =
            (((left_mass * left_x) + (right_mass * right_x)) / total_mass,
             ((left_mass * left_y) + (right_mass * right_y)) / total_mass,
             ((left_mass * left_z) + (right_mass * right_z)) / total_mass);
        let center_of_mass = (center_x, center_y, center_z);
        // TODO refactor the next two lines, as they are a bit ugly
        let left_r_max = left.as_ref().expect("unexpected null node #7").properties().r_max;
        let right_r_max = right.as_ref().expect("unexpected null node #8").properties().r_max;
        let r_max = f64::max(left_r_max, right_r_max);
        let x_min = f64::min(left.as_ref().expect("").properties().x_min,
                             right.as_ref().expect("").properties().x_min);
        let x_max = f64::max(left.as_ref().expect("").properties().x_max,
                             right.as_ref().expect("").properties().x_max);
        let y_min = f64::min(left.as_ref().expect("").properties().y_min,
                             right.as_ref().expect("").properties().y_min);
        let y_max = f64::max(left.as_ref().expect("").properties().y_max,
                             right.as_ref().expect("").properties().y_max);
        let z_min = f64::min(left.as_ref().expect("").properties().z_min,
                             right.as_ref().expect("").properties().z_min);
        let z_max = f64::max(left.as_ref().expect("").properties().z_max,
                             right.as_ref().expect("").properties().z_max);
        Node::Interior {
            split_dimension: split_dimension,
            split_value: split_value,
            left: left,
            right: right,
            properties: Properties {
                center_of_mass: center_of_mass,
                total_mass: total_mass,
                r_max: r_max,
                x_min: x_min,
                x_max: x_max,
                y_min: y_min,
                y_max: y_max,
                z_min: z_min,
                z_max: z_max,
            },
        }
    }
}
// Traverses the tree and returns a vector of all particles in the tree.
// pub fn traverse_tree(tree: &KDTree) -> Vec<Particle> {
// let node = tree.root.clone();
// let mut to_return: Vec<Particle> = Vec::new();
// match node.left {
// Some(ref node) => {
// println!("appended a particle left");
// to_return.append(&mut traverse_tree_helper(node));
// }
// None => (),
// }
// match node.right {
// Some(ref node) => {
// println!("appended a particlei right");
// to_return.append(&mut traverse_tree_helper(node));
// }
// None => {
// to_return.append(&mut (node.points
// .as_ref()
// .expect("unexpected null node #9")
// .clone()));
// }
// }
// return to_return;
// return node.points.as_ref().expect("unexpected null vector of points");
//
// }
// Traverses tree and returns first child found with points.
pub fn traverse_tree_helper(node: &Node) -> Vec<Particle> {
    let mut to_return: Vec<Particle> = Vec::new();
    match *node {
        Node::Interior { .. } => {
            to_return.append(&mut traverse_tree_helper(node));
        }
        Node::Leaf { ref points, .. } => {
            to_return.append(&mut points.clone());
        }
    }
    to_return
    // match node.left {
    // Some(ref node) => {
    // to_return.append(&mut traverse_tree_helper(node));
    // }
    // None => (),
    // }
    // match node.right {
    // Some(ref node) => {
    // to_return.append(&mut traverse_tree_helper(node));
    // }
    // None => {
    // to_return.append(&mut (node.points
    // .as_ref()
    // .expect("unexpected null node #10")
    // .clone()));
    // }
    // }
}
