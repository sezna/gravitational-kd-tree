use crate::entity::Entity;
use crate::Node;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[repr(C)]
// 192 bytes big
pub struct GravTree {
    pub root: Node,            // The root Node.
    number_of_entities: usize, // The number of entities in the tree.
    /// The tolerance for the distance from an entity to the center of mass of an entity
    /// If the distance is beyond this threshold, we treat the entire node as one giant
    /// entity instead of recursing into it.
    theta: f64,
    max_pts: i32,
    /// The length of time that passes each step. This coefficient is multiplied by the velocity
    /// before the velocity is added to the position of the entities each step.
    time_step: f64,
}

impl GravTree {
    pub fn new(pts: &mut Vec<Entity>, theta: f64, max_pts: i32, time_step: f64) -> GravTree {
        let size_of_vec = pts.len();
        return GravTree {
            root: Node::new_root_node(pts, theta, max_pts, time_step),
            number_of_entities: size_of_vec,
            theta: theta,
            max_pts: max_pts,
            time_step: time_step,
        };
    }
    /// Traverses the tree and returns a vector of all entities in the tree.
    pub fn as_vec(&self) -> Vec<Entity> {
        let node = self.root.clone();
        let mut to_return: Vec<Entity> = Vec::new();
        match node.left {
            Some(ref node) => {
                to_return.append(&mut node.traverse_tree_helper());
            }
            None => (),
        }
        match node.right {
            Some(ref node) => {
                to_return.append(&mut node.traverse_tree_helper());
            }
            None => {
                to_return.append(
                    &mut (node
                        .points
                        .as_ref()
                        .expect("unexpected null node #9")
                        .clone()),
                );
            }
        }
        return to_return;
        // return node.points.as_ref().expect("unexpected null vector of points");
    }
    /// Gets the total number of entities contained by this tree.
    pub fn get_number_of_entities(&self) -> usize {
        self.number_of_entities
    }

    /// This function creates a vector of all entities from the tree and applies gravity to them.
    /// Returns a new GravTree.
    // of note: The c++ implementation of this just stores a vector of
    // accelerations and matches up the
    // indexes with the indexes of the entities, and then applies them. That way
    // some memory is saved.
    // I am not sure if this will be necessary or very practical in the rust
    // implementation (I would have to implement indexing in my GravTree struct).
    pub fn time_step(&self) -> GravTree {
        // TODO currently there is a time when the entities are stored twice.
        // Store only accelerations perhaps?
        let post_gravity_entity_vec: Vec<Entity> = self.root.traverse_tree_helper();
        // for i in &mut post_gravity_entity_vec {
        //     *i = i.apply_gravity_from(&self.root);
        // }
        return GravTree::new(
            &mut post_gravity_entity_vec
                .par_iter()
                .map(|x| x.apply_gravity_from(&self.root))
                .collect(),
            self.theta,
            self.max_pts,
            self.time_step,
        );
    }

    // For now, data files are text files where there is one entity per line.
    // entities are stored as
    // x y z vx vy vz mass radius
    // TODO perhaps write the reading so that it doesn't require newlines?

    /// Reads a data file generated by this program. To see the format of this file,
    /// go to [[write_data_file]]. Takes in a file path, a theta value, a
    /// max_pts value, and a time_step. These are not encoded in the data files to allow for SwiftViz to read
    /// the files without issue. There is currently no way to offload theta, max_pts, and time_step
    /// into the data file.
    /// Returns a new GravTree with the data from the file on success, or an error
    /// message if the data format is incorrect.
    /// Panics if the file path provided is incorrect.
    pub fn from_data_file(
        file_string: String,
        theta: f64,
        max_pts: i32,
        time_step: f64,
    ) -> Result<GravTree, &'static str> {
        let file_path = Path::new(&file_string);
        let display = file_path.display();
        let mut file = match File::open(&file_path) {
            Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
            Ok(file) => file,
        };
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
            Ok(_) => (),
        }
        let mut tmp_str: String = String::new();
        let mut tmp: Vec<String> = Vec::new();
        let mut entities: Vec<Entity> = Vec::new();
        for i in s.chars() {
            if i != '\n' && i != ' ' {
                tmp_str = format!("{}{}", tmp_str, i);
            } else if i == ' ' {
                tmp.push(tmp_str);
                tmp_str = "".to_string();
            } else {
                tmp.push(tmp_str.clone());
                tmp_str = "".to_string();
                if tmp.len() == 8 {
                    // In the future, I'd like to make a super-error enum   to contain all the errors that could happen in here.
                    let x_val: f64 = tmp[0].parse().unwrap();
                    let y_val: f64 = tmp[1].parse().unwrap();
                    let z_val: f64 = tmp[2].parse().unwrap();
                    let vx_val: f64 = tmp[3].parse().unwrap();
                    let vy_val: f64 = tmp[4].parse().unwrap();
                    let vz_val: f64 = tmp[5].parse().unwrap();
                    let mass_val: f64 = tmp[6].parse().unwrap();
                    let radius_val: f64 = tmp[7].parse().unwrap();
                    let tmp_part = Entity {
                        x: x_val,
                        y: y_val,
                        z: z_val,
                        vx: vx_val,
                        vy: vy_val,
                        vz: vz_val,
                        mass: mass_val,
                        radius: radius_val,
                        theta: theta,
                        time_step: time_step,
                    };
                    entities.push(tmp_part);
                    tmp.clear();
                } else {
                    return Err("Input file invalid.");
                }
            }
        }
        return Ok(GravTree::new(&mut entities, theta, max_pts, time_step));
    }

    /// Writes a utf8 file with one entity per line, space separated values of the format:
    /// x y z vx vy vz mass radius. Must have a newline after the final entity.
    /// This is compatible with SwiftVis visualizations.
    pub fn write_data_file(self, file_path: String) {
        let mut file = File::create(file_path).unwrap(); //TODO unwraps are bad
        let mut to_write = self.as_vec();
        let mut to_write_string: String;
        to_write_string = format!("{}", to_write.pop().expect("").as_string());
        while !to_write.is_empty() {
            to_write_string = format!(
                "{}\n{}",
                to_write_string,
                to_write.pop().expect("").as_string()
            );
        }
        to_write_string = format!("{}\n", to_write_string);
        assert!(
            file.write(to_write_string.as_bytes()).unwrap() == to_write_string.as_bytes().len()
        );
    }
}
#[test]
fn test_input() {
    let test_tree = GravTree::from_data_file("test_files/test_input.txt".to_string(), 0.2, 3, 0.2);
    assert!(test_tree.unwrap().as_vec().len() == 3601);
}
#[test]
fn test_output() {
    let mut test_vec: Vec<Entity> = Vec::new();
    for _ in 0..1000 {
        test_vec.push(Entity::random_entity());
    }
    let kd = GravTree::new(&mut test_vec, 0.2, 3, 0.2);
    GravTree::write_data_file(kd, "test_files/test_output.txt".to_string());
    let test_tree = GravTree::from_data_file("test_files/test_output.txt".to_string(), 0.2, 3, 0.2);
    assert!(test_vec.len() == test_tree.unwrap().as_vec().len());
}
