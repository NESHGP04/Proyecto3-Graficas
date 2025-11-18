use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::color::Color;
use std::f32::consts::PI;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;

// ============= FUNCIONES DE RUIDO PROCEDURAL =============

fn random(x: f32, y: f32) -> f32 {
    let a = x * 12.9898 + y * 78.233;
    let b = a.sin() * 43758.5453;
    b - b.floor()
}

fn noise(x: f32, y: f32) -> f32 {
    let i_x = x.floor();
    let i_y = y.floor();
    let f_x = x - i_x;
    let f_y = y - i_y;

    let a = random(i_x, i_y);
    let b = random(i_x + 1.0, i_y);
    let c = random(i_x, i_y + 1.0);
    let d = random(i_x + 1.0, i_y + 1.0);

    let u = f_x * f_x * (3.0 - 2.0 * f_x);
    let v = f_y * f_y * (3.0 - 2.0 * f_y);

    a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v
}

fn fbm(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;

    for _ in 0..octaves {
        value += amplitude * noise(x * frequency, y * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value
}

// ============= SHADER 1: SOL (ESTRELLA) =============
// Capas: gradiente radial, manchas solares, llamaradas, corona
pub fn sun_shader(position: &Vec3, time: f32) -> Color {
    // Distancia desde el centro
    let distance = (position.x * position.x + position.y * position.y + position.z * position.z).sqrt();
    
    // Capa 1: Gradiente radial (centro amarillo brillante, borde naranja-rojo)
    let radial_gradient = 1.0 - (distance * 0.8).min(1.0);
    
    // Capa 2: Manchas solares (usando ruido)
    let sunspot_noise = fbm(position.x * 3.0 + time * 0.5, position.y * 3.0, 4);
    let sunspots = if sunspot_noise > 0.6 { 0.7 } else { 1.0 };
    
    // Capa 3: Llamaradas (animadas)
    let flare_noise = fbm(
        position.x * 5.0 + time * 2.0,
        position.y * 5.0 + (time * 1.5).sin() * 0.5,
        3
    );
    let flares = (flare_noise * 0.3 + 0.7).clamp(0.5, 1.0);
    
    // Capa 4: Corona (brillo en los bordes)
    let corona = (1.0_f32 - distance).powf(0.3) * 0.5;
    
    // Mezcla de colores base
    let yellow = Color::new(255, 220, 100);
    let orange = Color::new(255, 150, 50);
    let red = Color::new(255, 80, 30);
    let white = Color::new(255, 255, 255);
    
    // Interpola entre colores según la distancia
    let base_color = if radial_gradient > 0.7 {
        // Centro: amarillo-blanco
        yellow * 0.7 + white * 0.3
    } else if radial_gradient > 0.4 {
        // Medio: amarillo-naranja
        yellow * (radial_gradient - 0.4) * 3.33 + orange * (1.0 - (radial_gradient - 0.4) * 3.33)
    } else {
        // Borde: naranja-rojo
        orange * (radial_gradient * 2.5) + red * (1.0 - radial_gradient * 2.5)
    };
    
    // Aplica todas las capas
    let final_color = base_color * sunspots * flares;
    
    // Añade corona brillante
    final_color + white * corona
}

// ============= SHADER 2: PLANETA ROCOSO (TIPO TIERRA/MARTE) =============
// Capas: continentes, océanos, nubes, casquetes polares
pub fn rocky_planet_shader(position: &Vec3, time: f32) -> Color {
    // Usa coordenadas esféricas para mapeo consistente
    let theta = position.y.atan2(position.x) + time * 0.1; // Rotación lenta
    let phi = (position.z / (position.x * position.x + position.y * position.y + position.z * position.z).sqrt()).acos();
    
    // Capa 1: Continentes vs Océanos
    let land_noise = fbm(theta * 3.0, phi * 3.0, 5);
    let is_land = land_noise > 0.5;
    
    // Capa 2: Variación de elevación en continentes
    let elevation = fbm(theta * 10.0, phi * 10.0, 3);
    
    // Capa 3: Nubes
    let cloud_noise = fbm(theta * 5.0 - time * 0.5, phi * 5.0, 3);
    let clouds = if cloud_noise > 0.6 { 0.3 } else { 0.0 };
    
    // Capa 4: Casquetes polares
    let pole_factor = (phi / PI).abs();
    let is_pole = pole_factor < 0.15 || pole_factor > 0.85;
    
    // Colores base
    let ocean = Color::new(20, 80, 180);       // Azul océano
    let shallow = Color::new(40, 120, 200);     // Azul claro
    let sand = Color::new(220, 200, 150);       // Arena
    let grass = Color::new(60, 140, 60);        // Verde
    let forest = Color::new(30, 100, 40);       // Verde oscuro
    let mountain = Color::new(120, 120, 120);   // Gris montañas
    let snow = Color::new(240, 250, 255);       // Blanco nieve
    let white_cloud = Color::new(255, 255, 255);
    
    // Determina el color base
    let base_color = if is_pole {
        snow
    } else if is_land {
        // Tierra con variación de elevación
        if elevation > 0.7 {
            mountain
        } else if elevation > 0.55 {
            forest
        } else if elevation > 0.45 {
            grass
        } else {
            sand
        }
    } else {
        // Océanos con profundidad
        if land_noise > 0.45 {
            shallow
        } else {
            ocean
        }
    };
    
    // Aplica nubes
    let final_color = base_color * (1.0 - clouds) + white_cloud * clouds;
    
    final_color
}

// ============= SHADER 3: GIGANTE GASEOSO (TIPO JÚPITER) =============
// Capas: bandas horizontales, tormenta, turbulencia, variación de color
pub fn gas_giant_shader(position: &Vec3, time: f32) -> Color {
    // Usa latitud para bandas horizontales
    let latitude = position.y + time * 0.05; // Rotación lenta
    
    // Capa 1: Bandas principales
    let band = (latitude * 8.0).sin() * 0.5 + 0.5;
    
    // Capa 2: Turbulencia en las bandas
    let turbulence = fbm(
        position.x * 10.0 + time * 0.3,
        latitude * 5.0,
        4
    );
    
    // Capa 3: Gran Mancha Roja (tormenta)
    let spot_x = position.x - 0.3;
    let spot_y = position.y + 0.1;
    let spot_distance = (spot_x * spot_x + spot_y * spot_y).sqrt();
    let storm = if spot_distance < 0.4 {
        let storm_noise = fbm(position.x * 20.0 + time, position.y * 20.0, 2);
        (1.0 - spot_distance / 0.4) * storm_noise
    } else {
        0.0
    };
    
    // Capa 4: Variación de intensidad en bandas
    let intensity_variation = fbm(position.x * 15.0, latitude * 8.0, 2);
    
    // Colores de las bandas
    let light_band = Color::new(220, 200, 170);  // Crema claro
    let dark_band = Color::new(180, 130, 90);    // Marrón
    let orange_band = Color::new(200, 150, 100); // Naranja
    let red_storm = Color::new(200, 80, 60);     // Rojo tormenta
    
    // Mezcla bandas claras y oscuras
    let band_color = if band > 0.5 {
        light_band * (band * 1.5).min(1.0) + orange_band * (1.0 - band)
    } else {
        dark_band * ((1.0_f32 - band) * 1.5).min(1.0) + orange_band * band
    };
    
    // Aplica turbulencia
    let turbulent_color = band_color * (0.8 + turbulence * 0.4);
    
    // Aplica tormenta
    let final_color = turbulent_color * (1.0 - storm) + red_storm * storm;
    
    // Aplica variación de intensidad
    final_color * (0.7 + intensity_variation * 0.3)
}

// ============= SHADER 4: PLANETA HELADO (BONUS - TIPO URANO/NEPTUNO) =============
pub fn ice_planet_shader(position: &Vec3, time: f32) -> Color {
    let theta = position.y.atan2(position.x) + time * 0.15;
    let phi = (position.z / (position.x * position.x + position.y * position.y + position.z * position.z).sqrt()).acos();
    
    // Capa de hielo con grietas
    let ice_noise = fbm(theta * 8.0, phi * 8.0, 4);
    let cracks = fbm(theta * 20.0, phi * 20.0, 2);
    
    // Colores
    let ice_blue = Color::new(180, 220, 255);
    let deep_blue = Color::new(100, 150, 220);
    let white = Color::new(230, 240, 255);
    
    let base = if ice_noise > 0.6 {
        white
    } else if ice_noise > 0.4 {
        ice_blue
    } else {
        deep_blue
    };
    
    // Añade grietas oscuras
    let final_color = if cracks > 0.7 {
        base * 0.7
    } else {
        base
    };
    
    final_color
}

// ============= SHADER 5: PLANETA VOLCÁNICO (BONUS - TIPO IO) =============
pub fn volcanic_planet_shader(position: &Vec3, time: f32) -> Color {
    let theta = position.y.atan2(position.x);
    let phi = (position.z / (position.x * position.x + position.y * position.y + position.z * position.z).sqrt()).acos();
    
    // Lava activa (animada)
    let lava_flow = fbm(theta * 5.0 + time * 2.0, phi * 5.0 + time, 3);
    
    // Roca volcánica
    let rock_texture = fbm(theta * 10.0, phi * 10.0, 4);
    
    // Colores
    let black_rock = Color::new(40, 30, 30);
    let gray_rock = Color::new(80, 70, 70);
    let lava_orange = Color::new(255, 120, 30);
    let lava_yellow = Color::new(255, 200, 50);
    
    let rock_color = if rock_texture > 0.5 {
        gray_rock
    } else {
        black_rock
    };
    
    // Lava activa
    let lava_intensity = (lava_flow * 0.5 + 0.5).clamp(0.0, 1.0);
    let is_lava = lava_flow > 0.6;
    
    if is_lava {
        let lava_color = lava_orange * (1.0 - lava_intensity) + lava_yellow * lava_intensity;
        lava_color * (0.8 + (time * 5.0).sin() * 0.2)
    } else {
        rock_color
    }
}

// ============= SHADER 6: ANILLOS (PARA GIGANTES GASEOSOS) =============
pub fn ring_shader(position: &Vec3, distance_from_center: f32) -> Color {
    // Los anillos son un plano alrededor del planeta
    // distance_from_center es la distancia radial en el plano XZ
    
    // Múltiples anillos con gaps
    let ring_pattern = (distance_from_center * 20.0).sin();
    let ring_noise = fbm(distance_from_center * 30.0, position.y * 50.0, 3);
    
    // Colores de anillos
    let light_ring = Color::new(200, 180, 160);
    let dark_ring = Color::new(120, 110, 100);
    let gap = Color::new(0, 0, 0); // Transparente (negro)
    
    // Determina si es gap o anillo
    if ring_pattern > 0.8 {
        gap
    } else if ring_noise > 0.6 {
        light_ring * 0.8
    } else {
        dark_ring * 0.6
    }
}

// ============= SHADER 7: LUNA (SIMPLE - TIPO LUNA TERRESTRE) =============
pub fn moon_shader(position: &Vec3) -> Color {
    let theta = position.y.atan2(position.x);
    let phi = (position.z / (position.x * position.x + position.y * position.y + position.z * position.z).sqrt()).acos();
    
    // Cráteres
    let crater_noise = fbm(theta * 15.0, phi * 15.0, 4);
    
    // Colores
    let light_gray = Color::new(200, 200, 200);
    let dark_gray = Color::new(120, 120, 120);
    let crater = Color::new(80, 80, 80);
    
    if crater_noise > 0.7 {
        crater
    } else if crater_noise > 0.4 {
        light_gray
    } else {
        dark_gray
    }
}

// ============= SHADER NAVE ESPACIAL =============
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Transform position
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    let transformed = uniforms.model_matrix * position;

    // Perform perspective division
    let w = transformed.w;
    let transformed_position = Vec3::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w
    );

    // Transform normal
    let model_mat3 = Mat3::new(
        uniforms.model_matrix[0], uniforms.model_matrix[1], uniforms.model_matrix[2],
        uniforms.model_matrix[4], uniforms.model_matrix[5], uniforms.model_matrix[6],
        uniforms.model_matrix[8], uniforms.model_matrix[9], uniforms.model_matrix[10]
    );
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
    
    let transformed_normal = normal_matrix * vertex.normal;

    // Create a new Vertex with transformed attributes
    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position,
        transformed_normal,
    }
}

