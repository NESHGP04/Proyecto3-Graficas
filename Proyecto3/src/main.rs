use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod shader;
mod camera;
mod skybox;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::{triangle, ShaderType}; 
use shaders::vertex_shader;
use camera::Camera;
use skybox::Skybox;

pub struct Uniforms {
    model_matrix: Mat4,
}

struct CelestialBody {
    orbital_radius: f32,
    orbital_angle: f32,
    orbital_speed: f32,
    scale: f32,
    rotation: Vec3,
    shader_type: ShaderType,
    rotation_speed: f32,
    name: &'static str,
    color: u32, // Para las órbitas
}

// #[derive(Clone, Copy, PartialEq)]
// pub enum ShaderType {
//     Sun,
//     RockyPlanet,
//     GasGiant,
//     IcePlanet,
//     VolcanicPlanet,
//     Moon,
//     Spaceship,
// }

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, cos_x, -sin_x, 0.0,
        0.0, sin_x, cos_x, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sin_y, 0.0, cos_y, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z, cos_z, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0, 0.0, translation.x,
        0.0, scale, 0.0, translation.y,
        0.0, 0.0, scale, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    transform_matrix * rotation_matrix
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], shader_type: ShaderType, time: f32) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], shader_type, time));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let color = fragment.color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn draw_orbit(framebuffer: &mut Framebuffer, center: Vec3, radius: f32, color: u32, camera: &Camera, window_width: f32, window_height: f32) {
    let segments = 100;
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

        let p1 = Vec3::new(
            center.x + radius * angle1.cos(),
            center.y,
            center.z + radius * angle1.sin(),
        );

        let p2 = Vec3::new(
            center.x + radius * angle2.cos(),
            center.y,
            center.z + radius * angle2.sin(),
        );

        let screen1 = camera.get_screen_position(p1, window_width, window_height);
        let screen2 = camera.get_screen_position(p2, window_width, window_height);

        if screen1.z > 0.0 && screen2.z > 0.0 {
            draw_line(framebuffer, screen1.x as i32, screen1.y as i32, screen2.x as i32, screen2.y as i32, color);
        }
    }
}

fn draw_line(framebuffer: &mut Framebuffer, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x1;
    let mut y = y1;

    loop {
        if x >= 0 && y >= 0 && (x as usize) < framebuffer.width && (y as usize) < framebuffer.height {
            framebuffer.set_current_color(color);
            framebuffer.point(x as usize, y as usize, -1.0);
        }
        if x == x2 && y == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Dibuja texto simple en pantalla
fn draw_text(framebuffer: &mut Framebuffer, x: usize, y: usize, text: &str, color: u32) {
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            continue;
        }
        let cx = x + i * 8;
        if cx < framebuffer.width && y < framebuffer.height {
            draw_char(framebuffer, cx, y, b as char, color);
        }
    }
}

