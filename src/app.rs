use std::collections::VecDeque;

pub enum Scene {
    Setup,
    Tavern,
    March,
    Selection,
    Battle(usize), // Index of the monster in the active list
}

pub struct Monster {
    pub name: String,
    pub is_slain: bool,
    pub monster_type: String,
    pub dotdo_id: Option<u32>,
}

pub struct App {
    pub state: Scene,
    pub monsters: Vec<Monster>,
    pub slain_count: usize,
    pub trail: VecDeque<TrailRow>,
    pub hours: u32,
    pub mins: u32,
    pub monsters_goal: u32,
    pub path_width: usize,
    pub last_center: usize,
    pub ticks: u64,
    pub cursor_pos: usize,
    pub setup_cursor: usize,
    pub tick_rate_ms: u64,
    pub available_tasks: Vec<crate::dotdo::Task>,
    pub selected_indices: std::collections::HashSet<usize>,
    pub custom_monster_count: usize,
}

pub struct TrailRow {
    pub center: usize,
    pub content: Option<String>, // "󱜙" for monster, "⚔" for slain
    pub left_decoration: Option<String>,
    pub right_decoration: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let tasks = crate::dotdo::fetch_active_monsters();
        let mut trail = VecDeque::new();
        let last_center = 20; // Move center slightly to the right for better map view
        let path_width = 15;

        // Pre-fill trail so the Knight has a path to stand on
        for _ in 0..30 {
            trail.push_back(TrailRow {
                center: last_center,
                content: None,
                left_decoration: None,
                right_decoration: None,
            });
        }

        Self {
            state: Scene::Setup,
            monsters: Vec::new(),
            slain_count: 0,
            trail,
            hours: 1,
            mins: 0,
            monsters_goal: 3,
            path_width,
            last_center,
            ticks: 0,
            cursor_pos: 0,
            setup_cursor: 0,
            tick_rate_ms: 250,
            available_tasks: tasks,
            selected_indices: std::collections::HashSet::new(),
            custom_monster_count: 0,
        }
    }

    // Call this every time a "minute" or "tick" passes
    pub fn generate_row(&mut self) {
        if let Scene::March = self.state {
            if !self.is_goal_reached() {
                // Using the new v0.10 syntax
                let shift: i32 = rand::random_range(-1..=1);
                
                // Calculate new center with boundaries (e.g., 10 to 40 chars wide for map view)
                let new_center = (self.last_center as i32 + shift).clamp(10, 40) as usize;

                let left_dec = match rand::random_range(0..15) {
                    0 => Some("^".to_string()), // Mountain
                    1 => Some("t".to_string()), // Tree
                    2 => Some("~".to_string()), // Water
                    _ => None,
                };
                let right_dec = match rand::random_range(0..15) {
                    0 => Some("^".to_string()),
                    1 => Some("t".to_string()),
                    2 => Some("~".to_string()),
                    _ => None,
                };

                self.trail.push_front(TrailRow {
                    center: new_center,
                    content: None,
                    left_decoration: left_dec,
                    right_decoration: right_dec,
                });

                // Keep the trail buffer at a manageable size
                if self.trail.len() > 30 { 
                    self.trail.pop_back(); 
                }

                self.last_center = new_center;
                self.ticks += 1;
            }
        }
    }

    pub fn is_goal_reached(&self) -> bool {
        let hours_goal = self.hours as f32 + (self.mins as f32 / 60.0);
        // Convert goal hours to minutes/ticks for comparison
        let total_ticks_needed = (hours_goal * 3600.0) / (self.tick_rate_ms as f32 / 1000.0);
        self.ticks >= total_ticks_needed as u64
    }

    pub fn get_time_left(&self) -> String {
        if self.is_goal_reached() {
            return "00:00:00".to_string();
        }
        let hours_goal = self.hours as f32 + (self.mins as f32 / 60.0);
        let total_ticks_needed = (hours_goal * 3600.0) / (self.tick_rate_ms as f32 / 1000.0);
        let ticks_left = total_ticks_needed - self.ticks as f32;
        
        let seconds_left = (ticks_left * (self.tick_rate_ms as f32 / 1000.0)) as u64;
        let h = seconds_left / 3600;
        let m = (seconds_left % 3600) / 60;
        let s = seconds_left % 60;
        
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}