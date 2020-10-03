extern crate nalgebra_glm as glm;

use super::camera::Camera;
use super::model::Model;
use super::player::Player;
use super::renderable::Renderable;
use super::state::State;
use std::mem::ManuallyDrop;
use std::pin::Pin;

// Used to crete an unholy abomination upon which you should not cast your gaze.
// This ended up being a necessity due to wanting to keep the code written by students as "straight forward" as possible
// It is very very double plus ungood Rust, and intentionally leaks memory like a sieve. But it works, and you're more than welcome to pretend it doesn't exist!
// In case you're curious about how it works; It allocates memory on the heap (Box), promises to prevent it from being moved or deallocated until dropped (Pin)
// and finally prevents the compiler from dropping it automatically at all (ManuallyDrop). If that sounds like a janky solution, it's because it is.
// Prettier, Rustier and better solutions were tried numerous times, but were all found wanting of having what I arbitrarily decided to be the required level of
// simplicity of use.
pub type Node = ManuallyDrop<Pin<Box<SceneNode>>>;

pub struct SceneGraph {
    pub root: Node,
}

impl SceneGraph {
    pub fn new(scene_node: Node) -> SceneGraph {
        SceneGraph { root: scene_node }
    }

    pub fn update_transforms(&mut self, state: &State) {
        let mut transforms: Vec<glm::Mat4> = vec![glm::identity()];
        let mut root = &mut **self.root;
        update(&mut root, &mut transforms, state);
    }

    pub fn draw_scene(
        &mut self,
        state: &State,
        view_transform: &glm::Mat4,
        projection: &glm::Mat4,
    ) {
        let mut transforms: Vec<glm::Mat4> = vec![glm::identity()];
        let mut root = &mut self.root;
        draw(
            &mut root,
            &mut transforms,
            state,
            view_transform,
            projection,
        );
    }
}

fn update(node: &mut SceneNode, transforms: &mut Vec<glm::Mat4>, state: &State) {
    if node.disabled {
        transforms.pop();
        return;
    }

    let parent_transform = transforms[transforms.len() - 1].clone();

    unsafe {
        for node in &node.children {
            let value = &mut (**node).value;
            let local_world_transform = match value {
                NodeType::Model(model) => parent_transform * model.transform,
                NodeType::Camera(camera) => {
                    camera.parent_transform = parent_transform;
                    let transform = camera.transform;
                    parent_transform * glm::inverse(&transform)
                }
                NodeType::Player(player) => {
                    let transform = player.get_transform();
                    parent_transform * transform
                }
                NodeType::None => parent_transform.clone(),
            };
            transforms.push(local_world_transform);

            update(&mut **node, transforms, state);
        }
    }
    transforms.pop();
}

fn draw(
    node: &mut SceneNode,
    transforms: &mut Vec<glm::Mat4>,
    state: &State,
    view_transform: &glm::Mat4,
    projection: &glm::Mat4,
) {
    if node.disabled {
        transforms.pop();
        return;
    }

    let parent_transform = transforms[transforms.len() - 1].clone();
    let value = &mut node.value;

    match value {
        NodeType::Model(model) => unsafe {
            model.shader_program().activate();
            model.before_render(&state);
            let shader = model.shader_program();
            shader.set_mat4("viewTransform", &view_transform);
            shader.set_mat4("projectionTransform", &projection);
            shader.set_float("u_time", state.elapsed);
            model.world_transform = parent_transform.clone();
            model.on_render(state, view_transform, projection);
        },
        _ => {}
    };

    unsafe {
        for node in &node.children {
            let value = &(**node).value;
            let local_world_transform = match value {
                NodeType::Model(model) => parent_transform * model.transform,
                NodeType::Camera(camera) => {
                    let transform = camera.transform;
                    parent_transform * glm::inverse(&transform)
                }
                NodeType::Player(player) => {
                    let transform = player.get_transform();
                    parent_transform * transform
                }
                NodeType::None => parent_transform.clone(),
            };
            transforms.push(local_world_transform);

            draw(&mut **node, transforms, state, view_transform, projection);
        }
    }
    transforms.pop();
}

pub enum NodeType {
    None,
    Model(Model),
    Player(Player),
    Camera(Camera),
}

pub struct SceneNode {
    pub value: NodeType,
    pub children: Vec<*mut SceneNode>,
    pub disabled: bool,
}

impl SceneNode {
    pub fn new(node_type: NodeType) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            value: node_type,
            children: vec![],
            disabled: false,
        })))
    }

    pub fn new_model(model: Model) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            value: NodeType::Model(model),
            children: vec![],
            disabled: false,
        })))
    }

    pub fn new_camera(camera: Camera) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            value: NodeType::Camera(camera),
            children: vec![],
            disabled: false,
        })))
    }

    pub fn new_player(player: Player) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            value: NodeType::Player(player),
            children: vec![],
            disabled: false,
        })))
    }

    pub fn add_child(&mut self, child: &SceneNode) {
        self.children
            .push(child as *const SceneNode as *mut SceneNode)
    }
}
