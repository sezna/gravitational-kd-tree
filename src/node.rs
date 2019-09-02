use crate::dimension::Dimension;
use crate::entity::Entity;
use utilities::{find_median, max_min_xyz, xyz_distances};

#[repr(C)]
#[derive(Clone)]
pub struct Node {
    split_dimension: Option<Dimension>, // Dimension that this node splits at.
    split_value: f64,                   // Value that this node splits at.
    pub left: Option<Box<Node>>,        // Left subtree.
    pub right: Option<Box<Node>>,       // Right subtree.
    pub points: Option<Vec<Entity>>,    // Vector of the points if this node is a Leaf.
    pub center_of_mass: (f64, f64, f64), /* The center of mass for this node and it's children all
                                         * together. (x, y, z). */
    total_mass: f64, // Total mass of all entities under this node.
    r_max: f64,      // Maximum radius that is a child of this node.
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
    time_step: f64,
    /// max_pts represents the maximum amount of points allowed in a node.
    theta: f64,
}

impl Node {
    // Some convenience functions.
    /// Returns a node with default values.
    pub fn new(theta: f64, time_step: f64) -> Node {
        return Node {
            split_dimension: None,
            split_value: 0.0,
            left: None,
            right: None,
            points: None,
            center_of_mass: (0.0, 0.0, 0.0), // (pos * mass) + (pos * mass) / sum of masses
            total_mass: 0.0,
            r_max: 0.0,
            x_max: 0.0,
            x_min: 0.0,
            y_max: 0.0,
            y_min: 0.0,
            z_max: 0.0,
            z_min: 0.0,
            theta: theta,
            time_step: time_step,
        };
    }

    /// Looks into its own children's maximum and minimum values, setting its own
    /// values accordingly.
    pub fn set_max_mins(&mut self) {
        let xmin = f64::min(
            self.left.as_ref().unwrap().x_min,
            self.right.as_ref().unwrap().x_min,
        );
        let xmax = f64::max(
            self.left.as_ref().unwrap().x_max,
            self.right.as_ref().unwrap().x_max,
        );
        let ymin = f64::min(
            self.left.as_ref().unwrap().y_min,
            self.right.as_ref().unwrap().y_min,
        );
        let ymax = f64::max(
            self.left.as_ref().unwrap().y_max,
            self.right.as_ref().unwrap().y_max,
        );
        let zmin = f64::min(
            self.left.as_ref().unwrap().z_min,
            self.right.as_ref().unwrap().z_min,
        );
        let zmax = f64::max(
            self.left.as_ref().unwrap().z_max,
            self.right.as_ref().unwrap().z_max,
        );
        let left_r_max = self.left.as_ref().expect("unexpected null node #7").r_max;
        let right_r_max = self.right.as_ref().expect("unexpected null node #8").r_max;
        self.r_max = f64::max(left_r_max, right_r_max);
        self.x_min = xmin;
        self.x_max = xmax;
        self.y_min = ymin;
        self.y_max = ymax;
        self.z_min = zmin;
        self.z_max = zmax;
    }
    // Used when treating a node as the sum of its parts in gravity calculations.
    /// Converts a node into an entity with the x, y, z, and mass being derived from the center of
    /// mass and the total mass of the entities it contains.
    pub fn as_entity(&self) -> Entity {
        return Entity {
            x: self.center_of_mass.0,
            y: self.center_of_mass.1,
            z: self.center_of_mass.2,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            mass: self.total_mass,
            radius: 0.0,
            theta: self.theta,
            time_step: self.time_step,
        };
    }
    // Function that is not being used anymore. Returns a vector of the node and
    // all of its subnodes.
    pub fn max_distance(&self) -> f64 {
        let x_distance = self.x_max - self.x_min;
        let y_distance = self.y_max - self.y_min;
        let z_distance = self.z_max - self.z_min;
        return f64::max(x_distance, f64::max(y_distance, z_distance));
    }

    // Traverses tree and returns first child found with points.
    pub fn traverse_tree_helper(&self) -> Vec<Entity> {
        let mut to_return: Vec<Entity> = Vec::new();
        match self.left {
            Some(ref node) => {
                to_return.append(&mut node.traverse_tree_helper());
            }
            None => (),
        }
        match self.right {
            Some(ref node) => {
                to_return.append(&mut node.traverse_tree_helper());
            }
            None => {
                to_return.append(
                    &mut (self
                        .points
                        .as_ref()
                        .expect("unexpected null node #10")
                        .clone()),
                );
            }
        }
        return to_return;
    }

