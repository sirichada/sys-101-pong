use core::fmt::Write;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::screen::{screenwriter, Writer};

pub struct PongGame {
    // Screen dimensions
    width: usize,
    height: usize,
    
    // Paddle properties
    player_paddle_x: usize,
    player_paddle_y: usize,
    player_paddle_width: usize,
    player_paddle_height: usize,
    player_paddle_speed: usize,
    
    // Computer paddle properties
    computer_paddle_x: usize,
    computer_paddle_y: usize,
    computer_paddle_width: usize,
    computer_paddle_height: usize,
    computer_paddle_speed: usize,
    
    // Ball properties
    ball_x: usize,
    ball_y: usize,
    ball_size: usize,
    ball_velocity_x: isize,
    ball_velocity_y: isize,
    
    // Game state
    player_score: usize,
    computer_score: usize,
    game_over: bool,
    
    // Colors
    background_color: (u8, u8, u8),
    paddle_color: (u8, u8, u8),
    ball_color: (u8, u8, u8),
    text_color: (u8, u8, u8),
}

impl PongGame {
    pub fn new(width: usize, height: usize) -> Self {
        let player_paddle_height = height / 6;
        let player_paddle_width = width / 50;
        let ball_size = width / 50;
        
        PongGame {
            width,
            height,
            
            player_paddle_x: width / 20,
            player_paddle_y: height / 2 - player_paddle_height / 2,
            player_paddle_width,
            player_paddle_height,
            player_paddle_speed: height / 50,
            
            computer_paddle_x: width - width / 20 - player_paddle_width,
            computer_paddle_y: height / 2 - player_paddle_height / 2,
            computer_paddle_width: player_paddle_width,
            computer_paddle_height: player_paddle_height,
            computer_paddle_speed: height / 60, // Slightly slower than player
            
            ball_x: width / 2 - ball_size / 2,
            ball_y: height / 2 - ball_size / 2,
            ball_size,
            ball_velocity_x: 2,
            ball_velocity_y: 1,
            
            player_score: 0,
            computer_score: 0,
            game_over: false,
            
            background_color: (0, 0, 0),      // Black
            paddle_color: (255, 255, 255),    // White
            ball_color: (255, 255, 0),        // Yellow
            text_color: (0, 255, 0),          // Green
        }
    }
    
    pub fn reset(&mut self) {
        // Reset ball position
        self.ball_x = self.width / 2 - self.ball_size / 2;
        self.ball_y = self.height / 2 - self.ball_size / 2;
        
        // Reset paddle positions
        self.player_paddle_y = self.height / 2 - self.player_paddle_height / 2;
        self.computer_paddle_y = self.height / 2 - self.computer_paddle_height / 2;
        
        // Reset velocity with slight randomization
        let direction = if self.player_score > self.computer_score { -1 } else { 1 };
        self.ball_velocity_x = direction * 2;
        self.ball_velocity_y = if self.ball_y % 2 == 0 { 1 } else { -1 };
        
        // Reset game state
        self.game_over = false;
    }
    
