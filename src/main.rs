use shader_art_rs::ui;

fn main() {
    pollster::block_on(ui::render());
}
