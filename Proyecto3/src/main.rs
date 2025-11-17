use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions, ScaleMode};
use std::time::Duration;
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

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::{triangle, ShaderType};
use shaders::vertex_shader;
use crate::line::line;

pub struct Uniforms {
    model_matrix: Mat4,
}

struct CelestialBody {
    orbital_radius: f32,
    orbital_angle: f32,
    orbital_speed: f32,
    scale: f32,
    visible_radius: f32, 
    rotation: Vec3,
    shader_type: ShaderType,
    rotation_speed: f32,
    name: &'static str,
}


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
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
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

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], shader_type, time));
    }

    // Fragment Processing Stage
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
            framebuffer.point(x as usize, y as usize, 0.0);
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

fn draw_ring(framebuffer: &mut Framebuffer, center: Vec3, inner_radius: f32, outer_radius: f32, color: u32) {
    let steps = 100;
    for i in 0..steps {
        let theta1 = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let theta2 = ((i + 1) as f32 / steps as f32) * std::f32::consts::TAU;

        let x1_inner = center.x + inner_radius * theta1.cos();
        let y1_inner = center.y + inner_radius * theta1.sin();
        let x2_inner = center.x + inner_radius * theta2.cos();
        let y2_inner = center.y + inner_radius * theta2.sin();

        let x1_outer = center.x + outer_radius * theta1.cos();
        let y1_outer = center.y + outer_radius * theta1.sin();
        let x2_outer = center.x + outer_radius * theta2.cos();
        let y2_outer = center.y + outer_radius * theta2.sin();

        draw_line(framebuffer, x1_inner as i32, y1_inner as i32, x2_inner as i32, y2_inner as i32, color);
        draw_line(framebuffer, x1_outer as i32, y1_outer as i32, x2_outer as i32, y2_outer as i32, color);
    }
}


/// Dibuja texto simple en pantalla (blanco)
fn draw_text(framebuffer: &mut Framebuffer, x: usize, y: usize, text: &str, color: u32) {
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            continue;
        }
        let cx = x + i * 8;
        draw_char(framebuffer, cx, y, b as char, color);
    }
}

/// Dibuja un solo carácter ASCII de 8x8
fn draw_char(framebuffer: &mut Framebuffer, x: usize, y: usize, c: char, color: u32) {
    // Fuente monoespaciada básica de 8x8 (95 caracteres imprimibles)
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
    for (row, bits) in bitmap.iter().enumerate() {
        for col in 0..8 {
            if bits & (1 << col) != 0 {
                let px = x + col;
                let py = y + row;
                if px < framebuffer.width && py < framebuffer.height {
                    framebuffer.set_current_color(color);
                    framebuffer.point(px, py, 0.0);
                }
            }
        }
    }
}