pub fn fragment_shader(fragment: &Fragment, light_dir: &Vec3, normal: &Vec3) -> Color {
    // Iluminación difusa
    let normal = normal.normalize();
    let intensity = nalgebra_glm::dot(&normal, light_dir).max(0.0);
    
    // Aplica la iluminación al color del fragmento
    fragment.color * intensity
}

pub fn color_from_position(position: &Vec3) -> Color {
    // Gradiente basado en altura (eje Y)
    let t = (position.y + 1.0) * 0.5; // Normaliza entre 0 y 1
    
    // Gradiente de azul oscuro a azul claro
    let r = (50.0 + t * 100.0) as u8;
    let g = (100.0 + t * 100.0) as u8;
    let b = (200.0 + t * 55.0) as u8;
    
    Color::new(r, g, b)
}

// Colores metálicos con variación
pub fn metallic_shader(position: &Vec3, normal: &Vec3, light_dir: &Vec3) -> Color {
    let normal = normal.normalize();
    let intensity = nalgebra_glm::dot(&normal, light_dir).max(0.0);
    
    // Color base metálico (plateado/gris)
    let base = Color::new(180, 190, 200);
    
    // Componente especular (brillo)
    let view_dir = Vec3::new(0.0, 0.0, 1.0);
    let reflect_dir = 2.0 * nalgebra_glm::dot(&normal, light_dir) * normal - light_dir;
    let specular = nalgebra_glm::dot(&reflect_dir, &view_dir).max(0.0).powf(32.0);
    
    // Combina difuso + especular
    let diffuse = base * intensity;
    let specular_color = Color::new(255, 255, 255) * specular * 0.5;
    
    diffuse + specular_color
}

