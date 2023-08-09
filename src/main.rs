use shader_art_rs::cli;
use shader_art_rs::ui;
use spinoff::{spinners, Color, Spinner};

use anyhow::{Context, Result};

use clap::crate_version;

fn main() -> Result<()> {
    let matches = cli::cli().version(crate_version!()).get_matches();

    let mut animation_speed: u8 = 1;

    if let Some(speed) = matches.get_one::<u8>("speed") {
        animation_speed = *speed;
    }

    if let Some(filename) = matches.get_one::<String>("save") {
        let animation_filename = filename;
        let mut animation_resolution = [512, 512];
        animation_speed = 10;

        if let Some(resolution) = matches.get_one::<String>("resolution") {
            let parts: Vec<&str> = resolution.split('x').collect();
            let width: u16 = parts[0].parse().with_context(|| "Invalid width value")?;
            let height: u16 = parts[1].parse().with_context(|| "Invalid height value")?;

            animation_resolution = [width, height];
        }

        let mut spinner = Spinner::new(spinners::Dots, "Generating...", Color::White);
        pollster::block_on(ui::run(
            animation_speed,
            animation_filename,
            animation_resolution,
        ));
        spinner.success(format!("The animation is saved as `{}`", animation_filename).as_str());
    } else {
        pollster::block_on(ui::render(animation_speed));
    }

    Ok(())
}
