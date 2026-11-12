use crate::surface::GfxError;

const VERT_PREAMBLE: &str = "#version 330 core\n#define attribute in\n#define varying out\n";

const FRAG_PREAMBLE: &str = "#version 330 core\n#define varying in\n\
                             #define texture2D texture\nout vec4 FRAG_COLOR;\n";

const VERT_PREAMBLE_ES: &str = "";
const FRAG_PREAMBLE_ES: &str = "#define FRAG_COLOR gl_FragColor\n";

pub fn program(vert: &str, frag: &str) -> Result<gl::types::GLuint, GfxError> {
    let (vp, fp) = match crate::gl::es() {
        true => (VERT_PREAMBLE_ES, FRAG_PREAMBLE_ES),
        false => (VERT_PREAMBLE, FRAG_PREAMBLE),
    };
    crate::gl::program(&format!("{vp}{vert}"), &format!("{fp}{frag}"))
}

pub const RECT_VERT: &str = include_str!("../shaders/game.vert.glsl");

pub const SPRITE_VERT: &str = include_str!("../shaders/sprite.vert.glsl");

pub const GAME_FRAG: &str = concat!(
    include_str!("../shaders/game.frag.glsl"),
    include_str!("../shaders/effects/dark_gritty_retro.glsl"),
    include_str!("../shaders/effects/lcd_grid.glsl"),
    include_str!("../shaders/effects/shimmerless.glsl"),
    include_str!("../shaders/effects/simpletex.glsl"),
    include_str!("../shaders/effects/dot.glsl"),
    include_str!("../shaders/effects/grid.glsl"),
    include_str!("../shaders/effects/lcd3x.glsl"),
);

pub const SPRITE_FRAG: &str = include_str!("../shaders/sprite.frag.glsl");

pub const PAPER_VERT: &str = include_str!("../shaders/paper.vert.glsl");

pub const PAPER_FRAG: &str = include_str!("../shaders/paper.frag.glsl");

pub const BLIT_VERT: &str = include_str!("../shaders/blit.vert.glsl");

pub const TURN_VERT: &str = include_str!("../shaders/turn.vert.glsl");

pub const BLIT_FRAG: &str = include_str!("../shaders/blit.frag.glsl");

pub const GHOST_FRAG: &str = include_str!("../shaders/ghost.frag.glsl");
