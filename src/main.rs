extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
mod camera;
mod line_strings;
mod material;
mod mesh;
mod model;
mod player;
mod primitives;
mod renderable;
mod scene_graph;
mod shader;
mod state;
mod tile_address;
mod util;
use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
mod polygons;
use line_strings::{LineOptions, LineStringDecoder};
use polygons::{PolygonDecoder, PolygonOptions};

mod protos;
use protos::vector_tile;

use clap::{App, Arg};
use flate2::read::GzDecoder;
use glutin::event_loop::ControlFlow;
use std::io::Read;
use std::ptr;

const SCREEN_W: u32 = 1500;
const SCREEN_H: u32 = 1300;
extern crate earcutr;

fn main() {
    let matches = App::new("Vector Tile Renderer")
        .arg(
            Arg::with_name("longitude")
                .short("b")
                .long("longitude")
                .takes_value(true)
                .help("center longitude"),
        )
        .arg(
            Arg::with_name("latitude")
                .short("c")
                .long("latitude")
                .takes_value(true)
                .help("center latitude"),
        )
        .arg(
            Arg::with_name("zoom")
                .short("z")
                .long("zoom")
                .takes_value(true)
                .help("zoom"),
        )
        .arg(
            Arg::with_name("tile_radius")
                .short("t")
                .long("tile_radius")
                .takes_value(true)
                .help("tile_radius"),
        )
        .arg(
            Arg::with_name("api_key")
                .short("a")
                .long("api_key")
                .takes_value(true)
                .help("api_key"),
        )
        .get_matches();

    let center_lat = matches.value_of("latitude").unwrap_or("63.41743");
    let center_lat = match center_lat.parse::<f64>() {
        Ok(n) => n,
        Err(_) => panic!("unable to parse center_lat"),
    };

    let center_lon = matches.value_of("longitude").unwrap_or("10.40424");
    let center_lon = match center_lon.parse::<f64>() {
        Ok(n) => n,
        Err(_) => panic!("unable to parse center_lon"),
    };

    let zoom = matches.value_of("zoom").unwrap_or("17");
    let zoom = match zoom.parse::<u32>() {
        Ok(n) => n,
        Err(_) => panic!("unable to parse zoom"),
    };

    let tile_radius = matches.value_of("tile_radius").unwrap_or("2");
    let tile_radius = match tile_radius.parse::<u32>() {
        Ok(n) => n,
        Err(_) => panic!("unable to parse tile_radius"),
    };

    let api_key = matches.value_of("api_key").unwrap_or("2");
    let api_key = String::from(api_key);

    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    /*   windowed_context
        .window()
        .set_cursor_grab(true)
        .expect("failed to grab cursor");
    windowed_context.window().set_cursor_visible(false); */

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Send a copy of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // build shaders
        let shader_color = shader::Shader::new("./shaders/color.vert", "./shaders/color.frag");

        let mut green_mat = material::Material::new(shader_color);
        green_mat.set_vec4("u_color", 0.1, 0.56, 0.121, 1.0);
        let mut dark_green_mat = material::Material::new(shader_color);
        dark_green_mat.set_vec4("u_color", 0.0, 0.078, 0.05, 1.0);
        let mut yellow_mat = material::Material::new(shader_color);
        yellow_mat.set_vec4("u_color", 1.0, 0.68, 0.0001, 1.0);
        let mut red_mat = material::Material::new(shader_color);
        red_mat.set_vec4("u_color", 1.0, 0.1, 0.1, 1.0);
        let mut blue_mat = material::Material::new(shader_color);
        blue_mat.set_vec4("u_color", 0.1, 0.1, 1.0, 1.0);
        let mut gray_mat = material::Material::new(shader_color);
        gray_mat.set_vec4("u_color", 0.5, 0.5, 0.5, 1.0);
        let mut dark_gray_mat = material::Material::new(shader_color);
        dark_gray_mat.set_vec4("u_color", 0.1, 0.1, 0.1, 1.0);

        let shader_color_lit =
            shader::Shader::new("./shaders/color.vert", "./shaders/color_lit.frag");
        let mut blue_lit_mat = material::Material::new(shader_color_lit);
        blue_lit_mat.set_vec4("u_color", 0.1, 0.1, 1.0, 1.0);
        let mut gray_lit_mat = material::Material::new(shader_color_lit);
        gray_lit_mat.set_vec4("u_color", 0.5, 0.5, 0.5, 1.0);
        let mut light_gray_lit_mat = material::Material::new(shader_color_lit);
        light_gray_lit_mat.set_vec4("u_color", 0.8, 0.8, 0.8, 1.0);

        let texture_shader =
            shader::Shader::new("./shaders/texture.vert", "./shaders/texture_phong.frag");
        let texture_mat = material::Material::new(texture_shader);

        // create models
        let root = scene_graph::SceneNode::new(scene_graph::NodeType::None);
        let mut graph = scene_graph::SceneGraph::new(root);

        let center_tile = tile_address::latlon_to_tile_address(center_lat, center_lon, zoom);
        let tiles = center_tile.get_tiles(tile_radius);

        let options = vec![
            FeatureOption {
                layer: String::from("road"),
                material: dark_gray_mat.clone(),
                filter: has_key_any_value(
                    String::from("class"),
                    vec![
                        String::from("street"),
                        String::from("primary"),
                        String::from("secondary"),
                        String::from("motorway_link"),
                        String::from("motorway"),
                        String::from("path"),
                        String::from("trunk"),
                    ],
                ),
                geo_type: vector_tile::Tile_GeomType::LINESTRING,
                polygon_options: None,
                line_string_options: Some(LineOptions::new(0.11, 5.0)),
                texture: None,
            },
            FeatureOption {
                layer: String::from("road"),
                material: gray_mat.clone(),
                filter: has_key_value(String::from("class"), String::from("path")),
                geo_type: vector_tile::Tile_GeomType::LINESTRING,
                polygon_options: None,
                line_string_options: Some(LineOptions::new(0.12, 3.0)),
                texture: None,
            },
            FeatureOption {
                layer: String::from("road"),
                material: yellow_mat.clone(),
                filter: has_key_any_value(
                    String::from("class"),
                    vec![String::from("major_rail"), String::from("service_rail")],
                ),
                geo_type: vector_tile::Tile_GeomType::LINESTRING,
                polygon_options: None,
                line_string_options: Some(LineOptions::new(0.15, 1.0)),
                texture: None,
            },
            FeatureOption {
                layer: String::from("building"),
                material: gray_lit_mat.clone(),
                filter: none_filter(),
                geo_type: vector_tile::Tile_GeomType::POLYGON,
                polygon_options: Some(PolygonOptions {
                    max_height: 1.0,
                    min_height: 0.0,
                    build_walls: true,
                }),
                line_string_options: None,
                texture: None,
            },
            FeatureOption {
                layer: String::from("water"),
                material: blue_lit_mat.clone(),
                filter: none_filter(),
                geo_type: vector_tile::Tile_GeomType::POLYGON,
                polygon_options: Some(PolygonOptions {
                    max_height: 0.0,
                    min_height: 0.0,
                    build_walls: false,
                }),
                line_string_options: None,
                texture: None,
            },
            FeatureOption {
                layer: String::from("landuse"),
                material: texture_mat.clone(),
                filter: has_not_key_value(String::from("type"), String::from("rock")),
                geo_type: vector_tile::Tile_GeomType::POLYGON,
                polygon_options: Some(PolygonOptions {
                    max_height: -0.1,
                    min_height: 0.0,
                    build_walls: false,
                }),
                line_string_options: None,
                texture: Some(String::from("./assets/textures/grass.jpg")),
            },
           FeatureOption {
                layer: String::from("landuse"),
                material: gray_mat.clone(),
                filter: has_key_value(String::from("type"), String::from("rock")),
                geo_type: vector_tile::Tile_GeomType::POLYGON,
                polygon_options: Some(PolygonOptions {
                    max_height: -0.05,
                    min_height: 0.0,
                    build_walls: false,
                }),
                line_string_options: None,
                texture: None,
            },
        ];

        let scale = 100.0;
        let inverse_tile_scale =
            1.0 / (tile_address::EARTH_CIRCUMFERENCE_METERS as f64 / (1 << zoom) as f64);
        for tile_address in tiles {
            let url = format!(
                "https://api.mapbox.com/v4/mapbox.mapbox-streets-v8/{}/{}/{}.vector.pbf?access_token={}",
                tile_address.z, tile_address.x, tile_address.y, api_key
            );

            let mut reader = ureq::get(&url[..]).call().into_reader();
            let mut bytes: Vec<u8> = Vec::new();
            reader.read_to_end(&mut bytes).unwrap();
            let mut decompressor = GzDecoder::new(&bytes[..]);
            let mut bytes: Vec<u8> = Vec::new();
            decompressor.read_to_end(&mut bytes).unwrap();
            let tile: vector_tile::Tile = protobuf::parse_from_bytes(&bytes).unwrap();

            let offset_x: i32 = tile_address.x as i32 - center_tile.x as i32;
            let offset_y: i32 = tile_address.y as i32 - center_tile.y as i32;

            let translation = glm::translate(
                &glm::identity(),
                &glm::vec3(offset_x as f32 * scale, 0.0, offset_y as f32 * scale),
            );
            let tile_transform =
                translation * glm::scale(&glm::identity(), &glm::vec3(scale, 5.0, scale));

            for layer in tile.get_layers() {
                let extent = layer.get_extent();
                for option in &options {
                    if option.layer.as_str() != layer.get_name() {
                        continue;
                    }

                    let features = get_filtered_features(layer, &option.filter);

                    if option.geo_type == vector_tile::Tile_GeomType::POLYGON {
                        let polygon_options = option.polygon_options.as_ref().unwrap();
                        for feature in features {
                            let geo_type = feature.get_field_type();

                            if geo_type != option.geo_type {
                                continue;
                            }

                            let geometry = feature.get_geometry();
                            let mut options = polygon_options.clone();
                            let height = get_float(String::from("height"), &layer, &feature);
                            let min_height =
                                get_float(String::from("min_height"), &layer, &feature);
                            if let Some(min_height) = min_height {
                                options.min_height =
                                    ((min_height as f64) * inverse_tile_scale) as f32 * 7.0;
                            }

                            if let Some(height) = height {
                                options.max_height = options.min_height
                                    + ((height as f64) * inverse_tile_scale) as f32 * 7.0;
                            }

                            let mut polygon_builder = polygons::PolygonBuilder::new(options);
                            let mut decoder =
                                PolygonDecoder::new(extent, geometry, &mut polygon_builder);
                            decoder.decode();

                            let m = polygon_builder.output_mesh;
                            let indices = m.indices;
                            let vertices = m.vertices;
                            let uvs = m.uvs;
                            let normals = m.normals;
                            let mut mesh = mesh::Mesh::new_full(vertices, indices, normals, uvs);

                            if let Some(texture) = &option.texture {
                                mesh.set_texture(texture.as_str());
                            }

                            let poly_model = model::Model::new(
                                String::from("wdwdwd"),
                                vec![mesh],
                                option.material.clone(),
                                tile_transform,
                                Some(Box::new(|transform, mat, s| {
                                    mat.set_vec3(
                                        "u_viewPos",
                                        s.camera_position.x,
                                        s.camera_position.y,
                                        s.camera_position.z,
                                    );
                                    *transform
                                })),
                            );
                            let poly_node = scene_graph::SceneNode::new_model(poly_model);
                            graph.root.add_child(&poly_node);
                        }
                    } else if option.geo_type == vector_tile::Tile_GeomType::LINESTRING {
                        let line_string_options = option.line_string_options.as_ref().unwrap();
                        for feature in features {
                            let geo_type = feature.get_field_type();

                            if geo_type != option.geo_type {
                                continue;
                            }

                            let geometry = feature.get_geometry();
                            let thickness = (line_string_options.width as f64 * inverse_tile_scale)
                                as f32
                                * 1.0;
                            let height = line_string_options.height;

                            let mut decoder =
                                LineStringDecoder::new(extent, geometry, thickness, height);
                            decoder.decode();
                            let m = decoder.output_mesh;
                            let indices = m.indices;
                            let vertices = m.vertices;
                            let uvs = m.uvs;
                            let normals = m.normals;
                            if vertices.len() < 3 {
                                continue;
                            }
                            let mesh = mesh::Mesh::new_full(vertices, indices, normals, uvs);

                            let poly_model = model::Model::new(
                                String::from("line"),
                                vec![mesh],
                                option.material.clone(),
                                tile_transform,
                                Some(Box::new(|transform, mat, s| {
                                    mat.set_vec3(
                                        "u_viewPos",
                                        s.camera_position.x,
                                        s.camera_position.y,
                                        s.camera_position.z,
                                    );
                                    *transform
                                })),
                            );

                            let poly_node = scene_graph::SceneNode::new_model(poly_model);
                            graph.root.add_child(&poly_node);
                        }
                    }
                }
            }
        }

        let mut map_plane_transform: glm::Mat4 = glm::translate(
            &glm::identity(),
            &glm::vec3(
                -scale * (tile_radius as f32 + 1.0) * 1.5,
                -0.9,
                1.5 * scale * (tile_radius as f32 + 1.0),
            ),
        );
        map_plane_transform = glm::rotate_x(&map_plane_transform, -90_f32.to_radians());
        map_plane_transform = glm::scale(
            &map_plane_transform,
            &glm::vec3(
                3.0 * scale * (tile_radius as f32 + 1.0),
                3.0 * scale * (tile_radius as f32 + 1.0),
                3.0 * scale * (tile_radius as f32 + 1.0),
            ),
        );

        let mut plane_mesh = primitives::generate_quad();
        let normals = util::calculate_normals(&plane_mesh.vertices, &plane_mesh.indices);
        plane_mesh.set_normals(normals);

        let map_plane_model = model::Model::new(
            String::from("plane_mesh"),
            vec![plane_mesh],
            dark_green_mat.clone(),
            map_plane_transform,
            None,
        );

        let plane_node = scene_graph::SceneNode::new_model(map_plane_model);
        graph.root.add_child(&plane_node);

        let player = player::Player::new();
        let mut player_node = scene_graph::SceneNode::new_player(player);
        graph.root.add_child(&player_node);

        let yaw: f32 = -90.0;
        let pitch: f32 = -30.0;
        let radius = 30.0;
        let camera = camera::Camera::new(yaw, pitch, radius);
        let mut camera_node = scene_graph::SceneNode::new_camera(camera);
        player_node.add_child(&camera_node);

        let camera = &mut camera_node.value;
        let camera = match camera {
            scene_graph::NodeType::Camera(camera) => camera,
            _ => panic!("not camera"),
        };

        let player = &mut player_node.value;
        let player = match player {
            scene_graph::NodeType::Player(player) => player,
            _ => panic!("not player"),
        };

        // Set up openGLprojection
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop

        let projection_transform: glm::Mat4 = glm::perspective(
            SCREEN_W as f32 / SCREEN_H as f32,
            45_f32.to_radians(),
            1.0,
            1000.0,
        );

        let mut state = state::State {
            camera_position: camera.position,
            elapsed: 0.0,
            delta_time: 0.0,
            frame_num: 0,
            pressed_keys: vec![],
            delta_x: 0.0,
            delta_y: 0.0,
        };

        let mut frame_num = 0;
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;
            frame_num += 1;
            state.elapsed = elapsed;
            state.delta_time = delta_time;
            state.frame_num = frame_num;

            if let Ok(mut delta) = mouse_delta.lock() {
                player.process_mouse(delta.0, delta.1);
                state.delta_x = delta.0;
                state.delta_y = delta.1;
                *delta = (0.0, 0.0);
            }

            if let Ok(keys) = pressed_keys.lock() {
                state.pressed_keys = keys.clone();
                player.process_keyboard(keys, delta_time);
            }

            unsafe {
                graph.update_transforms(&state);
                state.camera_position = (camera.parent_transform
                    * glm::vec4(camera.position.x, camera.position.y, camera.position.z, 1.0))
                .xyz();

                gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // using change of coordinates:
                let view_transform = camera.get_view_transform();
                graph.draw_scene(&state, &view_transform, &projection_transform);
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}

struct FeatureOption {
    pub layer: String,
    pub material: material::Material,
    pub filter: FilterFunc,
    pub geo_type: vector_tile::Tile_GeomType,
    pub polygon_options: Option<PolygonOptions>,
    pub texture: Option<String>,
    pub line_string_options: Option<LineOptions>,
}

type FilterFunc = Box<dyn Fn(&vector_tile::Tile_Layer, &vector_tile::Tile_Feature) -> bool>;

pub fn get_filtered_features<'a>(
    layer: &'a vector_tile::Tile_Layer,
    filter: &'a FilterFunc,
) -> impl Iterator<Item = &'a vector_tile::Tile_Feature> {
    layer
        .get_features()
        .iter()
        .filter(move |feature| filter(layer, feature))
}

