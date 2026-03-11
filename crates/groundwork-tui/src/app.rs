use std::time::Duration;

use bevy_ecs::prelude::*;
use ratatui::DefaultTerminal;

use groundwork_sim::grid::GROUND_LEVEL;
use groundwork_sim::Tick;

use crate::input;
use crate::render;

pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub slice_z: usize,
    pub running: bool,
    pub auto_tick: bool,
    pub tick_rate_ms: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: groundwork_sim::create_world(),
            schedule: groundwork_sim::create_schedule(),
            slice_z: GROUND_LEVEL + 1, // Start just above ground to see the water spring.
            running: true,
            auto_tick: false,
            tick_rate_ms: 200,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while self.running {
            terminal.draw(|frame| render::draw(frame, &self.world, self))?;

            let timeout = Duration::from_millis(self.tick_rate_ms);
            if ratatui::crossterm::event::poll(timeout)? {
                input::handle_event(self)?;
            } else if self.auto_tick {
                groundwork_sim::tick(&mut self.world, &mut self.schedule);
            }
        }
        Ok(())
    }

    pub fn tick_count(&self) -> u64 {
        self.world.resource::<Tick>().0
    }

    pub fn step(&mut self) {
        groundwork_sim::tick(&mut self.world, &mut self.schedule);
    }
}
