use crate::prelude::*;

pub struct Camera {
    pub left_x: i32,
    pub right_x: i32,
    pub top_y: i32,
    pub bottom_y: i32,
}

const CAMERA_WIDTH: i32 = 48;
const CAMERA_HEIGHT: i32 = 44;

impl Camera {
    pub fn new(player_position: Point) -> Self {
        // View area is 48x44, thus player is at {24,22}
        Self {
            left_x: player_position.x - CAMERA_WIDTH / 2, // DISPLAY_WIDTH / 2,
            right_x: player_position.x + CAMERA_WIDTH / 2, // DISPLAY_WIDTH / 2,
            top_y: player_position.y - CAMERA_HEIGHT / 2, //DISPLAY_HEIGHT / 2,
            bottom_y: player_position.y + CAMERA_HEIGHT / 2, //DISPLAY_HEIGHT / 2,
        }
    }

    pub fn on_player_move(&mut self, player_position: Point) {
        self.left_x = player_position.x - CAMERA_WIDTH / 2; //DISPLAY_WIDTH / 2;
        self.right_x = player_position.x + CAMERA_WIDTH / 2; //DISPLAY_WIDTH / 2;
        self.top_y = player_position.y - CAMERA_HEIGHT / 2; //DISPLAY_HEIGHT / 2;
        self.bottom_y = player_position.y + CAMERA_HEIGHT / 2; //DISPLAY_HEIGHT / 2;
    }

    pub fn center_point(&self) -> Point {
        Point::new(
            (self.right_x - self.left_x) / 2,
            (self.bottom_y - self.top_y) / 2,
        )
    }
}
