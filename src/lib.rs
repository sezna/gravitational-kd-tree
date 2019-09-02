#![feature(test)]
#![deny(improper_ctypes)]
extern crate either;
extern crate rand;
extern crate test;
// TODO list
// speed check compare the mutated accel value vs the recursive addition
// function that takes the acceleration on an entity and applies it
// function that puts all of the new entities into a new GravTree
mod dimension;
pub mod entity;
pub mod gravtree;
mod node;
mod utilities;
use dimension::Dimension;
use gravtree::GravTree;
use node::Node;
use std::ffi::CStr;
use std::mem::{transmute, transmute_copy};

#[allow(unused_imports)] // this is used in the test
use entity::Entity;
/* FFI interface functions are all plopped right here. I don't know if there's a better place to put them. */

use std::os::raw::{c_char, c_double, c_int, c_uchar};
use std::slice;

#[no_mangle]
pub extern "C" fn new(
    array: *const Entity,
    length: c_int,
    theta: c_double,
    max_pts: c_int,
    time_step: c_double,
) -> *mut c_uchar {
    assert!(!array.is_null(), "Null pointer in new()");
    let array: &[Entity] = unsafe { slice::from_raw_parts(array, length as usize) };
    let mut rust_vec_of_entities = Vec::from(array);
    let gravtree = GravTree::new(
        &mut rust_vec_of_entities,
        f64::from(theta),
        i32::from(max_pts),
        f64::from(time_step),
    );
    let _gravtree = unsafe { transmute(Box::new(gravtree)) };
    return _gravtree;
}

#[no_mangle]
pub extern "C" fn time_step(gravtree_buf: *mut c_uchar) -> *mut c_uchar {
    let gravtree: GravTree = unsafe { transmute_copy(&gravtree_buf) };
    let _gravtree = unsafe { transmute(Box::new(gravtree.time_step())) };
    return _gravtree;
}

#[no_mangle]
pub extern "C" fn from_data_file(
    file_path_buff: *const c_char,
    theta: c_double,
    max_pts: c_int,
    time_step: c_double,
) -> *mut c_uchar {
    let file_path = unsafe { CStr::from_ptr(file_path_buff) };

    let gravtree = GravTree::from_data_file(
        String::from(file_path.to_str().unwrap()),
        f64::from(theta),
        i32::from(max_pts),
        f64::from(time_step),
    )
    .unwrap();
    // Seg fault is happening within the below line.
    let _gravtree = unsafe { transmute(Box::new(gravtree.time_step())) };
    return _gravtree;
}

#[no_mangle]
pub extern "C" fn write_data_file(file_path_buff: *const c_char, gravtree_buf: *mut c_uchar) {
    let gravtree: GravTree = unsafe { transmute_copy(&gravtree_buf) };
    let file_path = unsafe { CStr::from_ptr(file_path_buff) };
    gravtree.write_data_file(String::from(file_path.to_str().unwrap()));
}

#[test]
fn test_traversal() {
    let mut vec: Vec<Entity> = Vec::new();
    for _ in 0..100 {
        let entity = Entity::random_entity();
        vec.push(entity);
    }
    let vec_clone = vec.clone();
    let tree = GravTree::new(&mut vec, 0.2, 3, 0.2);
    let traversed_vec = tree.as_vec();
    let mut all_found = true;
    for i in vec_clone {
        if !traversed_vec.contains(&i) {
            all_found = false;
        }
    }

    assert!(all_found);
}

#[test]
#[allow(dead_code)]
fn test_tree() {
    let mut vec_that_wants_to_be_a_kdtree: Vec<Entity> = Vec::new();
    for _ in 0..100 {
        for _ in 0..100 {
            for _ in 0..10 {
                let entity = Entity::random_entity();
                vec_that_wants_to_be_a_kdtree.push(entity);
            }
        }
    }
    let kdtree_test = GravTree::new(&mut vec_that_wants_to_be_a_kdtree, 0.2, 3, 0.2);
    assert!(kdtree_test.get_number_of_entities() == 100000);
    // kdtree_test.display_tree();
    go_to_edges(kdtree_test, 14usize, 15usize);
    let mut smaller_vec: Vec<Entity> = Vec::new();
    for _ in 0..50 {
        let entity = Entity::random_entity();
        smaller_vec.push(entity);
    }
    // Testing center of mass assignment
    let mut vector = vec![
        Entity {
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            x: 1.0,
            y: 2.0,
            z: 3.0,
            mass: 2.0,
            radius: 1.0,
            theta: 0.2,
            time_step: 0.2,
        },
        Entity {
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            x: 2.0,
            y: 1.0,
            z: 3.0,
            mass: 2.0,
            radius: 1.0,
            theta: 0.2,
            time_step: 0.2,
        },
    ];
    let center_of_mass_test = GravTree::new(&mut vector, 0.2, 3, 0.2);
    assert!(center_of_mass_test.root.center_of_mass == (1.5, 1.5, 3.0));
}
#[allow(dead_code)]
/// This function is used for testing. It checks the number of nodes on each side, along the "edge" of the tree.
/// left_nodes is how many nodes you expect to see along the left size, and right_nodes is how many you expect to
///  see along the right.
fn go_to_edges(grav_tree: GravTree, left_nodes: usize, right_nodes: usize) {
    let mut count_of_nodes = 0;
    let mut node = grav_tree.root.left.expect("null root node\n");
    let mut node2 = node.clone();
    while node.left.is_some() {
        count_of_nodes = count_of_nodes + 1;
        node = node.left.expect("unexpected null node #1\n");
    }
    assert!(count_of_nodes == left_nodes);
    count_of_nodes = 0;
    while node2.right.is_some() {
        count_of_nodes = count_of_nodes + 1;
        node2 = node2.right.expect("unexpected null node #2\n");
    }
    assert!(count_of_nodes == right_nodes);
}