/// Dibuja un solo carácter ASCII de 8x8
fn draw_char(framebuffer: &mut Framebuffer, x: usize, y: usize, c: char, color: u32) {
    const FONT: [[u8; 8]; 95] = [
        [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], // ' '
        [0x18,0x3C,0x3C,0x18,0x18,0x00,0x18,0x00], // '!'
        [0x36,0x36,0x24,0x00,0x00,0x00,0x00,0x00], // '"'
        [0x36,0x36,0x7F,0x36,0x7F,0x36,0x36,0x00], // '#'
        [0x0C,0x3E,0x03,0x1E,0x30,0x1F,0x0C,0x00], // '$'
        [0x00,0x63,0x33,0x18,0x0C,0x66,0x63,0x00], // '%'
        [0x1C,0x36,0x1C,0x6E,0x3B,0x33,0x6E,0x00], // '&'
        [0x06,0x06,0x04,0x00,0x00,0x00,0x00,0x00], // '''
        [0x18,0x0C,0x06,0x06,0x06,0x0C,0x18,0x00], // '('
        [0x06,0x0C,0x18,0x18,0x18,0x0C,0x06,0x00], // ')'
        [0x00,0x66,0x3C,0xFF,0x3C,0x66,0x00,0x00], // '*'
        [0x00,0x0C,0x0C,0x3F,0x0C,0x0C,0x00,0x00], // '+'
        [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C,0x18], // ','
        [0x00,0x00,0x00,0x3F,0x00,0x00,0x00,0x00], // '-'
        [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C,0x00], // '.'
        [0x60,0x30,0x18,0x0C,0x06,0x03,0x01,0x00], // '/'
        [0x3E,0x63,0x73,0x7B,0x6F,0x67,0x3E,0x00], // '0'
        [0x0C,0x0E,0x0F,0x0C,0x0C,0x0C,0x3F,0x00], // '1'
        [0x1E,0x33,0x30,0x1C,0x06,0x33,0x3F,0x00], // '2'
        [0x1E,0x33,0x30,0x1C,0x30,0x33,0x1E,0x00], // '3'
        [0x38,0x3C,0x36,0x33,0x7F,0x30,0x78,0x00], // '4'
        [0x3F,0x03,0x1F,0x30,0x30,0x33,0x1E,0x00], // '5'
        [0x1C,0x06,0x03,0x1F,0x33,0x33,0x1E,0x00], // '6'
        [0x3F,0x33,0x30,0x18,0x0C,0x0C,0x0C,0x00], // '7'
        [0x1E,0x33,0x33,0x1E,0x33,0x33,0x1E,0x00], // '8'
        [0x1E,0x33,0x33,0x3E,0x30,0x18,0x0E,0x00], // '9'
        [0x00,0x0C,0x0C,0x00,0x00,0x0C,0x0C,0x00], // ':'
        [0x00,0x0C,0x0C,0x00,0x00,0x0C,0x0C,0x18], // ';'
        [0x18,0x0C,0x06,0x03,0x06,0x0C,0x18,0x00], // '<'
        [0x00,0x00,0x3F,0x00,0x00,0x3F,0x00,0x00], // '='
        [0x06,0x0C,0x18,0x30,0x18,0x0C,0x06,0x00], // '>'
        [0x1E,0x33,0x30,0x18,0x0C,0x00,0x0C,0x00], // '?'
        [0x3E,0x63,0x6F,0x6F,0x6F,0x03,0x1E,0x00], // '@'
        [0x0C,0x1E,0x33,0x33,0x3F,0x33,0x33,0x00], // 'A'
        [0x3F,0x66,0x66,0x3E,0x66,0x66,0x3F,0x00], // 'B'
        [0x3C,0x66,0x03,0x03,0x03,0x66,0x3C,0x00], // 'C'
        [0x1F,0x36,0x66,0x66,0x66,0x36,0x1F,0x00], // 'D'
        [0x7F,0x46,0x16,0x1E,0x16,0x46,0x7F,0x00], // 'E'
        [0x7F,0x46,0x16,0x1E,0x16,0x06,0x0F,0x00], // 'F'
        [0x3C,0x66,0x03,0x03,0x73,0x66,0x7C,0x00], // 'G'
        [0x33,0x33,0x33,0x3F,0x33,0x33,0x33,0x00], // 'H'
        [0x1E,0x0C,0x0C,0x0C,0x0C,0x0C,0x1E,0x00], // 'I'
        [0x78,0x30,0x30,0x30,0x33,0x33,0x1E,0x00], // 'J'
        [0x67,0x66,0x36,0x1E,0x36,0x66,0x67,0x00], // 'K'
        [0x0F,0x06,0x06,0x06,0x46,0x66,0x7F,0x00], // 'L'
        [0x63,0x77,0x7F,0x7F,0x6B,0x63,0x63,0x00], // 'M'
        [0x63,0x67,0x6F,0x7B,0x73,0x63,0x63,0x00], // 'N'
        [0x3E,0x63,0x63,0x63,0x63,0x63,0x3E,0x00], // 'O'
        [0x3F,0x66,0x66,0x3E,0x06,0x06,0x0F,0x00], // 'P'
        [0x3E,0x63,0x63,0x63,0x6B,0x33,0x5E,0x00], // 'Q'
        [0x3F,0x66,0x66,0x3E,0x36,0x66,0x67,0x00], // 'R'
        [0x1E,0x33,0x03,0x1E,0x30,0x33,0x1E,0x00], // 'S'
        [0x3F,0x2D,0x0C,0x0C,0x0C,0x0C,0x1E,0x00], // 'T'
        [0x33,0x33,0x33,0x33,0x33,0x33,0x3E,0x00], // 'U'
        [0x33,0x33,0x33,0x33,0x33,0x1E,0x0C,0x00], // 'V'
        [0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00], // 'W'
        [0x63,0x63,0x36,0x1C,0x1C,0x36,0x63,0x00], // 'X'
        [0x33,0x33,0x33,0x1E,0x0C,0x0C,0x1E,0x00], // 'Y'
        [0x7F,0x63,0x31,0x18,0x4C,0x66,0x7F,0x00], // 'Z'
        [0x1E,0x06,0x06,0x06,0x06,0x06,0x1E,0x00], // '['
        [0x03,0x06,0x0C,0x18,0x30,0x60,0x40,0x00], // '\'
        [0x1E,0x18,0x18,0x18,0x18,0x18,0x1E,0x00], // ']'
        [0x08,0x1C,0x36,0x63,0x00,0x00,0x00,0x00], // '^'
        [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xFF], // '_'
        [0x0C,0x0C,0x18,0x00,0x00,0x00,0x00,0x00], // '`'
        [0x00,0x00,0x1E,0x30,0x3E,0x33,0x6E,0x00], // 'a'
        [0x07,0x06,0x06,0x3E,0x66,0x66,0x3B,0x00], // 'b'
        [0x00,0x00,0x1E,0x33,0x03,0x33,0x1E,0x00], // 'c'
        [0x38,0x30,0x30,0x3E,0x33,0x33,0x6E,0x00], // 'd'
        [0x00,0x00,0x1E,0x33,0x3F,0x03,0x1E,0x00], // 'e'
        [0x1C,0x36,0x06,0x0F,0x06,0x06,0x0F,0x00], // 'f'
        [0x00,0x00,0x6E,0x33,0x33,0x3E,0x30,0x1F], // 'g'
        [0x07,0x06,0x36,0x6E,0x66,0x66,0x67,0x00], // 'h'
        [0x0C,0x00,0x0E,0x0C,0x0C,0x0C,0x1E,0x00], // 'i'
        [0x30,0x00,0x38,0x30,0x30,0x33,0x33,0x1E], // 'j'
        [0x07,0x06,0x66,0x36,0x1E,0x36,0x67,0x00], // 'k'
        [0x0E,0x0C,0x0C,0x0C,0x0C,0x0C,0x1E,0x00], // 'l'
        [0x00,0x00,0x33,0x7F,0x7F,0x6B,0x63,0x00], // 'm'
        [0x00,0x00,0x1F,0x33,0x33,0x33,0x33,0x00], // 'n'
        [0x00,0x00,0x1E,0x33,0x33,0x33,0x1E,0x00], // 'o'
        [0x00,0x00,0x3B,0x66,0x66,0x3E,0x06,0x0F], // 'p'
        [0x00,0x00,0x6E,0x33,0x33,0x3E,0x30,0x78], // 'q'
        [0x00,0x00,0x3B,0x6E,0x66,0x06,0x0F,0x00], // 'r'
        [0x00,0x00,0x3E,0x03,0x1E,0x30,0x1F,0x00], // 's'
        [0x08,0x0C,0x3E,0x0C,0x0C,0x2C,0x18,0x00], // 't'
        [0x00,0x00,0x33,0x33,0x33,0x33,0x6E,0x00], // 'u'
        [0x00,0x00,0x33,0x33,0x33,0x1E,0x0C,0x00], // 'v'
        [0x00,0x00,0x63,0x6B,0x7F,0x7F,0x36,0x00], // 'w'
        [0x00,0x00,0x63,0x36,0x1C,0x36,0x63,0x00], // 'x'
        [0x00,0x00,0x33,0x33,0x33,0x3E,0x30,0x1F], // 'y'
        [0x00,0x00,0x3F,0x19,0x0C,0x26,0x3F,0x00], // 'z'
        [0x38,0x0C,0x0C,0x07,0x0C,0x0C,0x38,0x00], // '{'
        [0x0C,0x0C,0x0C,0x00,0x0C,0x0C,0x0C,0x00], // '|'
        [0x07,0x0C,0x0C,0x38,0x0C,0x0C,0x07,0x00], // '}'
        [0x6E,0x3B,0x00,0x00,0x00,0x00,0x00,0x00], // '~'
    ];

    if c < ' ' || c > '~' {
        return;
    }
    let index = c as usize - 32;
    let bitmap = FONT[index];
    
    framebuffer.set_current_color(color);
    
    for (row, bits) in bitmap.iter().enumerate() {
        for col in 0..8 {
            if bits & (1 << col) != 0 {
                let px = x + col;
                let py = y + row;
                if px < framebuffer.width && py < framebuffer.height {
                    framebuffer.point(px, py, -2.0); // Depth negativo para estar al frente
                }
            }
        }
    }
}

