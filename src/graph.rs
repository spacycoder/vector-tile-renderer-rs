

pub Node {
    MeshNode,
    MeshNode,
}

#[derive(Debug)]
pub struct Graph {
    root: Handle<Node>,
    pool: Pool<Node>,
    stack: Vec<Handle<Node>>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            root: Handle::NONE,
            pool: Pool::new(),
            stack: Vec::new(),
        }
    }
}

impl Graph {
    pub fn new() -> Self {
        let mut root = Node::Base(Default::default());
        let root = pool.spawn(root);
        Self {
            stack: Vec::new(),
            root,
            pool,
        }
    }
}