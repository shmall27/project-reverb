use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub struct PeerNode {
    pub connections_offset: i32,
    pub max_connections: i32,
}

#[derive(Debug, Clone)]
pub struct PeerLevel {
    data: Vec<PeerNode>,
    max_connections: i32,
}

pub struct PeerTree {
    pub data: Vec<PeerLevel>
}

impl PeerTree {
    pub fn new(peer: PeerNode) -> Self {
        let level = PeerLevel {
            data: vec![peer],
            max_connections: peer.max_connections,
        };

        Self {
            data: vec![level]
        }
    }


    fn _breadth_search(level: &mut PeerLevel, range: Range<usize>, peer: &PeerNode) -> usize {  
        // maybe it doesn't like finding the item at the end of the array
        let mid_point = (range.start + range.end) / 2;
    
        if range.start >= range.end {
            return mid_point;
        }
    
        if peer.max_connections < level.data[mid_point].max_connections {
            Self::_breadth_search(level, range.start..mid_point, peer)
        } else if peer.max_connections > level.data[mid_point].max_connections {
            Self::_breadth_search(level, mid_point + 1..range.end, peer)
        } else {
            mid_point
        }
    }

    fn _depth_search(tree: &mut PeerTree, peer: &PeerNode, depth: &mut usize) -> (usize, usize) {
        while
        // not the host
        *depth == 0 || (
        // we're not at the end of the tree
        *depth < tree.data.len() && 
        // the peer's max_connections are less than the worst peer at the current level
        tree.data[*depth].data[0].max_connections >= peer.max_connections && 
        // the current level is full
        tree.data[*depth - 1].max_connections <= tree.data[*depth].data.len().try_into().unwrap()) {
            *depth += 1;
        }

        if tree.data.get(*depth).is_none() {
            tree.data.push(PeerLevel {
                data: Vec::new(),
                max_connections: 0,
            });
            return (*depth, 0);
        }

        let level_len = tree.data[*depth].data.len();
        let breadth = Self::_breadth_search(&mut tree.data[*depth], 0..level_len, peer);
        return (*depth, breadth);
    }

    pub fn insert_peer(&mut self, peer: PeerNode) {
        let mut depth = 0;
        let dimensions = Self::_depth_search(self, &peer, &mut depth);
        self.data[dimensions.0].max_connections += peer.max_connections;
        self.data[dimensions.0].data.insert(dimensions.1, peer);

        // remove weakest peer if the level is full
        if dimensions.0 != 0 && self.data[dimensions.0 - 1].max_connections < self.data[dimensions.0].data.len().try_into().unwrap() {
            let weakest_peer = self.data[dimensions.0].data.remove(0);
            self.data[dimensions.0].max_connections -= weakest_peer.max_connections;
            
            // there's an error when reinserting the weakest peer that ends up being the stronger peer in the next level
            println!("Trying to insert peer: {:?}", peer);
            println!("Reinserting weakest_peer: {:?}", weakest_peer);
            self.pretty_print();

            self.insert_peer(weakest_peer);
        }
    }

    pub fn pretty_print(&self) {
        for level in &self.data {
            let level_connections: Vec<i32> = level.data.iter().map(|peer| peer.max_connections).collect();
            println!("{:?}", level_connections);
        }
    }
}