    pub fn new_game(&mut self) {
        self.player_score = 0;
        self.computer_score = 0;
        self.reset();
    }
    
    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(KeyCode::ArrowUp) => {
                // Move paddle up
                if self.player_paddle_y > self.player_paddle_speed {
                    self.player_paddle_y -= self.player_paddle_speed;
                } else {
                    self.player_paddle_y = 0;
                }
            },
            DecodedKey::RawKey(KeyCode::ArrowDown) => {
                // Move paddle down
                if self.player_paddle_y + self.player_paddle_height + self.player_paddle_speed < self.height {
                    self.player_paddle_y += self.player_paddle_speed;
                } else {
                    self.player_paddle_y = self.height - self.player_paddle_height;
                }
            },
            DecodedKey::Unicode(' ') if self.game_over => {
                // Restart game
                self.new_game();
            },
            _ => {}
        }
    }
    
    pub fn update(&mut self) {
        if self.game_over {
            return;
        }
        
        // Update ball position
        let new_ball_x = self.ball_x as isize + self.ball_velocity_x;
        let new_ball_y = self.ball_y as isize + self.ball_velocity_y;
        
        // Check top and bottom collisions
        if new_ball_y <= 0 || new_ball_y + self.ball_size as isize >= self.height as isize {
            self.ball_velocity_y = -self.ball_velocity_y;
        }
        
        // Check paddle collisions
        // Player paddle
        if new_ball_x <= (self.player_paddle_x + self.player_paddle_width) as isize && 
           new_ball_x >= self.player_paddle_x as isize &&
           new_ball_y + self.ball_size as isize >= self.player_paddle_y as isize && 
           new_ball_y <= (self.player_paddle_y + self.player_paddle_height) as isize {
            self.ball_velocity_x = -self.ball_velocity_x;
            // Add a little bit of spin based on where the ball hits the paddle
            let relative_intersect_y = (self.player_paddle_y as isize + (self.player_paddle_height as isize / 2)) - (new_ball_y + (self.ball_size as isize / 2));
            self.ball_velocity_y = -relative_intersect_y / 10;
            if self.ball_velocity_y == 0 {
                self.ball_velocity_y = if new_ball_y % 2 == 0 { 1 } else { -1 };
            }
        }
        
        // Computer paddle
        if (new_ball_x + self.ball_size as isize) >= self.computer_paddle_x as isize && 
           new_ball_x <= (self.computer_paddle_x + self.computer_paddle_width) as isize &&
           new_ball_y + self.ball_size as isize >= self.computer_paddle_y as isize && 
           new_ball_y <= (self.computer_paddle_y + self.computer_paddle_height) as isize {
            self.ball_velocity_x = -self.ball_velocity_x;
            // Add a little bit of spin based on where the ball hits the paddle
            let relative_intersect_y = (self.computer_paddle_y as isize + (self.computer_paddle_height as isize / 2)) - (new_ball_y + (self.ball_size as isize / 2));
            self.ball_velocity_y = -relative_intersect_y / 10;
            if self.ball_velocity_y == 0 {
                self.ball_velocity_y = if new_ball_y % 2 == 0 { 1 } else { -1 };
            }
        }
        
        // Check for scoring
        if new_ball_x <= 0 {
            // Computer scores
            self.computer_score += 1;
            self.reset();
        } else if new_ball_x + self.ball_size as isize >= self.width as isize {
            // Player scores
            self.player_score += 1;
            self.reset();
        } else {
            // Update ball position
            self.ball_x = new_ball_x as usize;
            self.ball_y = new_ball_y as usize;
        }
        
        // Check for game over condition (first to 5 points wins)
        if self.player_score >= 5 || self.computer_score >= 5 {
            self.game_over = true;
        }
        
        // Update computer paddle AI (simple following)
        let ball_center_y = self.ball_y + self.ball_size / 2;
        let paddle_center_y = self.computer_paddle_y + self.computer_paddle_height / 2;
        
        // Only move if the ball is moving towards the computer
        if self.ball_velocity_x > 0 {
            if paddle_center_y < ball_center_y {
                // Move down
                if self.computer_paddle_y + self.computer_paddle_height + self.computer_paddle_speed < self.height {
                    self.computer_paddle_y += self.computer_paddle_speed;
                } else {
                    self.computer_paddle_y = self.height - self.computer_paddle_height;
                }
            } else if paddle_center_y > ball_center_y {
                // Move up
                if self.computer_paddle_y > self.computer_paddle_speed {
                    self.computer_paddle_y -= self.computer_paddle_speed;
                } else {
                    self.computer_paddle_y = 0;
                }
            }
        }
    }
    
    pub fn render(&self) {
        let writer = screenwriter();
        
        // Clear screen
        writer.clear();
        
        // Draw middle line
        for y in (0..self.height).step_by(10) {
            for i in 0..5 {
                writer.draw_pixel(self.width / 2, y + i, 50, 50, 50);
            }
        }
        
        // Draw player paddle
        for y in self.player_paddle_y..(self.player_paddle_y + self.player_paddle_height) {
            for x in self.player_paddle_x..(self.player_paddle_x + self.player_paddle_width) {
                writer.draw_pixel(x, y, self.paddle_color.0, self.paddle_color.1, self.paddle_color.2);
            }
        }
        
        // Draw computer paddle
        for y in self.computer_paddle_y..(self.computer_paddle_y + self.computer_paddle_height) {
            for x in self.computer_paddle_x..(self.computer_paddle_x + self.computer_paddle_width) {
                writer.draw_pixel(x, y, self.paddle_color.0, self.paddle_color.1, self.paddle_color.2);
            }
        }
        
        // Draw ball
        for y in self.ball_y..(self.ball_y + self.ball_size) {
            for x in self.ball_x..(self.ball_x + self.ball_size) {
                writer.draw_pixel(x, y, self.ball_color.0, self.ball_color.1, self.ball_color.2);
            }
        }
        
        // Draw scores
        writer.write_pixel(self.width / 4, 20, 255);
        writeln!(Writer, "{}                           {}", self.player_score, self.computer_score).unwrap();
        
        // Draw game over message if applicable
        if self.game_over {
            let message = if self.player_score > self.computer_score {
                "You Win!"
            } else {
                "Computer Wins!"
            };
            
            writer.write_pixel(self.width / 2 - 40, self.height / 2 - 20, 255);
            writeln!(Writer, "{}", message).unwrap();
            writer.write_pixel(self.width / 2 - 100, self.height / 2, 255);
            writeln!(Writer, "Press SPACE to play again").unwrap();
        }
    }
}