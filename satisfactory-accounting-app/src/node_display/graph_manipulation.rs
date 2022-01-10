// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//! Utilities for manipulating the node graph.

use log::warn;
use satisfactory_accounting::accounting::{Group, Node, NodeKind};

/// Move a node from one position in a group to another. Both src and dest paths should be
/// rooted at this group. Assumes that this node is the lowest common ancestor of src and
/// dest, that is that src and dest have no parents in common below this node.
pub fn move_child(group: &Group, src: &[usize], dest: &[usize]) -> Option<Group> {
    let (_, src_prefix) = src.split_last().expect("source path was empty");
    let (_, dest_prefix) = dest.split_last().expect("source path was empty");
    assert!(
        src_prefix
            .iter()
            .zip(dest_prefix.iter())
            .take_while(|(s, d)| s == d)
            .count()
            == 0,
        "src and dest had overlapping prefixes"
    );
    let src_first = src.first().copied().unwrap();
    let mut dest_first = dest.first().copied().unwrap();
    if src_prefix.is_empty() && src_first < dest_first {
        // If removal of src will affect dest, change the first index of dest.
        dest_first -= 1;
    }

    if src_first >= group.children.len() {
        warn!("Attempting to move from an out of bounds index");
        return None;
    }

    let mut new_group = group.clone();
    let moved = if src_prefix.is_empty() {
        new_group.children.remove(src_first)
    } else {
        let (replacement, moved) = remove_child(&new_group.children[src_first], &src[1..])?;
        new_group.children[src_first] = replacement;
        moved
    };

    if dest_first > new_group.children.len() {
        warn!("Attempting to move to an out of boudns index");
        return None;
    }

    if dest_prefix.is_empty() {
        new_group.children.insert(dest_first, moved);
    } else {
        new_group.children[dest_first] =
            insert_child(&new_group.children[dest_first], &dest[1..], moved)?;
    }

    Some(new_group)
}

/// Recursively removes a child node. Returns the new group to replace the one modified
/// and the node that was removed. Returns none if not a group or out of bounds.
pub fn remove_child(node: &Node, child: &[usize]) -> Option<(Node, Node)> {
    let group = match node.kind() {
        NodeKind::Group(group) => group,
        _ => {
            warn!("Source for remove child did not point to a group");
            return None;
        }
    };

    let (&next_idx, rest) = child
        .split_first()
        .expect("Don't call remove_child with an empty path");

    if next_idx >= group.children.len() {
        warn!("Attempting to remove from an out of bounds index");
        return None;
    }
    let mut new_group = group.clone();
    if rest.is_empty() {
        let moved = new_group.children.remove(next_idx);
        Some((new_group.into(), moved))
    } else {
        let (replacement, moved) = remove_child(&new_group.children[next_idx], rest)?;
        new_group.children[next_idx] = replacement;
        Some((new_group.into(), moved))
    }
}

/// Recursively inserts a child node. Returns the new group to replace the one modified.
/// Returns none if not a group or out of bounds.
pub fn insert_child(node: &Node, child: &[usize], moved: Node) -> Option<Node> {
    let group = match node.kind() {
        NodeKind::Group(group) => group,
        _ => {
            warn!("Source for insert child did not point to a group");
            return None;
        }
    };

    let (&next_idx, rest) = child
        .split_first()
        .expect("Don't call insert_child with an empty path");

    if next_idx > group.children.len() {
        warn!("Attempting to insert to an out of bounds index");
        return None;
    }

    let mut new_group = group.clone();
    if rest.is_empty() {
        new_group.children.insert(next_idx, moved);
    } else {
        new_group.children[next_idx] = insert_child(&new_group.children[next_idx], rest, moved)?;
    }
    Some(new_group.into())
}