fn main() {
    let window_width = 1200;
    let window_height = 800;
    let framebuffer_width = 1200;
    let framebuffer_height = 800;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Sistema Solar - Órbitas Planetarias",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(150, 50);
    window.update();

    framebuffer.set_background_color(0x000008); // Negro espacio profundo

    // Carga el modelo de esfera
    let obj = Obj::load("../assets/models/sphere.obj").expect("Failed to load sphere.obj");
    let vertex_arrays = obj.get_vertex_array();

    let mut time = 0.0f32;

    // Centro del sistema (donde está el sol)
    let sun_center_x = 600.0;
    let sun_center_y = 450.0;

    // Configuración de la luna del planeta rocoso
    let mut moon_angle: f32 = 0.0;
    let moon_orbital_radius = 100.0; // distancia desde el planeta
    let moon_speed = 0.05;
    let moon_scale = 40.0;

    // Define los cuerpos celestes con órbitas
let mut bodies = vec![
    CelestialBody {
        orbital_radius: 250.0,
        orbital_angle: 0.0,
        orbital_speed: 0.02,
        scale: 80.0,
        visible_radius: 80.0 / 2.0, // planeta rocoso sin anillos
        rotation: Vec3::new(0.0, 0.0, 0.0),
        shader_type: ShaderType::RockyPlanet,
        rotation_speed: 0.015,
        name: "Planeta Rocoso",
    },
    CelestialBody {
        orbital_radius: 480.0,
        orbital_angle: std::f32::consts::PI,
        orbital_speed: 0.01,
        scale: 130.0,
        visible_radius: 130.0 / 2.0 + 80.0, // 👈 planeta gaseoso + anillos
        rotation: Vec3::new(0.0, 0.0, 0.0),
        shader_type: ShaderType::GasGiant,
        rotation_speed: 0.012,
        name: "Gigante Gaseoso",
    },
    CelestialBody {
        orbital_radius: 620.0,
        orbital_angle: std::f32::consts::PI * 1.5,
        orbital_speed: 0.007,
        scale: 100.0,
        visible_radius: 100.0 / 2.0,
        rotation: Vec3::new(0.0, 0.0, 0.0),
        shader_type: ShaderType::IcePlanet,
        rotation_speed: 0.01,
        name: "Planeta Helado",
    },
];

    // Camera control
    let mut camera_zoom = 1.0f32;
    let mut camera_x = 0.0f32;
    let mut camera_y = 0.0f32;
    let camera_speed = 10.0;

    let mut paused = false;
    let mut show_orbits = true;

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║        SISTEMA SOLAR - ÓRBITAS PLANETARIAS            ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!();
    println!("☀️  SOL (Centro) con {} planetas en órbita", bodies.len());
    println!();
    println!("🪐 PLANETAS (desde el más cercano):");
    println!("  1. 🌙 Luna - Órbita: 180 (muy rápida)");
    println!("  2. 🌍 Planeta Rocoso - Órbita: 250");
    println!("  3. 🌋 Planeta Volcánico - Órbita: 350");
    println!("  4. 🪐 Gigante Gaseoso - Órbita: 480");
    println!("  5. ❄️  Planeta Helado - Órbita: 620 (muy lenta)");
    println!();
    println!("🎮 CONTROLES:");
    println!("  ↑↓←→ : Mover cámara");
    println!("  A/S : Zoom out/in");
    println!("  ESPACIO : Pausar/Reanudar órbitas");
    println!("  O : Mostrar/Ocultar órbitas (WIP)");
    println!("  R : Reset cámara al centro");
    println!("  ESC : Salir");
    println!();
    println!("💡 TIP: ¡Observa cómo los planetas internos orbitan más rápido!");
    println!();

    // Evitar colisiones ajustando las distancias orbitales
for i in 1..bodies.len() {
    let prev = &bodies[i - 1];
    let min_distance = prev.orbital_radius + prev.visible_radius + bodies[i].visible_radius + 40.0; // 40px de margen
    if bodies[i].orbital_radius < min_distance {
        bodies[i].orbital_radius = min_distance;
    }
}

let moon_orbital_radius = 100.0;
if moon_orbital_radius < bodies[0].visible_radius + moon_scale {
    println!("⚠️ Ajustando órbita lunar para evitar colisión con el planeta rocoso");
}

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Toggle pause
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            paused = !paused;
            println!("{}", if paused { "⏸️  Sistema PAUSADO" } else { "▶️  Sistema en MOVIMIENTO" });
        }

        // Toggle orbits
        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            show_orbits = !show_orbits;
            println!("Órbitas: {}", if show_orbits { "Visible" } else { "Oculto" });
        }

        // Reset camera
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            camera_x = 0.0;
            camera_y = 0.0;
            camera_zoom = 1.0;
            println!("📷 Cámara reseteada");
        }

        // Camera movement
        if window.is_key_down(Key::Right) {
            camera_x -= camera_speed;
        }
        if window.is_key_down(Key::Left) {
            camera_x += camera_speed;
        }
        if window.is_key_down(Key::Up) {
            camera_y += camera_speed;
        }
        if window.is_key_down(Key::Down) {
            camera_y -= camera_speed;
        }

        // Zoom
        if window.is_key_down(Key::S) {
            camera_zoom += 0.01;
            if camera_zoom > 2.0 {
                camera_zoom = 2.0;
            }
        }
        if window.is_key_down(Key::A) {
            camera_zoom -= 0.01;
            if camera_zoom < 0.3 {
                camera_zoom = 0.3;
            }
        }

        framebuffer.clear();

        // Update time
        if !paused {
            time += 0.016;
            
            // Update orbital positions and rotations
            for body in &mut bodies {
                body.orbital_angle += body.orbital_speed;
                body.rotation.y += body.rotation_speed;
            }
        }

        // Calculate sun position with camera
        let sun_screen_x = sun_center_x * camera_zoom + camera_x;
        let sun_screen_y = sun_center_y * camera_zoom + camera_y;

        // Render the SUN first (always at center)
        let sun_position = Vec3::new(sun_screen_x, sun_screen_y, 0.0);
        let sun_matrix = create_model_matrix(
            sun_position,
            140.0 * camera_zoom,
            Vec3::new(0.0, time * 0.005, 0.0)
        );
        let sun_uniforms = Uniforms { model_matrix: sun_matrix };
        render(&mut framebuffer, &sun_uniforms, &vertex_arrays, ShaderType::Sun, time);

        // Render all planets in their orbits
        for body in &bodies {
            // Calculate orbital position using polar coordinates
            let orbit_x = sun_center_x + body.orbital_radius * body.orbital_angle.cos();
            let orbit_y = sun_center_y + body.orbital_radius * body.orbital_angle.sin();
            
            // Apply camera transformations
            let screen_position = Vec3::new(
                orbit_x * camera_zoom + camera_x,
                orbit_y * camera_zoom + camera_y,
                0.0
            );

            // Only render if visible on screen (with generous margin)
            if screen_position.x > -300.0 && screen_position.x < window_width as f32 + 300.0 &&
               screen_position.y > -300.0 && screen_position.y < window_height as f32 + 300.0 {
                
                let model_matrix = create_model_matrix(
                    screen_position,
                    body.scale * camera_zoom,
                    body.rotation
                );
                let uniforms = Uniforms { model_matrix };

                render(&mut framebuffer, &uniforms, &vertex_arrays, body.shader_type, time);
            }
        }

        // Mostrar instrucciones en esquina inferior izquierda
        let instructions_y = framebuffer_height - 90;
        let color_text = 0xFFFFFF;

        draw_text(
            &mut framebuffer,
            20,
            instructions_y,
            "Presiona ESPACIO para pausar cuando los planetas estén en buenas posiciones",
            color_text,
        );
        draw_text(
            &mut framebuffer,
            20,
            instructions_y + 15,
            "* Usa A/S para hacer zoom y capturar detalles",
            color_text,
        );
        draw_text(
            &mut framebuffer,
            20,
            instructions_y + 30,
            "* Usa flechas para centrarte en cada planeta",
            color_text,
        );
        draw_text(
            &mut framebuffer,
            20,
            instructions_y + 45,
            "* Presiona R para volver al centro y ver todo el sistema",
            color_text,
        );

        // Dibuja los anillos del planeta gaseoso
