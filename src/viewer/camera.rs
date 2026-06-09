use glam::{DMat4, DVec3};

pub struct OrbitCamera {
    pub target: DVec3,
    pub radius: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub perspective: bool,
    pub fov_y: f64,
    pub z_near: f64,
    pub z_far: f64,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            target: DVec3::ZERO,
            radius: 50.0,
            yaw: 0.0,
            pitch: 0.4,
            perspective: true,
            fov_y: std::f64::consts::FRAC_PI_4,
            z_near: 0.1,
            z_far: 10000.0,
        }
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        let forward = self.forward();
        let right = forward.cross(DVec3::Y).normalize();
        let up = right.cross(forward).normalize();
        let speed = self.radius * 0.002;
        self.target -= right * dx * speed;
        self.target += up * dy * speed;
    }

    pub fn zoom(&mut self, delta: f64) {
        self.radius = (self.radius * (1.0 - delta)).clamp(0.1, 10000.0);
    }

    pub fn dolly(&mut self, amount: f64) {
        // amount is unscaled (raw pixels or notch-equivalent)
        // scaled by radius * 0.002 to match old pan sensitivity
        self.target += self.forward() * amount * self.radius * 0.002;
    }

    pub fn rotate(&mut self, dx: f64, dy: f64) {
        self.yaw -= dx * 0.005;
        self.pitch = (self.pitch - dy * 0.005).clamp(-1.5, 1.5);
    }

    pub fn toggle_projection(&mut self) {
        self.perspective = !self.perspective;
    }

    pub fn view_matrix(&self) -> DMat4 {
        let pos = self.position();
        DMat4::look_at_lh(pos, self.target, DVec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f64) -> DMat4 {
        if self.perspective {
            DMat4::perspective_lh(self.fov_y, aspect, self.z_near, self.z_far)
        } else {
            let height = self.radius * 0.5;
            let width = height * aspect;
            DMat4::orthographic_lh(-width, width, -height, height, self.z_near, self.z_far)
        }
    }

    pub fn matrix(&self, aspect: f64) -> DMat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }

    pub fn position(&self) -> DVec3 {
        let spherical = DVec3::new(
            self.radius * self.yaw.cos() * self.pitch.cos(),
            self.radius * self.pitch.sin(),
            self.radius * self.yaw.sin() * self.pitch.cos(),
        );
        self.target + spherical
    }

    /// World-aligned right vector (horizontal only, Y=0).
    /// For screen-space panning, use locally computed vectors in `pan()`.
    #[expect(dead_code)]
    pub fn right(&self) -> DVec3 {
        DVec3::new(self.yaw.cos(), 0.0, self.yaw.sin())
    }

    /// World-aligned up vector (pitch-aware, uses `forward()` internally).
    /// For screen-space panning, use locally computed vectors in `pan()`.
    #[expect(dead_code)]
    pub fn up(&self) -> DVec3 {
        let forward = self.forward();
        let right = forward.cross(DVec3::Y).normalize();
        right.cross(forward).normalize()
    }

    pub fn forward(&self) -> DVec3 {
        (self.target - self.position()).normalize()
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::new()
    }
}
