use ratatui::style::Color;
use ratatui::style::palette::tailwind;

pub const TAILWIND_GRID: [[Color; 11]; 22] = [
    [tailwind::SLATE.c50, tailwind::SLATE.c100, tailwind::SLATE.c200, tailwind::SLATE.c300, tailwind::SLATE.c400, tailwind::SLATE.c500, tailwind::SLATE.c600, tailwind::SLATE.c700, tailwind::SLATE.c800, tailwind::SLATE.c900, tailwind::SLATE.c950],
    [tailwind::GRAY.c50, tailwind::GRAY.c100, tailwind::GRAY.c200, tailwind::GRAY.c300, tailwind::GRAY.c400, tailwind::GRAY.c500, tailwind::GRAY.c600, tailwind::GRAY.c700, tailwind::GRAY.c800, tailwind::GRAY.c900, tailwind::GRAY.c950],
    [tailwind::ZINC.c50, tailwind::ZINC.c100, tailwind::ZINC.c200, tailwind::ZINC.c300, tailwind::ZINC.c400, tailwind::ZINC.c500, tailwind::ZINC.c600, tailwind::ZINC.c700, tailwind::ZINC.c800, tailwind::ZINC.c900, tailwind::ZINC.c950],
    [tailwind::NEUTRAL.c50, tailwind::NEUTRAL.c100, tailwind::NEUTRAL.c200, tailwind::NEUTRAL.c300, tailwind::NEUTRAL.c400, tailwind::NEUTRAL.c500, tailwind::NEUTRAL.c600, tailwind::NEUTRAL.c700, tailwind::NEUTRAL.c800, tailwind::NEUTRAL.c900, tailwind::NEUTRAL.c950],
    [tailwind::STONE.c50, tailwind::STONE.c100, tailwind::STONE.c200, tailwind::STONE.c300, tailwind::STONE.c400, tailwind::STONE.c500, tailwind::STONE.c600, tailwind::STONE.c700, tailwind::STONE.c800, tailwind::STONE.c900, tailwind::STONE.c950],
    [tailwind::RED.c50, tailwind::RED.c100, tailwind::RED.c200, tailwind::RED.c300, tailwind::RED.c400, tailwind::RED.c500, tailwind::RED.c600, tailwind::RED.c700, tailwind::RED.c800, tailwind::RED.c900, tailwind::RED.c950],
    [tailwind::ORANGE.c50, tailwind::ORANGE.c100, tailwind::ORANGE.c200, tailwind::ORANGE.c300, tailwind::ORANGE.c400, tailwind::ORANGE.c500, tailwind::ORANGE.c600, tailwind::ORANGE.c700, tailwind::ORANGE.c800, tailwind::ORANGE.c900, tailwind::ORANGE.c950],
    [tailwind::AMBER.c50, tailwind::AMBER.c100, tailwind::AMBER.c200, tailwind::AMBER.c300, tailwind::AMBER.c400, tailwind::AMBER.c500, tailwind::AMBER.c600, tailwind::AMBER.c700, tailwind::AMBER.c800, tailwind::AMBER.c900, tailwind::AMBER.c950],
    [tailwind::YELLOW.c50, tailwind::YELLOW.c100, tailwind::YELLOW.c200, tailwind::YELLOW.c300, tailwind::YELLOW.c400, tailwind::YELLOW.c500, tailwind::YELLOW.c600, tailwind::YELLOW.c700, tailwind::YELLOW.c800, tailwind::YELLOW.c900, tailwind::YELLOW.c950],
    [tailwind::LIME.c50, tailwind::LIME.c100, tailwind::LIME.c200, tailwind::LIME.c300, tailwind::LIME.c400, tailwind::LIME.c500, tailwind::LIME.c600, tailwind::LIME.c700, tailwind::LIME.c800, tailwind::LIME.c900, tailwind::LIME.c950],
    [tailwind::GREEN.c50, tailwind::GREEN.c100, tailwind::GREEN.c200, tailwind::GREEN.c300, tailwind::GREEN.c400, tailwind::GREEN.c500, tailwind::GREEN.c600, tailwind::GREEN.c700, tailwind::GREEN.c800, tailwind::GREEN.c900, tailwind::GREEN.c950],
    [tailwind::EMERALD.c50, tailwind::EMERALD.c100, tailwind::EMERALD.c200, tailwind::EMERALD.c300, tailwind::EMERALD.c400, tailwind::EMERALD.c500, tailwind::EMERALD.c600, tailwind::EMERALD.c700, tailwind::EMERALD.c800, tailwind::EMERALD.c900, tailwind::EMERALD.c950],
    [tailwind::TEAL.c50, tailwind::TEAL.c100, tailwind::TEAL.c200, tailwind::TEAL.c300, tailwind::TEAL.c400, tailwind::TEAL.c500, tailwind::TEAL.c600, tailwind::TEAL.c700, tailwind::TEAL.c800, tailwind::TEAL.c900, tailwind::TEAL.c950],
    [tailwind::CYAN.c50, tailwind::CYAN.c100, tailwind::CYAN.c200, tailwind::CYAN.c300, tailwind::CYAN.c400, tailwind::CYAN.c500, tailwind::CYAN.c600, tailwind::CYAN.c700, tailwind::CYAN.c800, tailwind::CYAN.c900, tailwind::CYAN.c950],
    [tailwind::SKY.c50, tailwind::SKY.c100, tailwind::SKY.c200, tailwind::SKY.c300, tailwind::SKY.c400, tailwind::SKY.c500, tailwind::SKY.c600, tailwind::SKY.c700, tailwind::SKY.c800, tailwind::SKY.c900, tailwind::SKY.c950],
    [tailwind::BLUE.c50, tailwind::BLUE.c100, tailwind::BLUE.c200, tailwind::BLUE.c300, tailwind::BLUE.c400, tailwind::BLUE.c500, tailwind::BLUE.c600, tailwind::BLUE.c700, tailwind::BLUE.c800, tailwind::BLUE.c900, tailwind::BLUE.c950],
    [tailwind::INDIGO.c50, tailwind::INDIGO.c100, tailwind::INDIGO.c200, tailwind::INDIGO.c300, tailwind::INDIGO.c400, tailwind::INDIGO.c500, tailwind::INDIGO.c600, tailwind::INDIGO.c700, tailwind::INDIGO.c800, tailwind::INDIGO.c900, tailwind::INDIGO.c950],
    [tailwind::VIOLET.c50, tailwind::VIOLET.c100, tailwind::VIOLET.c200, tailwind::VIOLET.c300, tailwind::VIOLET.c400, tailwind::VIOLET.c500, tailwind::VIOLET.c600, tailwind::VIOLET.c700, tailwind::VIOLET.c800, tailwind::VIOLET.c900, tailwind::VIOLET.c950],
    [tailwind::PURPLE.c50, tailwind::PURPLE.c100, tailwind::PURPLE.c200, tailwind::PURPLE.c300, tailwind::PURPLE.c400, tailwind::PURPLE.c500, tailwind::PURPLE.c600, tailwind::PURPLE.c700, tailwind::PURPLE.c800, tailwind::PURPLE.c900, tailwind::PURPLE.c950],
    [tailwind::FUCHSIA.c50, tailwind::FUCHSIA.c100, tailwind::FUCHSIA.c200, tailwind::FUCHSIA.c300, tailwind::FUCHSIA.c400, tailwind::FUCHSIA.c500, tailwind::FUCHSIA.c600, tailwind::FUCHSIA.c700, tailwind::FUCHSIA.c800, tailwind::FUCHSIA.c900, tailwind::FUCHSIA.c950],
    [tailwind::PINK.c50, tailwind::PINK.c100, tailwind::PINK.c200, tailwind::PINK.c300, tailwind::PINK.c400, tailwind::PINK.c500, tailwind::PINK.c600, tailwind::PINK.c700, tailwind::PINK.c800, tailwind::PINK.c900, tailwind::PINK.c950],
    [tailwind::ROSE.c50, tailwind::ROSE.c100, tailwind::ROSE.c200, tailwind::ROSE.c300, tailwind::ROSE.c400, tailwind::ROSE.c500, tailwind::ROSE.c600, tailwind::ROSE.c700, tailwind::ROSE.c800, tailwind::ROSE.c900, tailwind::ROSE.c950],
];

pub const PALETTE_NAMES: [&str; 22] = [
    "Slate", "Gray", "Zinc", "Neutral", "Stone", "Red", "Orange", "Amber", "Yellow", "Lime",
    "Green", "Emerald", "Teal", "Cyan", "Sky", "Blue", "Indigo", "Violet", "Purple", "Fuchsia",
    "Pink", "Rose",
];

pub const SHADE_NAMES: [&str; 11] = ["50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950"];