let gas_giant = &bodies[2]; // el gigante gaseoso en tu lista
let gas_x = sun_center_x + gas_giant.orbital_radius * gas_giant.orbital_angle.cos();
let gas_y = sun_center_y + gas_giant.orbital_radius * gas_giant.orbital_angle.sin();

let gas_screen = Vec3::new(
    gas_x * camera_zoom + camera_x,
    gas_y * camera_zoom + camera_y,
    0.0,
);


draw_ring(
    &mut framebuffer,
    gas_screen,
    160.0 * camera_zoom, // radio interno
    220.0 * camera_zoom, // radio externo
    0xAAAAAA,            // color gris claro
);

// Dibuja la luna orbitando el planeta rocoso
let rocky_planet = &bodies[0]; // el primero en tu lista
let rocky_x = sun_center_x + rocky_planet.orbital_radius * rocky_planet.orbital_angle.cos();
let rocky_y = sun_center_y + rocky_planet.orbital_radius * rocky_planet.orbital_angle.sin();

if !paused {
    moon_angle += moon_speed;
}

let moon_x = rocky_x + moon_orbital_radius * moon_angle.cos();
let moon_y = rocky_y + moon_orbital_radius * moon_angle.sin();

let moon_screen = Vec3::new(
    moon_x * camera_zoom + camera_x,
    moon_y * camera_zoom + camera_y,
    0.0,
);

let moon_matrix = create_model_matrix(
    moon_screen,
    moon_scale * camera_zoom,
    Vec3::new(0.0, moon_angle * 2.0, 0.0),
);
let moon_uniforms = Uniforms { model_matrix: moon_matrix };
render(&mut framebuffer, &moon_uniforms, &vertex_arrays, ShaderType::Moon, time);


        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}