pub fn none_filter() -> FilterFunc {
    return Box::new(
        |layer: &vector_tile::Tile_Layer, feature: &vector_tile::Tile_Feature| {
            return true;
        },
    );
}

pub fn has_key_value(key: String, value: String) -> FilterFunc {
    return Box::new(
        move |layer: &vector_tile::Tile_Layer, feature: &vector_tile::Tile_Feature| {
            let tags = feature.get_tags();
            let mut i: usize = 0;
            while i < tags.len() {
                let prop_index = tags[i];
                let curr_key = &layer.keys[prop_index as usize];
                if curr_key == key.as_str() {
                    let values = layer.get_values();
                    let value_index = tags[i + 1];
                    let curr_value = values[value_index as usize].get_string_value();
                    if curr_value == value {
                        return true;
                    }
                }
                i += 2;
            }

            return false;
        },
    );
}

pub fn has_not_key_value(key: String, value: String) -> FilterFunc {
    return Box::new(
        move |layer: &vector_tile::Tile_Layer, feature: &vector_tile::Tile_Feature| {
            let tags = feature.get_tags();
            let mut i: usize = 0;
            while i < tags.len() {
                let prop_index = tags[i];
                let curr_key = &layer.keys[prop_index as usize];
                if curr_key == key.as_str() {
                    let values = layer.get_values();
                    let value_index = tags[i + 1];
                    let curr_value = values[value_index as usize].get_string_value();
                    if curr_value == value {
                        return false;
                    }
                }
                i += 2;
            }

            return true;
        },
    );
}

