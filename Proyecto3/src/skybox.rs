// skybox.rs - Sistema de skybox con estrellas procedurales

use crate::framebuffer::Framebuffer;
use nalgebra_glm::Vec3;

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub brightness: u8,
    pub size: u8,
}

pub struct Skybox {
    stars: Vec<Star>,
}

impl Skybox {
    pub fn new(star_count: usize, width: usize, height: usize) -> Self {
        let mut stars = Vec::new();
        
        // Generar estrellas usando un seed determinista
        for i in 0..star_count {
            let seed = i as f32 * 12.9898;
            let x = ((seed * 78.233).sin() * 43758.5453).fract() * width as f32;
            let y = ((seed * 45.164).sin() * 43758.5453).fract() * height as f32;
            let brightness = (((seed * 12.345).sin() * 43758.5453).fract() * 155.0 + 100.0) as u8;
            let size = if ((seed * 67.890).sin() * 43758.5453).fract() > 0.95 { 2 } else { 1 };
            
            stars.push(Star { x, y, brightness, size });
        }
        
        Skybox { stars }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer) {
    for star in &self.stars {
        let color = ((star.brightness as u32) << 16) 
                  | ((star.brightness as u32) << 8) 
                  | (star.brightness as u32);
        
        framebuffer.set_current_color(color);
        
        let x = star.x as usize;
        let y = star.y as usize;
        
        if x < framebuffer.width && y < framebuffer.height {
            // Usar depth = 1000000.0 en lugar de INFINITY
            framebuffer.point(x, y, 1000000.0);
            
            if star.size == 2 {
                if x + 1 < framebuffer.width {
                    framebuffer.point(x + 1, y, 1000000.0);
                }
                if y + 1 < framebuffer.height {
                    framebuffer.point(x, y + 1, 1000000.0);
                }
                if x + 1 < framebuffer.width && y + 1 < framebuffer.height {
                    framebuffer.point(x + 1, y + 1, 1000000.0);
                }
            }
        }
    }
}

pub fn render_with_twinkle(&self, framebuffer: &mut Framebuffer, time: f32) {
    for (i, star) in self.stars.iter().enumerate() {
        let twinkle = ((time * 2.0 + i as f32 * 0.1).sin() * 0.5 + 0.5);
        let brightness = (star.brightness as f32 * (0.7 + twinkle * 0.3)) as u8;
        
        let color = ((brightness as u32) << 16) 
                  | ((brightness as u32) << 8) 
                  | (brightness as u32);
        
        framebuffer.set_current_color(color);
        
        let x = star.x as usize;
        let y = star.y as usize;
        
        if x < framebuffer.width && y < framebuffer.height {
            framebuffer.point(x, y, 1000000.0);  // ← Cambiar aquí también
            
            if star.size == 2 {
                if x + 1 < framebuffer.width {
                    framebuffer.point(x + 1, y, 1000000.0);
                }
                if y + 1 < framebuffer.height {
                    framebuffer.point(x, y + 1, 1000000.0);
                }
                if x + 1 < framebuffer.width && y + 1 < framebuffer.height {
                    framebuffer.point(x + 1, y + 1, 1000000.0);
                }
            }
        }
    }
}
}