use nalgebra_glm::{Vec3, dot};
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;
use crate::shader;

#[derive(Clone, Copy, PartialEq)]
pub enum ShaderType {
    Sun,
    RockyPlanet,
    GasGiant,
    IcePlanet,
    VolcanicPlanet,
    Moon,
    Spaceship,
}

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex, shader_type: ShaderType, time: f32) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let (a, b, c) = (v1.transformed_position, v2.transformed_position, v3.transformed_position);
    
    let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a, &b, &c);
    
    let light_dir = Vec3::new(0.0, 0.0, -1.0);
    
    let triangle_area = edge_function(&a, &b, &c);
    
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
            
            let (w1, w2, w3) = barycentric_coordinates(&point, &a, &b, &c, triangle_area);
            
            if w1 >= 0.0 && w1 <= 1.0 &&
               w2 >= 0.0 && w2 <= 1.0 &&
               w3 >= 0.0 && w3 <= 1.0 {
                
                let normal = v1.transformed_normal * w1 + v2.transformed_normal * w2 + v3.transformed_normal * w3;
                let normal = normal.normalize();
                
                let world_pos = v1.position * w1 + v2.position * w2 + v3.position * w3;
                
                let intensity = dot(&normal, &light_dir).max(0.0);
                
                let base_color = match shader_type {
                    ShaderType::Sun => {
                        shader::sun_shader(&world_pos, time)
                    },
                    ShaderType::RockyPlanet => {
                        let color = shader::rocky_planet_shader(&world_pos, time);
                        color * intensity.max(0.2)
                    },
                    ShaderType::GasGiant => {
                        let color = shader::gas_giant_shader(&world_pos, time);
                        color * intensity.max(0.2)
                    },
                    ShaderType::IcePlanet => {
                        let color = shader::ice_planet_shader(&world_pos, time);
                        color * intensity.max(0.3)
                    },
                    ShaderType::VolcanicPlanet => {
                        let color = shader::volcanic_planet_shader(&world_pos, time);
                        color * intensity.max(0.4)
                    },
                    ShaderType::Moon => {
                        let color = shader::moon_shader(&world_pos);
                        color * intensity.max(0.15)
                    },
                    ShaderType::Spaceship => {
                        spaceship_shader(&world_pos, &normal, &light_dir, intensity)
                    },
                };
                
                let depth = a.z * w1 + b.z * w2 + c.z * w3;
                
                fragments.push(Fragment::new(x as f32, y as f32, base_color, depth));
            }
        }
    }
    
    fragments
}

// Shader para la nave espacial
fn spaceship_shader(position: &Vec3, normal: &Vec3, _light_dir: &Vec3, intensity: f32) -> Color {
    // Color base dorado metálico
    let base_color = Color::new(200, 170, 50);
    
    // Paneles y detalles
    let panel_variation = ((position.x * 5.0).sin() * (position.y * 5.0).cos()).abs();
    let panel_color = if panel_variation > 0.7 {
        Color::new(220, 190, 70)
    } else {
        base_color
    };
    
    // Iluminación más brillante para que destaque
    let lit_color = panel_color * intensity.max(0.4);
    
    lit_color
}

fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;
    
    (min_x, min_y, max_x, max_y)
}

fn barycentric_coordinates(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, area: f32) -> (f32, f32, f32) {
    let w1 = edge_function(b, c, p) / area;
    let w2 = edge_function(c, a, p) / area;
    let w3 = edge_function(a, b, p) / area;
    
    (w1, w2, w3)
}

fn edge_function(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}