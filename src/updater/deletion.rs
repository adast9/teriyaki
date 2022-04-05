use std::collections::HashMap;

use crate::parser::{clique::Clique, meta_parser::NodeInfo, triple::Triple, Stuff};

use super::funcs::{remove_from_supernode, remove_parent, to_single_node, get_node_index};

pub fn delete_triple(
    deletions: &Vec<Triple>,
    source_clique: &mut Vec<Clique>,
    target_clique: &mut Vec<Clique>,
    stuff: &mut Stuff,
) {
    for deletion in deletions {
        handle_deletion(stuff, deletion, source_clique, target_clique);
    }
}

fn handle_deletion(
    stuff: &mut Stuff,
    triple: &Triple,
    source_clique: &mut Vec<Clique>,
    target_clique: &mut Vec<Clique>,
) {
    let sub = stuff.nodes.get(&triple.sub).unwrap();
    let obj = stuff.nodes.get(&triple.obj).unwrap();
    let p_sub = sub.parent;
    let p_obj = obj.parent;

    if let Some(p_sub) = p_sub {
        let supernode_id_for_sub = stuff.supernodes.get(&p_sub).unwrap().clone()[0];
        remove_from_supernode(stuff, supernode_id_for_sub, &p_sub);
        handle_split(stuff, &p_sub);
        handle_singleton_supernodes(stuff, source_clique, target_clique, &p_sub);
    } else {
        // Remove outgoing edges from sub
        remove_from_node(&mut stuff.nodes, triple, true);
        // Remove node from source clique
        remove_preds_in_clique(&stuff, sub, triple, source_clique, true);

    }
    if let Some(p_obj) = p_obj {
        let supernode_id_for_obj = stuff.supernodes.get(&p_obj).unwrap().clone()[0];
        remove_from_supernode(stuff, supernode_id_for_obj, &p_obj);
        handle_split(stuff, &p_obj);
        handle_singleton_supernodes(stuff, source_clique, target_clique, &p_obj);
    } else {
        remove_from_node(&mut stuff.nodes, triple, false);
    }
}

/// Removes incoming or outgoing edges from `node`.
fn remove_triple_from_node(nodes: &mut HashMap<u32, NodeInfo>, triple: &Triple) {
    remove_from_node(nodes, triple, true);
    remove_from_node(nodes, triple, false);
}

fn remove_preds_in_clique(
    stuff: &mut Stuff,
    nodeinfo: &NodeInfo,
    triple: &Triple,
    clique: &mut Vec<Clique>,
    is_sub: bool
) {
    let node = if is_sub { triple.sub } else { triple.obj };

    let node_index = get_node_index(stuff, &node, 0);
    let outgoing_edges = group_outgoing_edges(nodeinfo);

    if clique[node_index].nodes.len().clone() == 1 {
        // Check if clique[node_index].preds contains outgoing_edges
        for i in 0..clique[node_index].preds.len() {
            clique[node_index].preds.retain(|x| x != &outgoing_edges[i]);
        }
    }
}

/// Group all outgoing edges into one vector
fn group_outgoing_edges(nodeinfo: &NodeInfo) -> Vec<u32> {
    let mut outgoing_edges: Vec<u32> = Vec::new();
    for edge in &mut nodeinfo.outgoing.clone() {
        outgoing_edges.append(edge);
    }
    return outgoing_edges;
}

// If empty then move sub in source clique to empty clique

// Check if edges is empty
fn is_edges_empty(nodeinfo: &NodeInfo, is_outgoing: bool) -> bool {
    let edges = if is_outgoing { nodeinfo.outgoing } else { nodeinfo.incoming };

    if edges.is_empty() {
        return true;
    }
    return false;
}

fn remove_from_node(nodes: &mut HashMap<u32, NodeInfo>, triple: &Triple, is_sub: bool) {
    let node = if is_sub { triple.sub } else { triple.obj };
    let mut nodeinfo = nodes.get(&node).unwrap();

    if is_sub {
        nodeinfo
        .outgoing
        .retain(|x| x[1] != node && x[0] != triple.pred);
    } else {
        nodeinfo
        .incoming
        .retain(|x| x[1] != node && x[0] != triple.pred);
    }
}

/// Handles the case where a supernode only contains one node.
fn handle_singleton_supernodes(
    stuff: &mut Stuff,
    sc: &mut Vec<Clique>,
    tc: &mut Vec<Clique>,
    snode: &u32,
) {
    let source_index = stuff.index_map.get(snode).unwrap()[0];
    let target_index = stuff.index_map.get(snode).unwrap()[1];

    // if the node is the only node in the parent, remove the parent
    if stuff.supernodes.get(snode).unwrap().len() == 1 {
        to_single_node(stuff, sc, tc, &snode, source_index, target_index);
    }
}

/// Handles the split of a supernode.
fn handle_split(stuff: &mut Stuff, node: &u32) {
    let nodeinfo = stuff.nodes.get(node).unwrap();
    let supernode_id = nodeinfo.parent.unwrap();
    let mut snode = stuff.supernodes.get(&supernode_id).unwrap().clone();
    let node_inc = nodeinfo.incoming.clone();
    let node_out = nodeinfo.outgoing.clone();
    snode.retain(|x| x != node);

    let (rest_inc, rest_out) = get_inc_and_out(&stuff.nodes, &snode);

    if intersects_for_vec_vec(&node_inc, &rest_inc) || intersects_for_vec_vec(&node_out, &rest_out)
    {
        // if there is no intersection, remove the supernode
        remove_from_supernode(stuff, supernode_id, node);
    }
}

/// Returns a tuple of incoming and outgoing edges for a node.
fn get_inc_and_out(
    nodes: &HashMap<u32, NodeInfo>,
    snode: &Vec<u32>,
) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    let mut inc: Vec<Vec<u32>> = Vec::new();
    let mut out: Vec<Vec<u32>> = Vec::new();

    for n in snode {
        let nodeinfo = nodes.get(n).unwrap();
        inc.append(&mut nodeinfo.incoming.clone());
        out.append(&mut nodeinfo.outgoing.clone());
    }
    return (inc, out);
}

fn intersects_for_vec_vec(v1: &Vec<Vec<u32>>, v2: &Vec<Vec<u32>>) -> bool {
    for n in v1 {
        if v2.contains(&n) {
            return true;
        }
    }
    return false;
}
