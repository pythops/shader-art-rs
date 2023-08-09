use clap::{arg, Command};

pub fn cli() -> Command {
    Command::new("shader-art")
        .about("Shader Art")
        .arg(arg!(--save <filename>).help("Save the animation as GIF."))
        .arg(
            arg!(--speed <speed>)
                .help("The animation speed.")
                .value_parser(clap::value_parser!(u8)),
        )
        .arg(
            arg!(--resolution <resolution>)
                .requires("save")
                .help("The animation resolution."),
        )
}