    /// Takes in a mutable slice of entities and creates a recursive 3d tree structure.
    pub fn new_root_node(pts: &mut [Entity], theta: f64, max_pts: i32, time_step: f64) -> Node {
        // Start and end are probably 0 and pts.len(), respectively.
        let length_of_points = pts.len() as i32;
        let (xdistance, ydistance, zdistance) = xyz_distances(pts);
        // If our current collection is small enough to become a leaf (it has less than MAX_PTS points)
        if length_of_points <= max_pts {
            // then we convert it into a leaf node.

            // we calculate the center of mass and total mass for each axis and store it as a three-tuple.
            // This admittedly terse `fold` used to be a for loop. I refactored it for the sake of immutability.
            // I'm still unsure if this was optimal.
            let (x_total, y_total, z_total, max_radius, total_mass) =
                pts.iter().fold((0.0, 0.0, 0.0, 0.0, 0.0), |acc, pt| {
                    (
                        acc.0 + (pt.x * pt.mass),
                        acc.1 + (pt.y * pt.mass),
                        acc.2 + (pt.z * pt.mass),
                        if acc.3 > pt.radius { acc.3 } else { pt.radius },
                        acc.4 + pt.mass,
                    )
                });

            let (x_max, x_min, y_max, y_min, z_max, z_min) = max_min_xyz(pts);
            Node {
                center_of_mass: (
                    x_total / total_mass as f64,
                    y_total / total_mass as f64,
                    z_total / total_mass as f64,
                ),
                total_mass: total_mass,
                r_max: max_radius,
                points: Some(pts.to_vec()),
                left: None,
                right: None,
                split_dimension: None,
                split_value: 0.0,
                x_max: *x_max,
                x_min: *x_min,
                y_max: *y_max,
                y_min: *y_min,
                z_max: *z_max,
                z_min: *z_min,
                theta: theta,
                time_step: time_step,
            }
        // So the objective here is to find the median value for whatever axis has the greatest disparity in distance.
        // It is more efficient to pick three random values and pick the median of those as the pivot point, so that is
        // done if the vector has enough points. Otherwise, it picks the first element. FindMiddle just returns the middle
        // value of the three f64's given to it. Hopefully there is a more idomatic way to do this.
        } else {
            let mut root_node = Node::new(theta, time_step);
            let split_index;
            let (split_dimension, split_value) = if zdistance > ydistance && zdistance > xdistance {
                // "If the z distance is the greatest"
                // split on Z
                let (split_value, tmp) = find_median(Dimension::Z, pts);
                split_index = tmp;
                (Dimension::Z, split_value)
            } else if ydistance > xdistance && ydistance > zdistance {
                // "If the x distance is the greatest"
                // split on Y
                let (split_value, tmp) = find_median(Dimension::Y, pts);
                split_index = tmp;
                (Dimension::Y, split_value)
            } else {
                // "If the y distance is the greatest"
                // split on X
                let (split_value, tmp) = find_median(Dimension::X, pts);
                split_index = tmp;
                (Dimension::X, split_value)
            };
            root_node.split_dimension = Some(split_dimension);
            root_node.split_value = *split_value;
            let (mut lower_vec, mut upper_vec) = pts.split_at_mut(split_index);
            root_node.left = Some(Box::new(Node::new_root_node(
                &mut lower_vec,
                theta,
                max_pts,
                time_step,
            )));
            root_node.right = Some(Box::new(Node::new_root_node(
                &mut upper_vec,
                theta,
                max_pts,
                time_step,
            )));
            // The center of mass is a recursive definition. This finds the average COM for
            // each node.
            let left_mass = root_node
                .left
                .as_ref()
                .expect("unexpected null node #3")
                .total_mass;
            let right_mass = root_node
                .right
                .as_ref()
                .expect("unexpected null node #4")
                .total_mass;
            let (left_x, left_y, left_z) = root_node
                .left
                .as_ref()
                .expect("unexpected null node #5")
                .center_of_mass;
            let (right_x, right_y, right_z) = root_node
                .right
                .as_ref()
                .expect("unexpected null node #6")
                .center_of_mass;
            let total_mass = left_mass + right_mass;
            let (center_x, center_y, center_z) = (
                ((left_mass * left_x) + (right_mass * right_x)) / total_mass,
                ((left_mass * left_y) + (right_mass * right_y)) / total_mass,
                ((left_mass * left_z) + (right_mass * right_z)) / total_mass,
            );
            root_node.center_of_mass = (center_x, center_y, center_z);
            // TODO refactor the next two lines, as they are a bit ugly
            root_node.set_max_mins();
            return root_node;
        }
    }
}
