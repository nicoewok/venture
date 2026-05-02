
use crate::ui::{WorldMap, Animation};

pub enum Scene {
    Setup,
    Tavern,
    Naming,
    March,
    Selection,
    Battle(usize), // Index of the monster in the active list
}

pub struct Monster {
    pub name: String,
    pub is_slain: bool,
    pub monster_type: String,
    pub dotdo_id: Option<u32>,
    pub art_idx: usize,
}

pub struct App {
    pub state: Scene,
    pub monsters: Vec<Monster>,
    pub slain_count: usize,
    pub map: WorldMap,
    pub hours: u32,
    pub mins: u32,
    pub monsters_goal: u32,
    pub ticks: u64,
    pub cursor_pos: usize,
    pub setup_cursor: usize,
    pub tick_rate_ms: u64,
    pub available_tasks: Vec<crate::dotdo::Task>,
    pub selected_indices: std::collections::HashSet<usize>,
    pub custom_monsters: Vec<String>,
    pub naming_input: String,
    pub animation: Option<Animation>,
}

impl App {
    pub fn new() -> Self {
        let tasks = crate::dotdo::fetch_active_monsters();
        let map = WorldMap::new(60, 100); // 60 width, 100 height for a long journey

        Self {
            state: Scene::Setup,
            monsters: Vec::new(),
            slain_count: 0,
            map,
            hours: 1,
            mins: 0,
            monsters_goal: 3,
            ticks: 0,
            cursor_pos: 0,
            setup_cursor: 0,
            tick_rate_ms: 250,
            available_tasks: tasks,
            selected_indices: std::collections::HashSet::new(),
            custom_monsters: Vec::new(),
            naming_input: String::new(),
            animation: None,
        }
    }

    // Call this every time a "minute" or "tick" passes
    pub fn generate_row(&mut self) {
        if let Some(ref mut anim) = self.animation {
            match anim {
                Animation::Slash { frame, .. } => {
                    *frame += 1;
                    if *frame > 12 {
                        self.animation = None;
                        self.state = Scene::March;
                    }
                }
            }
            return;
        }

        if let Scene::March = self.state {
            if !self.is_goal_reached() {
                self.ticks += 1;
            }
        }
    }

    pub fn get_total_ticks(&self) -> u64 {
        let hours_goal = self.hours as f32 + (self.mins as f32 / 60.0);
        ((hours_goal * 3600.0) / (self.tick_rate_ms as f32 / 1000.0)) as u64
    }

    pub fn get_knight_path_index(&self) -> usize {
        let total = self.get_total_ticks();
        if total == 0 { return 0; }
        let progress = (self.ticks as f32 / total as f32).min(1.0);
        (progress * (self.map.path_steps.len() - 1) as f32) as usize
    }

    pub fn is_goal_reached(&self) -> bool {
        self.ticks >= self.get_total_ticks()
    }

    pub fn all_monsters_slain(&self) -> bool {
        !self.monsters.is_empty() && self.monsters.iter().all(|m| m.is_slain)
    }

    pub fn is_quest_finished(&self) -> bool {
        self.is_goal_reached() || self.all_monsters_slain()
    }

    pub fn get_time_left(&self) -> String {
        if self.is_goal_reached() {
            return "00:00:00".to_string();
        }
        let total_ticks_needed = self.get_total_ticks();
        let ticks_left = total_ticks_needed.saturating_sub(self.ticks) as f32;
        
        let seconds_left = (ticks_left * (self.tick_rate_ms as f32 / 1000.0)) as u64;
        let h = seconds_left / 3600;
        let m = (seconds_left % 3600) / 60;
        let s = seconds_left % 60;
        
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}