pub fn has_key_any_value(key: String, values: Vec<String>) -> FilterFunc {
    return Box::new(
        move |layer: &vector_tile::Tile_Layer, feature: &vector_tile::Tile_Feature| {
            let tags = feature.get_tags();
            let mut i: usize = 0;
            while i < tags.len() {
                let prop_index = tags[i];
                let curr_key = &layer.keys[prop_index as usize];
                if curr_key == key.as_str() {
                    let layer_values = layer.get_values();
                    let value_index = tags[i + 1];
                    let curr_value = layer_values[value_index as usize].get_string_value();
                    for value in &values {
                        if curr_value == value {
                            return true;
                        }
                    }
                }
                i += 2;
            }

            return false;
        },
    );
}

pub fn get_float(
    key: String,
    layer: &vector_tile::Tile_Layer,
    feature: &vector_tile::Tile_Feature,
) -> Option<f64> {
    let tags = feature.get_tags();
    let mut i: usize = 0;
    while i < tags.len() {
        let prop_index = tags[i];
        let curr_key = &layer.keys[prop_index as usize];
        if curr_key == key.as_str() {
            let layer_values = layer.get_values();
            let value_index = tags[i + 1];
            let curr_value = layer_values[value_index as usize].get_double_value();
            return Some(curr_value);
        }
        i += 2;
    }

    return None;
}
