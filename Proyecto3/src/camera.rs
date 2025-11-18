// camera.rs - Sistema de cámara 3D con 6 grados de libertad

use nalgebra_glm::{Vec3, Mat4};
use std::f32::consts::PI;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub yaw: f32,    // Rotación horizontal
    pub pitch: f32,  // Rotación vertical
    pub speed: f32,
    pub sensitivity: f32,
    pub zoom: f32,
    
    // Para warping
    pub is_warping: bool,
    pub warp_progress: f32,
    pub warp_start: Vec3,
    pub warp_end: Vec3,
    pub warp_duration: f32,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Camera {
            position,
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            yaw: -90.0,
            pitch: 0.0,
            speed: 15.0,
            sensitivity: 0.1,
            zoom: 1.0,
            is_warping: false,
            warp_progress: 0.0,
            warp_start: Vec3::zeros(),
            warp_end: Vec3::zeros(),
            warp_duration: 1.5,
        }
    }

    pub fn update_vectors(&mut self) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let front = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        );

        self.target = front.normalize();
        self.up = Vec3::new(0.0, 1.0, 0.0);
    }

    pub fn move_forward(&mut self, delta: f32) {
        let forward = Vec3::new(self.target.x, 0.0, self.target.z).normalize();
        self.position += forward * self.speed * delta;
    }

    pub fn move_backward(&mut self, delta: f32) {
        let forward = Vec3::new(self.target.x, 0.0, self.target.z).normalize();
        self.position -= forward * self.speed * delta;
    }

    pub fn move_left(&mut self, delta: f32) {
        let right = self.target.cross(&self.up).normalize();
        self.position -= right * self.speed * delta;
    }

    pub fn move_right(&mut self, delta: f32) {
        let right = self.target.cross(&self.up).normalize();
        self.position += right * self.speed * delta;
    }

    pub fn move_up(&mut self, delta: f32) {
        self.position.y += self.speed * delta;
    }

    pub fn move_down(&mut self, delta: f32) {
        self.position.y -= self.speed * delta;
    }

    pub fn rotate(&mut self, yaw_offset: f32, pitch_offset: f32) {
        self.yaw += yaw_offset * self.sensitivity;
        self.pitch += pitch_offset * self.sensitivity;

        // Limitar pitch para evitar gimbal lock
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.update_vectors();
    }

    pub fn start_warp(&mut self, target: Vec3) {
        self.is_warping = true;
        self.warp_progress = 0.0;
        self.warp_start = self.position;
        self.warp_end = target;
    }

    pub fn update_warp(&mut self, delta_time: f32) -> bool {
    if !self.is_warping {
        return false;
    }

    self.warp_progress += delta_time / self.warp_duration;

    if self.warp_progress >= 1.0 {
        self.position = self.warp_end;
        self.is_warping = false;
        self.warp_progress = 0.0;
        
        // Apuntar hacia el origen después del warp
        let direction = (Vec3::new(0.0, 0.0, 0.0) - self.position).normalize();
        self.target = direction;
        
        return true; // Warp completado
    }

    // Interpolación suave con easing
        let t = ease_in_out_cubic(self.warp_progress);
        self.position = self.warp_start * (1.0 - t) + self.warp_end * t;

        false
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        nalgebra_glm::look_at(
            &self.position,
            &(self.position + self.target),
            &self.up,
        )
    }

    pub fn get_screen_position(&self, world_pos: Vec3, window_width: f32, window_height: f32) -> Vec3 {
        // Proyección simple 3D a 2D
        let relative_pos = world_pos - self.position;
        let distance = relative_pos.magnitude();

        if distance < 0.1 {
            return Vec3::new(window_width / 2.0, window_height / 2.0, 0.0);
        }

        // Proyección en pantalla
        let forward = self.target;
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward).normalize();

        let local_x = relative_pos.dot(&right);
        let local_y = relative_pos.dot(&up);
        let local_z = relative_pos.dot(&forward);

        if local_z <= 0.1 {
            // Objeto detrás de la cámara
            return Vec3::new(-1000.0, -1000.0, -1.0);
        }

        let fov = 60.0_f32.to_radians();
        let aspect = window_width / window_height;

        let screen_x = window_width / 2.0 + (local_x / local_z) * (window_width / (2.0 * (fov / 2.0).tan()));
        let screen_y = window_height / 2.0 - (local_y / local_z) * (window_height / (2.0 * (fov / 2.0).tan() / aspect));

        Vec3::new(screen_x, screen_y, local_z)
    }

    pub fn check_collision(&self, object_pos: Vec3, object_radius: f32) -> bool {
        let distance = (self.position - object_pos).magnitude();
        distance < object_radius + 50.0 // 50 unidades de margen
    }

    pub fn resolve_collision(&mut self, object_pos: Vec3, object_radius: f32) {
        let direction = (self.position - object_pos).normalize();
        let min_distance = object_radius + 50.0;
        self.position = object_pos + direction * min_distance;
    }
}

// Función de easing para suavizar la animación de warp
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(3.0) / 2.0
    }
}