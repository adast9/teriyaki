use crate::parser::meta::{MetaFile, Node, Supernode};
use std::collections::HashMap;

use super::triple::Triple;

pub struct Meta {
    supernodes: HashMap<u32, Vec<u32>>,
    nodes: HashMap<u32, NodeInfo>,
}

impl Meta {
    pub fn new(supernodes: HashMap<u32, Vec<u32>>, nodes: HashMap<u32, NodeInfo>) -> Self {
        Self { supernodes, nodes }
    }

    pub fn serialize(&self) -> MetaFile {
        let mut s: Vec<Supernode> = Vec::new();
        let mut q: Vec<Node> = Vec::new();

        for (k, v) in &self.supernodes {
            s.push(Supernode {
                i: *k,
                g: v.to_vec(),
            });
        }

        for (k, v) in &self.nodes {
            q.push(Node {
                i: *k,
                p: v.parent,
                n: v.incoming.to_vec(),
                o: v.outgoing.to_vec(),
            });
        }
        return MetaFile { s, q };
    }

    pub fn deserialize(file: MetaFile) -> Self {
        let mut supernodes: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut nodes: HashMap<u32, NodeInfo> = HashMap::new();

        for snode in file.s {
            supernodes.insert(snode.i, snode.g.to_vec());
        }

        for node in file.q {
            nodes.insert(node.i, NodeInfo::new(&node.p, &node.n, &node.o));
        }
        return Self::new(supernodes, nodes);
    }

    pub fn contains(&self, node: &u32) -> bool {
        return self.nodes.contains_key(&node) || self.supernodes.contains_key(&node);
    }

    pub fn contains_supernode(&self, node: &u32) -> bool {
        return self.supernodes.contains_key(&node);
    }

    pub fn new_node(&mut self, triple: &Triple, is_sub: bool) {
        let node = if is_sub { triple.sub } else { triple.obj };
        let other = if is_sub { triple.obj } else { triple.sub };
        if self.contains(&node) {
            panic!("Trying to add new node {}, but it already exists", node);
        }
        self.nodes.insert(
            node,
            NodeInfo::new(&None, &vec![], &vec![vec![triple.pred, other]]),
        );
    }

    pub fn add_outgoing(&mut self, triple: &Triple) {
        self.nodes
            .get_mut(&triple.sub)
            .unwrap()
            .outgoing
            .push(vec![triple.pred, triple.obj]);
    }

    pub fn add_incoming(&mut self, triple: &Triple) {
        self.nodes
            .get_mut(&triple.obj)
            .unwrap()
            .incoming
            .push(vec![triple.pred, triple.sub]);
    }

    pub fn get_parent(&self, node: &u32) -> Option<u32> {
        return self.nodes.get(node).unwrap().parent;
    }

    pub fn has_parent(&self, node: &u32) -> bool {
        return self.get_parent(node).is_some();
    }

    pub fn remove_from_supernode(&mut self, node: &u32) {
        let p = self.get_parent(node).unwrap();
        self.supernodes.get_mut(&p).unwrap().retain(|x| *x != *node);
        self.nodes.get_mut(node).unwrap().remove_parent();
    }

    pub fn has_outgoing_pred(&self, node: &u32, pred: &u32) -> bool {
        if !self.contains_supernode(node) {
            for v in self.nodes.get(node).unwrap().outgoing {
                if v[0] == *pred {
                    return true;
                }
            }
            return false;
        } else {
            for v in self.supernodes.get(node).unwrap() {
                if self.has_outgoing_pred(v, pred) {
                    return true;
                }
            }
            return false;
        }
    }

    pub fn has_incoming_pred(&self, node: &u32, pred: &u32) -> bool {
        if !self.contains_supernode(node) {
            for v in self.nodes.get(node).unwrap().incoming {
                if v[0] == *pred {
                    return true;
                }
            }
            return false;
        } else {
            for v in self.supernodes.get(node).unwrap() {
                if self.has_incoming_pred(v, pred) {
                    return true;
                }
            }
            return false;
        }
    }

    pub fn supernode_len(&self, node: &u32) -> usize {
        if !self.contains_supernode(node) {
            panic!("Trying to get length of non-supernode {:?}", node);
        }
        let mut len = 0;
        return self.supernodes.get(node).unwrap().len();
    }

    pub fn to_single_node(&mut self, snode: &u32) {
        if !self.contains_supernode(snode) {
            panic!("Trying to convert non-supernode {:?} to single node", snode);
        } else if !self.supernode_len(snode) == 1 {
            panic!(
                "Trying to convert supernode {:?} to single node, but it has more than one node",
                snode
            );
        }
        let node = self.supernodes.get(snode).unwrap()[0];
        self.nodes.get_mut(&node).unwrap().remove_parent();
        self.supernodes.remove(snode);
    }

    /// Combines all nodes in `snode` into a single supernode in `stuff.supernodes`.
    /// Also updates the `parent` field of all nodes in `snode`.
    pub fn new_snode(&mut self, old: &Vec<u32>, new: &u32) {
        let mut new_snode: Vec<u32> = Vec::new();

        for n in old {
            if self.contains_supernode(&n) {
                let sn = self.supernodes.get(n).unwrap();
                new_snode.extend(sn);

                for s in sn {
                    self.nodes.get_mut(s).unwrap().set_parent(new);
                }
                self.supernodes.remove(n);
            } else {
                self.nodes.get_mut(n).unwrap().set_parent(new);
                new_snode.push(*n);
            }
        }
        self.supernodes.insert(*new, new_snode);
    }
}

pub struct NodeInfo {
    pub parent: Option<u32>,
    // todo: incoming and outgoing should be Vec<[u32;2]>
    pub incoming: Vec<Vec<u32>>,
    pub outgoing: Vec<Vec<u32>>,
}

impl NodeInfo {
    pub fn new(parent: &Option<u32>, incoming: &Vec<Vec<u32>>, outgoing: &Vec<Vec<u32>>) -> Self {
        NodeInfo {
            parent: parent.clone(),
            incoming: incoming.clone(),
            outgoing: outgoing.clone(),
        }
    }

    pub fn remove_parent(&mut self) {
        self.parent = None;
    }

    pub fn set_parent(&mut self, parent: &u32) {
        self.parent = Some(*parent);
    }
}
