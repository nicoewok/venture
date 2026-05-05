#[derive(Clone, Copy, Default, PartialEq)]
pub enum CellType {
    #[default]
    Grass,
    Path,
    Tree,
    Mountain,
    River,
    Bridge,
}

pub struct WorldMap {
    pub width: u16,
    pub height: u16,
    pub grid: Vec<Vec<CellType>>,
    pub path_steps: Vec<(u16, u16)>,
}

impl WorldMap {
    pub fn new(width: u16, height: u16) -> Self {
        let mut grid = vec![vec![CellType::Grass; width as usize]; height as usize];
        
        let mut connected_path = Vec::new();
        
        // Start slightly above the bottom so the player is immediately visible
        let mut walk_y = height as i32 - 3;
        let mut walk_x = (width / 2) as i32;
        
        // End at top center
        let target_x = (width / 2) as i32;
        let target_y = 0;

        connected_path.push((walk_x as u16, walk_y as u16));
        grid[walk_y as usize][walk_x as usize] = CellType::Path;

        while walk_y > target_y {
            let total_dist = (height as i32 - 1) - target_y;
            let current_dist = (height as i32 - 1) - walk_y;
            let progress = current_dist as f32 / total_dist as f32;
            
            // Base sine wave for a winding path
            let amplitude = 15.0; // Constant amplitude
            let freq = 0.2;
            let wave_offset = (amplitude * (walk_y as f32 * freq).sin()) as i32;
            
            // Linear interpolation from start_x to target_x
            let start_base_x = (width / 2) as i32;
            let base_x = start_base_x + ((target_x - start_base_x) as f32 * progress) as i32;
            
            let desired_x = base_x + wave_offset;
            let desired_x = desired_x.clamp(2, (width as i32) - 3);

            let dist_x = desired_x - walk_x;
            
            if dist_x != 0 {
                if dist_x > 0 { walk_x += 1; } else { walk_x -= 1; }
            } else {
                walk_y -= 1;
            }
            
            connected_path.push((walk_x as u16, walk_y as u16));
            grid[walk_y as usize][walk_x as usize] = CellType::Path;
        }

        // Add rivers
        for y_base in (15..height as usize - 5).step_by(30) {
            let offset_seed = y_base as f32 * 0.1;
            for x in 0..width as usize {
                let y_offset = (2.0 * (x as f32 * 0.15 + offset_seed).sin()) as i32;
                let y = (y_base as i32 + y_offset).clamp(0, height as i32 - 1) as usize;
                
                if grid[y][x] == CellType::Path {
                    grid[y][x] = CellType::Bridge;
                } else if grid[y][x] == CellType::Grass {
                    grid[y][x] = CellType::River;
                }
            }
        }

        // Add some basic terrain (trees and mountains) procedurally with clustering
        for y in 0..height as usize {
            for x in 0..width as usize {
                if grid[y][x] != CellType::Grass { continue; }
                
                let val = (x.wrapping_mul(37) ^ y.wrapping_mul(101)) % 100;
                
                // Tree clusters
                let tree_cluster = ((x / 3).wrapping_mul(13) ^ (y / 3).wrapping_mul(7)) % 10;
                if tree_cluster < 3 && val < 40 {
                    grid[y][x] = CellType::Tree;
                }
                
                // Mountain clusters
                let mtn_cluster = ((x / 5).wrapping_mul(17) ^ (y / 5).wrapping_mul(11)) % 10;
                if mtn_cluster < 2 && val < 30 {
                    grid[y][x] = CellType::Mountain;
                }
            }
        }

        Self { width, height, grid, path_steps: connected_path }
    }
}
