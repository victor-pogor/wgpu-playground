use glam::{Mat4, Vec3};

// Camera struct to encapsulate camera-related functionality
pub struct Camera {
    // Camera state
    offset: [f32; 2], // x, z offsets for panning
    zoom: f32,        // zoom factor
    rotation: f32,    // rotation in radians
    base_height: f32, // base height for the camera

    // Mouse interaction state for camera control
    mouse_pressed: bool,
    last_mouse_position: [f32; 2],
    ctrl_pressed: bool,
    shift_pressed: bool,
}

impl Camera {
    pub fn new(base_height: f32) -> Self {
        Self {
            offset: [0.0, 0.0],
            zoom: 1.0,
            rotation: 0.0,
            base_height,
            mouse_pressed: false,
            last_mouse_position: [0.0, 0.0],
            ctrl_pressed: false,
            shift_pressed: false,
        }
    }

    pub fn calculate_view_matrix(&self, base_position: [f32; 3]) -> Mat4 {
        // Apply camera transformations (pan, zoom, rotate)
        let mut camera_mat = Mat4::IDENTITY;

        // First apply rotation around Y axis
        camera_mat = camera_mat * Mat4::from_rotation_y(self.rotation);

        // Then apply translation (pan)
        camera_mat =
            camera_mat * Mat4::from_translation(Vec3::new(self.offset[0], 0.0, self.offset[1]));

        // Calculate zoom-adjusted camera position
        let camera_height = base_position[1] / self.zoom;
        let camera_pos = Vec3::new(base_position[0], camera_height, base_position[2]);

        // Get target position (always looking at the center for now)
        let target_pos = Vec3::new(0.0, 0.0, 0.0);

        let view_matrix = Mat4::look_at_rh(camera_pos, target_pos, Vec3::Y);

        // Apply camera transformations to view matrix
        view_matrix * camera_mat
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        // Scale pan amount based on zoom level (faster pan when zoomed out)
        let pan_speed = 1.0 / self.zoom;

        // Apply the rotation to the pan direction
        let sin_rot = self.rotation.sin();
        let cos_rot = self.rotation.cos();

        // Apply rotation to get world-space pan
        self.offset[0] += (delta_x * cos_rot - delta_y * sin_rot) * pan_speed;
        self.offset[1] += (delta_x * sin_rot + delta_y * cos_rot) * pan_speed;
    }

    pub fn zoom(&mut self, delta: f32) {
        // Apply zoom (delta is positive for zoom in, negative for zoom out)
        let zoom_speed = 0.1;
        let new_zoom = self.zoom * (1.0 + delta * zoom_speed);

        // Clamp zoom to reasonable limits
        self.zoom = new_zoom.clamp(0.1, 10.0);
    }

    pub fn rotate(&mut self, delta: f32) {
        // Apply rotation (in radians)
        self.rotation += delta * 0.01;

        // Keep rotation in 0-2π range for simplicity
        while self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
        while self.rotation < 0.0 {
            self.rotation += std::f32::consts::TAU;
        }
    }

    // Input handling methods
    pub fn handle_mouse_press(&mut self, position: [f32; 2], ctrl: bool, shift: bool) {
        self.mouse_pressed = true;
        self.last_mouse_position = position;
        self.ctrl_pressed = ctrl;
        self.shift_pressed = shift;
    }

    pub fn handle_mouse_release(&mut self) {
        self.mouse_pressed = false;
    }

    pub fn handle_mouse_move(&mut self, position: [f32; 2]) -> bool {
        if self.mouse_pressed {
            let delta_x = position[0] - self.last_mouse_position[0];
            let delta_y = position[1] - self.last_mouse_position[1];

            if self.ctrl_pressed {
                // Pan with Ctrl+drag
                self.pan(delta_x, delta_y);
                self.last_mouse_position = position;
                return true;
            } else if self.shift_pressed {
                // Rotate with Shift+drag
                self.rotate(delta_x);
                self.last_mouse_position = position;
                return true;
            }
        }
        false
    }

    pub fn handle_mouse_wheel(&mut self, delta: f32) {
        // Zoom with mouse wheel
        self.zoom(delta);
    }

    pub fn handle_key_state(&mut self, ctrl: bool, shift: bool) {
        self.ctrl_pressed = ctrl;
        self.shift_pressed = shift;
    }
}
