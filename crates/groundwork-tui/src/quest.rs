use std::collections::HashSet;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use bevy_ecs::world::World;

use groundwork_sim::grid::{VoxelGrid, GROUND_LEVEL};
use groundwork_sim::tree::{PlantType, SpeciesTable};
use groundwork_sim::voxel::Material;

// ---------------------------------------------------------------------------
// Quest definitions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuestId {
    PanAround,
    ChangeDepth,
    SwitchTo3D,
    FindTheSpring,
    PlaceWater,
    ObserveWetSoil,
    PlantFirstSeed,
    WatchItGrow,
    ChangeSpecies,
    ToggleAutoTick,
    StepManually,
    OpenInspect,
    InspectSoil,
    ViewUnderground,
    FindRoots,
    PlantThreeSpecies,
    PlantAllTypes,
    UseShovel,
    ShapeTerrain,
    GrowATree,
}

struct QuestDef {
    id: QuestId,
    name: &'static str,
    chapter: usize,
    detail: &'static str,
}

const QUEST_DEFS: &[QuestDef] = &[
    // Chapter 0: Getting Your Bearings
    QuestDef {
        id: QuestId::PanAround,
        name: "Move the camera",
        chapter: 0,
        detail: "Your garden is a 60m x 60m plot of living earth. \
                 Use WASD to slide your view across the landscape. \
                 Everything you see is a half-meter voxel you can shape.",
    },
    QuestDef {
        id: QuestId::ChangeDepth,
        name: "Change depth level",
        chapter: 0,
        detail: "Your garden has layers. Press K to look higher \
                 (sky, canopy) and J to look deeper (roots, stone). \
                 The surface sits at depth 0.",
    },
    QuestDef {
        id: QuestId::SwitchTo3D,
        name: "Try 3D view",
        chapter: 0,
        detail: "Press V to see your garden in projected 3D. \
                 Orbit with Q/E, fly with WASD, zoom with \
                 Shift+W/S. Press V again to return to slice view.",
    },
    // Chapter 1: Water is Life
    QuestDef {
        id: QuestId::FindTheSpring,
        name: "Find the water spring",
        chapter: 1,
        detail: "Every garden has a natural water spring deep \
                 underground. Water bubbles up, flows outward, \
                 and soaks into soil. Find it — it's the source \
                 of life for everything you'll grow.",
    },
    QuestDef {
        id: QuestId::PlaceWater,
        name: "Use the watering can",
        chapter: 1,
        detail: "Select the watering can with Tab, then Space to \
                 pour. Water obeys gravity — it falls through air \
                 and pools on solid ground. Plants need water \
                 nearby to grow.",
    },
    QuestDef {
        id: QuestId::ObserveWetSoil,
        name: "Find wet soil",
        chapter: 1,
        detail: "When water touches soil, it soaks in. Wet soil \
                 (%%) retains moisture that roots absorb. The \
                 wetter the soil, the faster seeds germinate.",
    },
    // Chapter 2: First Planting
    QuestDef {
        id: QuestId::PlantFirstSeed,
        name: "Plant a seed",
        chapter: 2,
        detail: "Select the seed bag with Tab and press Space \
                 to plant. Seeds fall through air and land on \
                 solid ground. They need both water and light \
                 to germinate.",
    },
    QuestDef {
        id: QuestId::WatchItGrow,
        name: "Watch a seed sprout",
        chapter: 2,
        detail: "Seeds germinate into seedlings, then saplings, \
                 then mature plants. Watch the nutrient bar in \
                 the inspect panel — at 200, the seed sprouts \
                 into a living plant.",
    },
    QuestDef {
        id: QuestId::ChangeSpecies,
        name: "Try a different species",
        chapter: 2,
        detail: "Press [ and ] to cycle through 12 species: \
                 trees, shrubs, flowers, and groundcover. Each \
                 has unique height, water needs, and growth rate.",
    },
    // Chapter 3: Time Control
    QuestDef {
        id: QuestId::ToggleAutoTick,
        name: "Toggle auto-tick",
        chapter: 3,
        detail: "Press P to start the simulation clock. Time \
                 advances and your garden evolves. Press P again \
                 to pause. Each tick, water flows, light spreads, \
                 and plants grow.",
    },
    QuestDef {
        id: QuestId::StepManually,
        name: "Step time manually",
        chapter: 3,
        detail: "Press Shift+P to advance exactly one tick. \
                 Watch water seep, soil moisten, and seeds \
                 accumulate nutrients tick by tick.",
    },
    // Chapter 4: Read the Land
    QuestDef {
        id: QuestId::OpenInspect,
        name: "Open the inspect panel",
        chapter: 4,
        detail: "Press I to toggle the inspect panel. It shows \
                 material type, water level, light level, and \
                 nutrients. Essential for understanding why \
                 plants thrive or struggle.",
    },
    QuestDef {
        id: QuestId::InspectSoil,
        name: "Inspect a soil cell",
        chapter: 4,
        detail: "Move your cursor to a soil cell. Soil has \
                 composition (sand, clay, organic, rock), pH, \
                 and bacteria. Organic-rich soil near roots \
                 grows the healthiest plants.",
    },
    // Chapter 5: Going Underground
    QuestDef {
        id: QuestId::ViewUnderground,
        name: "Go below surface",
        chapter: 5,
        detail: "Press J to descend below the surface. Under\u{00AD}\
                 ground you'll find soil layers, stone, water \
                 channels, and — once plants grow — roots \
                 spreading through the earth.",
    },
    QuestDef {
        id: QuestId::FindRoots,
        name: "Find plant roots",
        chapter: 5,
        detail: "Plant roots spread underground seeking water. \
                 They absorb moisture from soil and deliver \
                 nutrients upward. Healthy root networks mean \
                 healthy plants above.",
    },
    // Chapter 6: Biodiversity
    QuestDef {
        id: QuestId::PlantThreeSpecies,
        name: "Plant 3 species",
        chapter: 6,
        detail: "Different species thrive in different conditions. \
                 Trees grow tall but slowly. Shrubs fill gaps. \
                 Flowers bloom fast. A diverse garden is a \
                 resilient garden.",
    },
    QuestDef {
        id: QuestId::PlantAllTypes,
        name: "Plant all plant types",
        chapter: 6,
        detail: "Plant a tree, shrub, flower, and groundcover. \
                 Trees branch via space colonization, while \
                 others follow fixed templates. Each type fills \
                 a different ecological niche.",
    },
    // Chapter 7: Shaping the Land
    QuestDef {
        id: QuestId::UseShovel,
        name: "Dig with the shovel",
        chapter: 7,
        detail: "Select the shovel with Tab and press Space to \
                 dig. The shovel removes anything: soil, stone, \
                 seeds, even roots. Carve channels, clear space, \
                 or reshape terrain.",
    },
    QuestDef {
        id: QuestId::ShapeTerrain,
        name: "Use the range tool",
        chapter: 7,
        detail: "Press Space to mark a start, move your cursor, \
                 then Space again to fill the region. Perfect \
                 for building walls, digging ponds, or planting \
                 rows of seeds.",
    },
    // Chapter 8: Living Ecosystem
    QuestDef {
        id: QuestId::GrowATree,
        name: "Grow a tree",
        chapter: 8,
        detail: "A mature tree is the crown jewel of your garden. \
                 It disperses seeds that become new plants — the \
                 beginning of a self-sustaining ecosystem. Your \
                 garden is alive.",
    },
];

