// Uncomment these following global attributes to silence most warnings of "low" interest:

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

// assignment 3
mod shader;
mod util;
mod mesh;
mod scene_graph;
mod toolbox;

use scene_graph::SceneNode;
use gl::{BufferData, GenBuffers};
use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use toolbox::Heading;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors:&Vec<f32>, normals:&Vec<f32>) -> u32 {
    // * Generate a VAO and bind it
    let mut vertexArrIDs: u32 = 0;
    gl::GenVertexArrays(1, &mut vertexArrIDs);
    gl::BindVertexArray(vertexArrIDs);

    // * Generate a VBO and bind it
    let mut bufferIDs: u32 = 0;
    gl::GenBuffers(1, &mut bufferIDs);
    gl::BindBuffer(gl::ARRAY_BUFFER, bufferIDs);

    // * Fill it with data
    BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertices),
        pointer_to_array(vertices),
        gl::STATIC_DRAW,
    );

    // * Configure a VAP for the data and enable it
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, offset::<u32>(0));
    gl::EnableVertexAttribArray(0);


    // VBO for normals
    let mut normalBuffer: u32 = 0;
    GenBuffers(1, &mut normalBuffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, normalBuffer);

    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals),
        pointer_to_array(normals),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, offset::<u32>(0));
    gl::EnableVertexAttribArray(2);


    // VBO for colors --RGBA
    let mut colorBuffer: u32 = 0;
    gl::GenBuffers(1, &mut colorBuffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, colorBuffer);

    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(colors),
        pointer_to_array(colors),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, offset::<u32>(0));
    gl::EnableVertexAttribArray(1);


    // * Generate a IBO and bind it
    let mut indicesBufferIDs: u32 = 0;
    gl::GenBuffers(1, &mut indicesBufferIDs);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indicesBufferIDs);

    // * Fill it with data
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW,
    );

    // * Return the ID of the VAO
    return vertexArrIDs;
}



fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(
            INITIAL_SCREEN_W,
            INITIAL_SCREEN_H,
        ));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

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

        // == // Set up your VAO around here

        // terrain
        let terrain_mesh = mesh::Terrain::load("./resources/lunarsurface.obj");
        let terrain_vao = unsafe { create_vao(&terrain_mesh.vertices, &terrain_mesh.indices,&terrain_mesh.colors,&terrain_mesh.normals) };
        let mut terrain_node = SceneNode::from_vao(terrain_vao,terrain_mesh.index_count);

        // helicopter
        let heli_mesh = mesh::Helicopter::load("./resources/helicopter.obj");
        let body_vao = unsafe { create_vao(&heli_mesh.body.vertices, &heli_mesh.body.indices,&heli_mesh.body.colors,&heli_mesh.body.normals) };
        let door_vao = unsafe { create_vao(&heli_mesh.door.vertices, &heli_mesh.door.indices,&heli_mesh.door.colors,&heli_mesh.door.normals) };
        let main_rotor_vao = unsafe { create_vao(&heli_mesh.main_rotor.vertices, &heli_mesh.main_rotor.indices,&heli_mesh.main_rotor.colors,&heli_mesh.main_rotor.normals) };
        let tail_rotor_vao = unsafe { create_vao(&heli_mesh.tail_rotor.vertices, &heli_mesh.tail_rotor.indices,&heli_mesh.tail_rotor.colors,&heli_mesh.tail_rotor.normals) };

        // loop to draw 5 helicopters
        let mut heli_all_parents :Vec<scene_graph::Node> = Vec::<scene_graph::Node>::new();
        for n in 0..5 {
            let mut heli_parent_node = SceneNode::new();
            let mut body_node = SceneNode::from_vao(body_vao,heli_mesh.body.index_count);
            let mut door_node = SceneNode::from_vao(door_vao,heli_mesh.door.index_count);
            let mut main_rotor_node = SceneNode::from_vao(main_rotor_vao,heli_mesh.main_rotor.index_count);
            let mut tail_rotor_node = SceneNode::from_vao(tail_rotor_vao,heli_mesh.tail_rotor.index_count);

            heli_parent_node.add_child(&body_node);
            body_node.add_child(&door_node);
            heli_parent_node.add_child(&main_rotor_node);
            heli_parent_node.add_child(&tail_rotor_node);

            terrain_node.add_child(&heli_parent_node);

            // set-up reference point
            body_node.reference_point = glm::Vec3::new(0.0, 2.3, 0.0);
            tail_rotor_node.reference_point = glm::Vec3::new(0.35, 2.3, 10.4);

            heli_all_parents.push(heli_parent_node);
        }

        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once
        unsafe {
            let simple_shader = shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link();
            simple_shader.activate();
            
        };

        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove

        // store keyboard input motion
        let mut motionX : f32 = 0.0;
        let mut motionY : f32 = 0.0;
        let mut motionZ : f32 = 0.0;
        let mut rotationYaw : f32 = 0.0;
        let mut rotationPitch : f32 = 0.0;
        let mut moveDoorX : f32 = 0.0;
        let mut moveDoorZ : f32 = 0.0;

        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut prevous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(prevous_frame_time).as_secs_f32();
            prevous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Resized");
                    unsafe {
                        gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);
                    }
                }
            }

            

            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        // Move sideways
                        VirtualKeyCode::A => {
                            motionX += 20.0 * delta_time;
                        }
                        VirtualKeyCode::D => {
                            motionX -= 20.0 * delta_time;
                        }

                        // Move up/down
                        VirtualKeyCode::S => {
                            motionY += 20.0 * delta_time;
                        }
                        VirtualKeyCode::W => {
                            motionY -= 20.0 * delta_time;
                        }

                        // Zoom in/out
                        VirtualKeyCode::Space => {
                            motionZ += 20.0 * delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            motionZ -= 20.0 * delta_time;
                        }

                        // Yaw rotation
                        VirtualKeyCode::Left=> {
                            rotationYaw += 20.0 * delta_time;
                        }
                        VirtualKeyCode::Right => {
                            rotationYaw -= 20.0 * delta_time;
                        }

                        // Pitch rotation
                        VirtualKeyCode::Up=> {
                            rotationPitch += 20.0 * delta_time;
                        }
                        VirtualKeyCode::Down => {
                            rotationPitch -= 20.0 * delta_time;
                        }

                        // open the door
                        VirtualKeyCode::O=> {
                            moveDoorX = 0.1;
                            moveDoorZ = 1.0;
                        }
                        // close the door
                        VirtualKeyCode::C=> {
                            moveDoorX = 0.0;
                            moveDoorZ = 0.0;
                        }

                        // default handler:
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                // == // Optionally access the acumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            

            // == // Please compute camera transforms here (exercise 2 & 3)
            let mut view_projection_matrix;
            unsafe {
                let mut camTrans: glm::Mat4 =  glm::identity();
                // matrix for camera transformations
                camTrans = glm::rotation(rotationYaw.to_radians(), &glm::vec3(0.0, 1.0, 0.0)) * camTrans; // Yaw rotation
                camTrans = glm::rotation(rotationPitch.to_radians(), &glm::vec3(1.0, 0.0, 0.0)) * camTrans; // Pitch rotation
                camTrans = glm::translation(&glm::vec3(0.0 + motionX , 0.0 + motionY, 0.0 + motionZ)) * camTrans;

                let projection : glm::Mat4 = glm::perspective(window_aspect_ratio, 60.0f32.to_radians(), 1.0, 1000.0);

                let transZ : glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, -2.0));

                view_projection_matrix = projection * camTrans * transZ;
            };

            // animation
            for n in 0..5 {
                let mut heli_parent_node = &mut heli_all_parents[n];
                
                // position different helicopter in different place
                let posDiff:f32 = (n*30) as f32;

                // animated path
                let animatedPath:Heading = toolbox::simple_heading_animation((elapsed-delta_time)*0.5);
                heli_parent_node.position = glm::Vec3::new(posDiff + animatedPath.x,0.0,animatedPath.z);
                heli_parent_node.rotation = glm::Vec3::new(animatedPath.pitch,animatedPath.yaw,animatedPath.roll);

                // make rotors rotate
                heli_parent_node.get_child(1).rotation = glm::Vec3::new(0.0,(elapsed-delta_time) * 720.0f32.to_radians(),0.0);
                heli_parent_node.get_child(2).rotation = glm::Vec3::new((elapsed-delta_time) * 720.0f32.to_radians(),0.0,0.0);

                // open doors with "O", close with "C"
                heli_parent_node.get_child(0).get_child(0).position = glm::Vec3::new(moveDoorX,0.0,moveDoorZ);
            }
            // function to draw
            unsafe fn draw_scene(node: &scene_graph::SceneNode, view_projection_matrix: &glm::Mat4, transformation_so_far: &glm::Mat4) {
                let mut trans: glm::Mat4 = glm::identity();
                // move to origin
                trans = glm::translation(&glm::vec3(-node.reference_point.x,-node.reference_point.y,-node.reference_point.z,)) * trans;
                // rotate
                trans = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0)) * trans;
                trans = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0)) * trans;
                trans = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0)) * trans;
                // move back
                trans = glm::translation(&node.reference_point) * trans;
                
                
                trans = glm::translation(&node.position) * trans;
                trans = transformation_so_far * trans;
                
                let finalTrans : glm::Mat4 = view_projection_matrix * trans;
                
                if node.index_count>0 {
                    gl::UniformMatrix4fv(0,1,gl::FALSE,finalTrans.as_ptr());
                    gl::UniformMatrix4fv(1,1,gl::FALSE,trans.as_ptr());
                    gl::BindVertexArray(node.vao_id);
                    gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, offset::<u32>(0));
                }
                for &child in &node.children {
                    draw_scene(&*child, view_projection_matrix,&trans);
                }
                
            }

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // == // Issue the necessary gl:: commands to draw your scene here
                
                // gl::BindVertexArray(terrain_vao);
                // gl::DrawElements(gl::TRIANGLES, terrain_mesh.index_count, gl::UNSIGNED_INT, offset::<u32>(0));
                // gl::BindVertexArray(body_vao);
                // gl::DrawElements(gl::TRIANGLES, heli_mesh.body.index_count, gl::UNSIGNED_INT, offset::<u32>(0));
                // gl::BindVertexArray(door_vao);
                // // gl::DrawElements(gl::TRIANGLES, heli_mesh.door.index_count, gl::UNSIGNED_INT, offset::<u32>(0));
                // gl::BindVertexArray(main_rotor_vao);
                // gl::DrawElements(gl::TRIANGLES, heli_mesh.main_rotor.index_count, gl::UNSIGNED_INT, offset::<u32>(0));
                // gl::BindVertexArray(tail_rotor_vao);
                // gl::DrawElements(gl::TRIANGLES, heli_mesh.tail_rotor.index_count, gl::UNSIGNED_INT, offset::<u32>(0));

                draw_scene(&terrain_node, &view_projection_matrix, &glm::identity());
            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });

    // == //
    // == // From here on down there are only internals.
    // == //

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

    // Start the event loop -- This is where window events are initially handled
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
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                println!(
                    "New window size! width: {}, height: {}",
                    physical_size.width, physical_size.height
                );
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
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

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
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
            _ => {}
        }
    });
}