fn main() {
    let window_width = 1400;
    let window_height = 900;
    let framebuffer_width = 1400;
    let framebuffer_height = 900;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Sistema Solar 3D - Proyecto Final",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(100, 50);
    window.update();

    framebuffer.set_background_color(0x000008);

    // Cargar modelos
    let sphere_obj = Obj::load("../assets/models/sphere.obj").expect("Failed to load sphere.obj");
    let sphere_vertices = sphere_obj.get_vertex_array();

    let spaceship_obj = Obj::load("../assets/models/spaceship.obj").expect("Failed to load spaceship.obj");
    let spaceship_vertices = spaceship_obj.get_vertex_array();

    // Crear skybox con 1500 estrellas
    let skybox = Skybox::new(1500, framebuffer_width, framebuffer_height);

    // Inicializar cámara
    let mut camera = Camera::new(Vec3::new(0.0, 300.0, 800.0));
    camera.update_vectors();

    let mut time = 0.0f32;
    let mut last_frame = Instant::now();

    // Centro del sistema solar
    let sun_center = Vec3::new(0.0, 0.0, 0.0);

    // Definir cuerpos celestes
    let mut bodies = vec![
        CelestialBody {
            orbital_radius: 0.0,
            orbital_angle: 0.0,
            orbital_speed: 0.0,
            scale: 150.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Sun,
            rotation_speed: 0.005,
            name: "Sol",
            color: 0xFFFF00,
        },
        CelestialBody {
            orbital_radius: 300.0,
            orbital_angle: 0.0,
            orbital_speed: 0.015,
            scale: 60.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::RockyPlanet,
            rotation_speed: 0.02,
            name: "Planeta Rocoso",
            color: 0x4488FF,
        },
        CelestialBody {
            orbital_radius: 500.0,
            orbital_angle: PI / 2.0,
            orbital_speed: 0.012,
            scale: 70.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::VolcanicPlanet,
            rotation_speed: 0.018,
            name: "Planeta Volcánico",
            color: 0xFF4400,
        },
        CelestialBody {
            orbital_radius: 750.0,
            orbital_angle: PI,
            orbital_speed: 0.008,
            scale: 120.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::GasGiant,
            rotation_speed: 0.015,
            name: "Gigante Gaseoso",
            color: 0xFFAA66,
        },
        CelestialBody {
            orbital_radius: 1000.0,
            orbital_angle: PI * 1.5,
            orbital_speed: 0.005,
            scale: 80.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::IcePlanet,
            rotation_speed: 0.01,
            name: "Planeta Helado",
            color: 0x88DDFF,
        },
        CelestialBody {
            orbital_radius: 400.0,
            orbital_angle: PI / 4.0,
            orbital_speed: 0.025,
            scale: 40.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: ShaderType::Moon,
            rotation_speed: 0.03,
            name: "Luna",
            color: 0xCCCCCC,
        },
    ];

    // Luna orbitando el planeta rocoso
    let mut moon_angle = 0.0f32;
    let moon_orbital_radius = 100.0;
    let moon_speed = 0.05;

    let mut paused = false;
    let mut show_orbits = true;
    let mut show_ui = true;

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          SISTEMA SOLAR 3D - PROYECTO FINAL                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("🌍 CUERPOS CELESTES:");
    println!("  0: Sol (centro)");
    println!("  1: Planeta Rocoso");
    println!("  2: Planeta Volcánico");
    println!("  3: Gigante Gaseoso");
    println!("  4: Planeta Helado");
    println!("  5: Luna");
    println!();
    println!("🎮 CONTROLES:");
    println!("  W/A/S/D     : Mover cámara");
    println!("  SPACE/SHIFT : Subir/Bajar");
    println!("  Q/E         : Rotar cámara (yaw)");
    println!("  Z/C         : Rotar cámara (pitch)");
    println!("  0           : Vista general");
    println!("  1           : Sol");
    println!("  2           : Planeta Rocoso");
    println!("  3           : Planeta Volcánico");
    println!("  4           : Gigante Gaseoso");
    println!("  5           : Planeta Helado");
    println!("  6           : Luna");
    println!("  O           : Toggle órbitas");
    println!("  P           : Pausar/Reanudar");
    println!("  H           : Toggle UI");
    println!("  ESC         : Salir");
    println!();

    while window.is_open() {
        let current_frame = Instant::now();
        let delta_time = current_frame.duration_since(last_frame).as_secs_f32();
        last_frame = current_frame;

        if window.is_key_down(Key::Escape) {
            break;
        }

        // Controles de cámara 3D
        if window.is_key_down(Key::W) {
            camera.move_forward(delta_time);
        }
        if window.is_key_down(Key::S) {
            camera.move_backward(delta_time);
        }
        if window.is_key_down(Key::A) {
            camera.move_left(delta_time);
        }
        if window.is_key_down(Key::D) {
            camera.move_right(delta_time);
        }
        if window.is_key_down(Key::Space) {
            camera.move_up(delta_time);
        }
        if window.is_key_down(Key::LeftShift) {
            camera.move_down(delta_time);
        }

        // Rotación de cámara
        if window.is_key_down(Key::Q) {
            camera.rotate(-50.0 * delta_time, 0.0);
        }
        if window.is_key_down(Key::E) {
            camera.rotate(50.0 * delta_time, 0.0);
        }
        if window.is_key_down(Key::Z) {
            camera.rotate(0.0, 30.0 * delta_time);
        }
        if window.is_key_down(Key::C) {
            camera.rotate(0.0, -30.0 * delta_time);
        }

        // Warping a planetas (teclas 1-6)
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
            camera.start_warp(Vec3::new(0.0, 300.0, 800.0));
            println!("🚀 Warping a: Vista general del sistema");
        }

        // Sol
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            let target = Vec3::new(0.0, 200.0, 400.0);
            camera.start_warp(target);
            println!("🚀 Warping a: {}", bodies[0].name);
        }

        // Planeta Rocoso
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) && bodies.len() > 1 {
            let planet_pos = calculate_planet_position(&bodies[1], sun_center);
            let target = Vec3::new(planet_pos.x + 200.0, planet_pos.y + 100.0, planet_pos.z + 200.0);
            camera.start_warp(target);
            println!("🚀 Warping a: {} en ({:.0}, {:.0}, {:.0})", bodies[1].name, planet_pos.x, planet_pos.y, planet_pos.z);
        }

        // Planeta Volcánico
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) && bodies.len() > 2 {
            let planet_pos = calculate_planet_position(&bodies[2], sun_center);
            let target = Vec3::new(planet_pos.x + 220.0, planet_pos.y + 100.0, planet_pos.z + 220.0);
            camera.start_warp(target);
            println!("🚀 Warping a: {} en ({:.0}, {:.0}, {:.0})", bodies[2].name, planet_pos.x, planet_pos.y, planet_pos.z);
        }

        // Gigante Gaseoso
        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) && bodies.len() > 3 {
            let planet_pos = calculate_planet_position(&bodies[3], sun_center);
            let target = Vec3::new(planet_pos.x + 350.0, planet_pos.y + 150.0, planet_pos.z + 350.0);
            camera.start_warp(target);
            println!("🚀 Warping a: {} en ({:.0}, {:.0}, {:.0})", bodies[3].name, planet_pos.x, planet_pos.y, planet_pos.z);
        }

        // Planeta Helado
        if window.is_key_pressed(Key::Key5, minifb::KeyRepeat::No) && bodies.len() > 4 {
            let planet_pos = calculate_planet_position(&bodies[4], sun_center);
            let target = Vec3::new(planet_pos.x + 250.0, planet_pos.y + 120.0, planet_pos.z + 250.0);
            camera.start_warp(target);
            println!("🚀 Warping a: {} en ({:.0}, {:.0}, {:.0})", bodies[4].name, planet_pos.x, planet_pos.y, planet_pos.z);
        }

        // Luna
        if window.is_key_pressed(Key::Key6, minifb::KeyRepeat::No) && bodies.len() > 1 {
            let rocky_pos = calculate_planet_position(&bodies[1], sun_center);
            let moon_pos = Vec3::new(
                rocky_pos.x + moon_orbital_radius * moon_angle.cos(),
                rocky_pos.y,
                rocky_pos.z + moon_orbital_radius * moon_angle.sin(),
            );
            let target = Vec3::new(moon_pos.x + 120.0, moon_pos.y + 50.0, moon_pos.z + 120.0);
            camera.start_warp(target);
            println!("🚀 Warping a: Luna en ({:.0}, {:.0}, {:.0})", moon_pos.x, moon_pos.y, moon_pos.z);
        }
        

        // Toggles
        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            show_orbits = !show_orbits;
            println!("Órbitas: {}", if show_orbits { "✓ Visible" } else { "✗ Oculto" });
        }
        if window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            paused = !paused;
            println!("{}", if paused { "⏸️  PAUSADO" } else { "▶️  REPRODUCIENDO" });
        }
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) {
            show_ui = !show_ui;
            println!("UI: {}", if show_ui { "✓ Visible" } else { "✗ Oculto" });
        }

        // Actualizar warp
        camera.update_warp(delta_time);

        // Clear
        framebuffer.clear();

        // Render skybox
        skybox.render_with_twinkle(&mut framebuffer, time);

        // Update time
        if !paused {
            time += delta_time;
            for body in &mut bodies {
                body.orbital_angle += body.orbital_speed * delta_time;
                body.rotation.y += body.rotation_speed * delta_time;
            }
        }

        // Render planetas
        for body in &bodies {
            let world_pos = calculate_planet_position(body, sun_center);
            let screen_pos = camera.get_screen_position(world_pos, window_width as f32, window_height as f32);

            // Solo renderizar si está delante de la cámara
            if screen_pos.z > 0.0 {
                let model_matrix = create_model_matrix(screen_pos, body.scale, body.rotation);
                let uniforms = Uniforms { model_matrix };
                render(&mut framebuffer, &uniforms, &sphere_vertices, body.shader_type, time);
            }

            // Detección de colisiones
            if camera.check_collision(world_pos, body.scale) {
                camera.resolve_collision(world_pos, body.scale);
            }
        }

        // Render luna orbitando el planeta rocoso
        if bodies.len() > 1 {
            let rocky_planet = &bodies[1];
            let rocky_pos = calculate_planet_position(rocky_planet, sun_center);
            
            // Actualizar ángulo de la luna
            if !paused {
                moon_angle += moon_speed * delta_time;
            }
            
            // Calcular posición de la luna
            let moon_world_pos = Vec3::new(
                rocky_pos.x + moon_orbital_radius * moon_angle.cos(),
                rocky_pos.y,
                rocky_pos.z + moon_orbital_radius * moon_angle.sin(),
            );
            
            let moon_screen = camera.get_screen_position(moon_world_pos, window_width as f32, window_height as f32);
            
            if moon_screen.z > 0.0 {
                let moon_matrix = create_model_matrix(
                    moon_screen,
                    25.0,  // Tamaño de la luna
                    Vec3::new(0.0, moon_angle * 2.0, 0.0),
                );
                let moon_uniforms = Uniforms { model_matrix: moon_matrix };
                render(&mut framebuffer, &moon_uniforms, &sphere_vertices, ShaderType::Moon, time);
            }
            
            // Órbita de la luna (opcional)
            if show_orbits {
                draw_orbit(
                    &mut framebuffer,
                    rocky_pos,
                    moon_orbital_radius,
                    0xCCCCCC,
                    &camera,
                    window_width as f32,
                    window_height as f32,
                );
            }
        }

        // Render órbitas
        if show_orbits {
            for body in &bodies {
                if body.orbital_radius > 0.0 {
                    draw_orbit(
                        &mut framebuffer,
                        sun_center,
                        body.orbital_radius,
                        body.color,
                        &camera,
                        window_width as f32,
                        window_height as f32,
                    );
                }
            }
        }


        // Render nave espacial siguiendo la cámara
        let spaceship_offset = camera.target * 100.0 + Vec3::new(30.0, -20.0, 0.0);
        let spaceship_pos_world = camera.position + spaceship_offset;
        let spaceship_screen = camera.get_screen_position(spaceship_pos_world, window_width as f32, window_height as f32);

        if spaceship_screen.z > 0.0 {
            let spaceship_rotation = Vec3::new(0.0, camera.yaw.to_radians() + PI / 2.0, 0.0);
            let spaceship_matrix = create_model_matrix(spaceship_screen, 15.0, spaceship_rotation);
            let spaceship_uniforms = Uniforms { model_matrix: spaceship_matrix };
            render(&mut framebuffer, &spaceship_uniforms, &spaceship_vertices, ShaderType::Spaceship, time);
        }

        // UI simple
        if show_ui && !camera.is_warping {
            let ui_color = 0xFFFFFF;
            // Información de cámara
            draw_text(&mut framebuffer, 20, 20, &format!("Pos: ({:.0}, {:.0}, {:.0})", 
                camera.position.x, camera.position.y, camera.position.z), ui_color);
            
            // Controles
            if camera.is_warping {
                draw_text(&mut framebuffer, 20, 40, ">>> WARPING <<<", 0xFFFF00);
                let progress_percent = (camera.warp_progress * 100.0) as i32;
                draw_text(&mut framebuffer, 20, 55, &format!("Progress: {}%", progress_percent), 0xFFFF00);
            } else {
                draw_text(&mut framebuffer, 20, 40, "WASD: Move | Space/Shift: Up/Down", ui_color);
                draw_text(&mut framebuffer, 20, 55, "QE: Rotate | 0-6: Warp | O: Orbits | P: Pause | H: UI", ui_color);
            }
            
            draw_text(&mut framebuffer, 20, 55, "QE: Rotate | 0-6: Warp | O: Orbits | P: Pause | H: UI", ui_color);
            
            // FPS Counter (opcional)
            let fps = (1.0 / delta_time) as i32;
            draw_text(&mut framebuffer, 20, 75, &format!("FPS: {}", fps), ui_color);
            
            // Status
            if paused {
                draw_text(&mut framebuffer, framebuffer_width - 150, 20, "PAUSED", 0xFF0000);
            }
            
            if show_orbits {
                draw_text(&mut framebuffer, framebuffer_width - 150, 40, "Orbits: ON", 0x00FF00);
            }
        }

        // Controles de cámara 3D (SOLO si no está en warp)
        if !camera.is_warping {
            if window.is_key_down(Key::W) {
                camera.move_forward(delta_time);
            }
            if window.is_key_down(Key::S) {
                camera.move_backward(delta_time);
            }
            if window.is_key_down(Key::A) {
                camera.move_left(delta_time);
            }
            if window.is_key_down(Key::D) {
                camera.move_right(delta_time);
            }
            if window.is_key_down(Key::Space) {
                camera.move_up(delta_time);
            }
            if window.is_key_down(Key::LeftShift) {
                camera.move_down(delta_time);
            }

            // Rotación de cámara
            if window.is_key_down(Key::Q) {
                camera.rotate(-50.0 * delta_time, 0.0);
            }
            if window.is_key_down(Key::E) {
                camera.rotate(50.0 * delta_time, 0.0);
            }
            if window.is_key_down(Key::Z) {
                camera.rotate(0.0, 30.0 * delta_time);
            }
            if window.is_key_down(Key::C) {
                camera.rotate(0.0, -30.0 * delta_time);
            }
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
        println!("\n¡Gracias por explorar el sistema solar! 🚀");
}


fn calculate_planet_position(body: &CelestialBody, center: Vec3) -> Vec3 {
    Vec3::new(
        center.x + body.orbital_radius * body.orbital_angle.cos(),
        center.y,
        center.z + body.orbital_radius * body.orbital_angle.sin(),
    )
}