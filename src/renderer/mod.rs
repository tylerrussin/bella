pub mod rasterizer;
pub mod clipping;
pub mod lighting;
pub mod renderer;

pub use rasterizer::{
    reset_screen,
    fill_triangle,
};
pub use clipping::triangle_clip_against_plane;
pub use lighting::calculate_lighting;
pub use renderer::render_screen;