// Shader de nave espacial (azul metálico con detalles)
pub fn spaceship_shader(position: &Vec3, normal: &Vec3, light_dir: &Vec3) -> Color {
    let normal = normal.normalize();
    let intensity = nalgebra_glm::dot(&normal, light_dir).max(0.0);
    
    // Color base: azul metálico
    let base_color = Color::new(40, 80, 150);
    
    // Añade variación basada en la posición para simular paneles
    let panel_variation = ((position.x * 5.0).sin() * (position.y * 5.0).cos()).abs();
    let panel_color = if panel_variation > 0.7 {
        Color::new(60, 100, 170) // Paneles más claros
    } else {
        base_color
    };
    
    // Aplica iluminación
    let lit_color = panel_color * intensity;
    
    // Añade brillo especular en los bordes
    let rim_light = (1.0 - intensity).powf(3.0) * 0.3;
    let rim_color = Color::new(100, 150, 255) * rim_light;
    
    lit_color + rim_color
}

// Shader de nave de guerra (rojo/naranja)
pub fn warship_shader(position: &Vec3, normal: &Vec3, light_dir: &Vec3) -> Color {
    let normal = normal.normalize();
    let intensity = nalgebra_glm::dot(&normal, light_dir).max(0.0);
    
    // Color base: rojo oscuro metálico
    let base_color = Color::new(150, 30, 30);
    
    // Detalles naranjas en ciertas partes
    let detail = ((position.z * 3.0).sin() * 0.5 + 0.5).abs();
    let color = if detail > 0.8 {
        Color::new(200, 80, 20) // Detalles naranjas
    } else {
        base_color
    };
    
    color * intensity
}

// Shader futurista (cyan/magenta)
pub fn futuristic_shader(position: &Vec3, normal: &Vec3, light_dir: &Vec3) -> Color {
    let normal = normal.normalize();
    let intensity = nalgebra_glm::dot(&normal, light_dir).max(0.0);
    
    // Gradiente cyan a magenta
    let t = (position.y.sin() * 0.5 + 0.5).abs();
    
    let r = (t * 200.0 + 50.0) as u8;
    let g = (100.0) as u8;
    let b = ((1.0 - t) * 200.0 + 50.0) as u8;
    
    Color::new(r, g, b) * intensity
}