const CHAPTER_NAMES: &[&str] = &[
    "Getting Your Bearings",
    "Water is Life",
    "First Planting",
    "Time Control",
    "Read the Land",
    "Going Underground",
    "Biodiversity",
    "Shaping the Land",
    "Living Ecosystem",
];

// ---------------------------------------------------------------------------
// Action tracking
// ---------------------------------------------------------------------------

pub enum Action {
    Pan,
    ChangeDepth,
    SwitchTo3D,
    PlaceWater,
    PlantSeed(usize),
    CycleSpecies,
    ToggleAutoTick,
    StepManually,
    ToggleInspect,
    UseShovel,
    UseToolRange,
}

pub struct ActionTracker {
    pub pan_count: u32,
    pub depth_changed: bool,
    pub switched_3d: bool,
    pub placed_water: bool,
    pub planted_seed: bool,
    pub cycled_species: bool,
    pub toggled_auto_tick: bool,
    pub stepped_manually: bool,
    pub toggled_inspect: bool,
    pub used_shovel: bool,
    pub used_tool_range: bool,
    pub species_planted: HashSet<usize>,
}

impl ActionTracker {
    fn new() -> Self {
        Self {
            pan_count: 0,
            depth_changed: false,
            switched_3d: false,
            placed_water: false,
            planted_seed: false,
            cycled_species: false,
            toggled_auto_tick: false,
            stepped_manually: false,
            toggled_inspect: false,
            used_shovel: false,
            used_tool_range: false,
            species_planted: HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Quest state
// ---------------------------------------------------------------------------

pub struct Quest {
    pub id: QuestId,
    pub name: &'static str,
    pub chapter: usize,
    pub detail: &'static str,
    pub completed: bool,
}

pub struct QuestLog {
    pub quests: Vec<Quest>,
    pub current_chapter: usize,
    pub selected_index: usize,
    pub actions: ActionTracker,
    pub notification: Option<(String, u8)>,
    pub all_complete: bool,
}

impl QuestLog {
    pub fn new() -> Self {
        let quests = QUEST_DEFS
            .iter()
            .map(|def| Quest {
                id: def.id,
                name: def.name,
                chapter: def.chapter,
                detail: def.detail,
                completed: false,
            })
            .collect();

        Self {
            quests,
            current_chapter: 0,
            selected_index: 0,
            actions: ActionTracker::new(),
            notification: None,
            all_complete: false,
        }
    }

    pub fn record(&mut self, action: Action) {
        match action {
            Action::Pan => self.actions.pan_count += 1,
            Action::ChangeDepth => self.actions.depth_changed = true,
            Action::SwitchTo3D => self.actions.switched_3d = true,
            Action::PlaceWater => self.actions.placed_water = true,
            Action::PlantSeed(id) => {
                self.actions.planted_seed = true;
                self.actions.species_planted.insert(id);
            }
            Action::CycleSpecies => self.actions.cycled_species = true,
            Action::ToggleAutoTick => self.actions.toggled_auto_tick = true,
            Action::StepManually => self.actions.stepped_manually = true,
            Action::ToggleInspect => self.actions.toggled_inspect = true,
            Action::UseShovel => self.actions.used_shovel = true,
            Action::UseToolRange => self.actions.used_tool_range = true,
        }
    }

    /// Check quest completion against current world and action state.
    pub fn check(&mut self, world: &World, focus: (usize, usize, usize), show_inspect: bool) {
        if self.all_complete {
            return;
        }

        let grid = world.resource::<VoxelGrid>();
        let table = world.resource::<SpeciesTable>();

        // Pre-compute focus voxel properties
        let focus_voxel = grid.get(focus.0, focus.1, focus.2);
        let focus_mat = focus_voxel.map(|v| v.material);
        let focus_water = focus_voxel.map(|v| v.water_level).unwrap_or(0);

        // Lazily scan grid only for quests that need it
        let needs_trunk_check = self.quest_active(QuestId::WatchItGrow);
        let has_trunk =
            needs_trunk_check && grid.cells().iter().any(|v| v.material == Material::Trunk);

        let needs_tree_check = self.quest_active(QuestId::GrowATree);
        let tree_grown = needs_tree_check && {
            let mut leaves = 0u32;
            let mut branches = 0u32;
            for v in grid.cells() {
                if v.material == Material::Leaf {
                    leaves += 1;
                }
                if v.material == Material::Branch {
                    branches += 1;
                }
                if leaves >= 50 && branches >= 10 {
                    break;
                }
            }
            leaves >= 50 && branches >= 10
        };

        // Evaluate each uncompleted quest in the current chapter
        let mut newly_completed: Vec<(usize, &'static str)> = Vec::new();

        for (i, quest) in self.quests.iter().enumerate() {
            if quest.completed || quest.chapter != self.current_chapter {
                continue;
            }

            let complete = match quest.id {
                QuestId::PanAround => self.actions.pan_count >= 4,
                QuestId::ChangeDepth => self.actions.depth_changed,
                QuestId::SwitchTo3D => self.actions.switched_3d,
                QuestId::FindTheSpring => focus_mat == Some(Material::Water),
                QuestId::PlaceWater => self.actions.placed_water,
                QuestId::ObserveWetSoil => {
                    focus_mat == Some(Material::Soil) && focus_water > 50
                }
                QuestId::PlantFirstSeed => self.actions.planted_seed,
                QuestId::WatchItGrow => has_trunk,
                QuestId::ChangeSpecies => self.actions.cycled_species,
                QuestId::ToggleAutoTick => self.actions.toggled_auto_tick,
                QuestId::StepManually => self.actions.stepped_manually,
                QuestId::OpenInspect => self.actions.toggled_inspect,
                QuestId::InspectSoil => {
                    show_inspect && focus_mat == Some(Material::Soil)
                }
                QuestId::ViewUnderground => focus.2 < GROUND_LEVEL,
                QuestId::FindRoots => focus_mat == Some(Material::Root),
                QuestId::PlantThreeSpecies => self.actions.species_planted.len() >= 3,
                QuestId::PlantAllTypes => {
                    has_all_plant_types(&self.actions.species_planted, table)
                }
                QuestId::UseShovel => self.actions.used_shovel,
                QuestId::ShapeTerrain => self.actions.used_tool_range,
                QuestId::GrowATree => tree_grown,
            };

            if complete {
                newly_completed.push((i, quest.name));
            }
        }

        // Apply completions
        for (i, name) in newly_completed {
            self.quests[i].completed = true;
            self.notification = Some((format!("Done: {name}"), 40));
        }

        self.try_advance_chapter();
    }

    /// True if the given quest is in the current chapter and not yet completed.
    fn quest_active(&self, id: QuestId) -> bool {
        self.quests
            .iter()
            .any(|q| q.id == id && q.chapter == self.current_chapter && !q.completed)
    }

    fn try_advance_chapter(&mut self) {
        let chapter_complete = self
            .quests
            .iter()
            .filter(|q| q.chapter == self.current_chapter)
            .all(|q| q.completed);

        if chapter_complete {
            if self.current_chapter < CHAPTER_NAMES.len() - 1 {
                self.current_chapter += 1;
                self.selected_index = 0;
                self.notification = Some((
                    format!("Chapter: {}", CHAPTER_NAMES[self.current_chapter]),
                    60,
                ));
            } else {
                self.all_complete = true;
                self.notification = Some(("All missions complete!".to_string(), 80));
            }
        }
    }

    pub fn chapter_progress(&self) -> (usize, usize) {
        let total = self
            .quests
            .iter()
            .filter(|q| q.chapter == self.current_chapter)
            .count();
        let done = self
            .quests
            .iter()
            .filter(|q| q.chapter == self.current_chapter && q.completed)
            .count();
        (done, total)
    }

    pub fn select_next(&mut self) {
        let count = self
            .quests
            .iter()
            .filter(|q| q.chapter == self.current_chapter)
            .count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    pub fn select_prev(&mut self) {
        let count = self
            .quests
            .iter()
            .filter(|q| q.chapter == self.current_chapter)
            .count();
        if count > 0 {
            self.selected_index = (self.selected_index + count - 1) % count;
        }
    }

    /// Decrease the notification timer by one frame. Call from the render loop.
    pub fn tick_notification(&mut self) {
        if let Some((_, ref mut frames)) = self.notification {
            if *frames == 0 {
                self.notification = None;
            } else {
                *frames -= 1;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    /// Build the Lines for the MISSIONS section of the side panel.
    /// `expanded` — whether the full quest list is shown (M key toggles).
    /// `panel_width` — total panel width in characters.
    pub fn render_lines(&self, expanded: bool, panel_width: usize) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();

        // Badge
        lines.push(Line::from(vec![Span::styled(
            " MISSIONS (M) ",
            Style::default().fg(Color::Black).bg(Color::White),
        )]));

        // Notification flash
        if let Some((ref msg, _)) = self.notification {
            lines.push(Line::from(vec![Span::styled(
                format!(" {msg}"),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));
        }

        if !expanded {
            // Collapsed: show chapter + progress
            let (done, total) = self.chapter_progress();
            let label = Style::default().fg(Color::DarkGray);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" Ch.{} ", self.current_chapter + 1),
                    label,
                ),
                Span::styled(
                    format!("{done}/{total}"),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" {}", CHAPTER_NAMES[self.current_chapter]),
                    label,
                ),
            ]));
            lines.push(Line::from(""));
            return lines;
        }

        lines.push(Line::from(""));

        if self.all_complete {
            lines.push(Line::from(vec![Span::styled(
                " All missions complete!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(vec![Span::styled(
                " Your garden is alive.",
                Style::default().fg(Color::Green),
            )]));
            lines.push(Line::from(""));
            return lines;
        }

        // Chapter header
        lines.push(Line::from(vec![Span::styled(
            format!(
                " Ch.{}: {}",
                self.current_chapter + 1,
                CHAPTER_NAMES[self.current_chapter]
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Quest list for current chapter
        let chapter_quests: Vec<&Quest> = self
            .quests
            .iter()
            .filter(|q| q.chapter == self.current_chapter)
            .collect();

        let mut selected_detail: Option<&str> = None;

        for (i, quest) in chapter_quests.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let check = if quest.completed { "x" } else { " " };
            let prefix = if is_selected { ">" } else { " " };

            let marker_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let name_style = if quest.completed {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" {prefix}[{check}] "), marker_style),
                Span::styled(quest.name.to_string(), name_style),
            ]));

            if is_selected {
                selected_detail = Some(quest.detail);
            }
        }

        // Detail text for selected quest (word-wrapped)
        if let Some(detail) = selected_detail {
            lines.push(Line::from(""));
            let usable = panel_width.saturating_sub(4); // indent + margin
            let label_style = Style::default().fg(Color::DarkGray);
            for wrapped in word_wrap(detail, usable) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(wrapped, label_style),
                ]));
            }
        }

        // Navigation hint
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  ,/. select quest",
            Style::default().fg(Color::DarkGray),
        )]));

        lines.push(Line::from(""));
        lines
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn has_all_plant_types(planted: &HashSet<usize>, table: &SpeciesTable) -> bool {
    let mut has = [false; 4]; // Tree, Shrub, Groundcover, Flower
    for &id in planted {
        if let Some(species) = table.species.get(id) {
            match species.plant_type {
                PlantType::Tree => has[0] = true,
                PlantType::Shrub => has[1] = true,
                PlantType::Groundcover => has[2] = true,
                PlantType::Flower => has[3] = true,
            }
        }
    }
    has.iter().all(|&h| h)
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}
