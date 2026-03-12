/// Orbit camera for the 3D projected terminal view.
///
/// The camera orbits around a focus point. Position is derived from
/// focus + distance + yaw + pitch. Projection is orthographic —
/// all rays are parallel, giving consistent scale across the view.

use std::f64::consts::PI;

/// Simple 3-component vector for camera math.
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn len(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let l = self.len();
        if l < 1e-12 {
            return Self::new(0.0, 0.0, 1.0);
        }
        Self::new(self.x / l, self.y / l, self.z / l)
    }

    pub fn scale(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

pub struct Camera {
    /// Point the camera orbits around (world coordinates).
    pub focus: Vec3,
    /// Distance from the focus point.
    pub distance: f64,
    /// Horizontal rotation in radians (0 = looking along +Y).
    pub yaw: f64,
    /// Vertical tilt in radians (0 = horizontal, positive = looking down).
    pub pitch: f64,
    /// World units per terminal column in orthographic projection.
    pub ortho_scale: f64,
}

impl Camera {
    pub fn new(focus_x: f64, focus_y: f64, focus_z: f64) -> Self {
        Self {
            focus: Vec3::new(focus_x, focus_y, focus_z),
            distance: 40.0,
            yaw: -PI / 4.0,     // 45° — isometric-ish default
            pitch: PI / 5.0,    // 36° down — good for terrain overview
            ortho_scale: 0.6,   // each terminal column ≈ 0.6 world units
        }
    }

    /// Camera forward direction (from camera toward focus).
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            -self.pitch.sin(),
        )
        .normalized()
    }

    /// Camera right direction (horizontal, perpendicular to forward).
    pub fn right(&self) -> Vec3 {
        let world_up = Vec3::new(0.0, 0.0, 1.0);
        let fwd = self.forward();
        world_up.cross(fwd).normalized()
    }

    /// Camera up direction (perpendicular to forward and right).
    pub fn up(&self) -> Vec3 {
        let fwd = self.forward();
        let rgt = self.right();
        fwd.cross(rgt).normalized()
    }

    /// Camera position in world coordinates.
    pub fn position(&self) -> Vec3 {
        let fwd = self.forward();
        self.focus.sub(fwd.scale(self.distance))
    }

    /// Orbit horizontally around the focus point.
    pub fn orbit(&mut self, dyaw: f64) {
        self.yaw += dyaw;
    }

    /// Zoom by adjusting ortho_scale (smaller = closer).
    pub fn zoom(&mut self, factor: f64) {
        self.ortho_scale = (self.ortho_scale * factor).clamp(0.15, 3.0);
    }

    /// Move the focus point along the camera's forward direction (projected to XY plane).
    pub fn fly_forward(&mut self, amount: f64) {
        let fwd_xy = Vec3::new(self.yaw.cos(), self.yaw.sin(), 0.0).normalized();
        self.focus = self.focus.add(fwd_xy.scale(amount));
    }

    /// Move the focus point along the camera's right direction.
    pub fn pan_right(&mut self, amount: f64) {
        let rgt = self.right();
        // Project to XY plane for horizontal panning
        let rgt_xy = Vec3::new(rgt.x, rgt.y, 0.0).normalized();
        self.focus = self.focus.add(rgt_xy.scale(amount));
    }

    /// Move the focus point vertically.
    pub fn move_focus_z(&mut self, dz: f64) {
        self.focus.z += dz;
    }

    /// Compute a ray for a given terminal cell position.
    ///
    /// `col_offset` and `row_offset` are in terminal cells, relative to screen center.
    /// Terminal cells have ~2:1 aspect ratio (taller than wide), accounted for here.
    ///
    /// Returns (ray_origin, ray_direction). Direction is the same for all cells
    /// in orthographic projection.
    pub fn ray_for_cell(&self, col_offset: f64, row_offset: f64) -> (Vec3, Vec3) {
        let rgt = self.right();
        let up = self.up();
        let fwd = self.forward();

        // Terminal characters are roughly 2x taller than wide.
        // Each column spans ortho_scale world units.
        // Each row spans ortho_scale * 2.0 world units (because chars are tall).
        let world_dx = col_offset * self.ortho_scale;
        let world_dy = row_offset * self.ortho_scale * 2.0;

        let center = self.position();
        let origin = center
            .add(rgt.scale(world_dx))
            .add(up.scale(-world_dy)); // screen Y is inverted (row 0 = top)

        (origin, fwd)
    }

    /// Sync camera focus from the app's discrete focus coordinates.
    pub fn sync_focus(&mut self, fx: usize, fy: usize, fz: usize) {
        self.focus = Vec3::new(fx as f64 + 0.5, fy as f64 + 0.5, fz as f64 + 0.5);
    }
}
