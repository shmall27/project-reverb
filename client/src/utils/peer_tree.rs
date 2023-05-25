use std::ops::Range;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Copy, Clone)]
pub struct PeerNode {
    pub ip: SocketAddr,
    pub connections_offset: u8,
    pub max_connections: u8,
}

#[derive(Debug, Clone)]
pub struct PeerLevel {
    pub data: Vec<PeerNode>,
    pub max_connections: u8,
}

pub struct PeerTree {
    pub data: Vec<PeerLevel>
}

#[derive(Debug)]

pub struct PeerLocation {
    breadth: u8,
    depth: u8,
}

#[derive(Debug)]
pub struct PeerWithLocation {
    peer: PeerNode,
    location: PeerLocation,
}

impl PeerWithLocation {
    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut peer_bytes = Vec::new();

        let mut ip_bytes = match self.peer.ip.ip() {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };

        let port_bytes = self.peer.ip.port().to_be_bytes().to_vec();

        ip_bytes.extend(port_bytes);

        peer_bytes.extend(ip_bytes);
        peer_bytes.push(self.peer.connections_offset);
        peer_bytes.push(self.peer.max_connections);
        peer_bytes.push(self.location.breadth);
        peer_bytes.push(self.location.depth);

        return peer_bytes;
    }
}

impl PeerTree {
    pub fn new() -> Self {
        Self {
            data: Vec::new()
        }
    }


    fn _breadth_search(level: &mut PeerLevel, range: Range<usize>, peer: &PeerNode) -> usize {  
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

    pub fn insert_peer(&mut self, peer: PeerNode) -> Vec<PeerWithLocation>{
        let mut depth = 0;
        let mut insert_diff: Vec<PeerWithLocation> = Vec::new();
        let dimensions = Self::_depth_search(self, &peer, &mut depth);
        self.data[dimensions.0].max_connections += peer.max_connections;
        self.data[dimensions.0].data.insert(dimensions.1, peer);
        // need to accumulate the overall number of recursive changes to pass to the next peer
        // in most scenarios, there won't need to be recursive calls
        // if there are, it will probably be just one or two


        // remove weakest peer if the level is full
        if dimensions.0 != 0 && self.data[dimensions.0 - 1].max_connections < self.data[dimensions.0].data.len().try_into().unwrap() {
            let weakest_peer = self.data[dimensions.0].data.remove(0);
            self.data[dimensions.0].max_connections -= weakest_peer.max_connections;

            insert_diff.extend(self.insert_peer(weakest_peer));
        }
        return insert_diff;
    }

    pub fn easy_insert_peer(&mut self, peer: PeerNode, location: PeerLocation) {
        self.data[location.depth as usize].data.insert(location.breadth as usize, peer);
    }

    pub fn pretty_print(&self) {
        for level in &self.data {
            let level_connections: Vec<u8> = level.data.iter().map(|peer| peer.max_connections).collect();
            println!("{:?}", level_connections);
        }
    }
}
