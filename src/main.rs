#[macro_use]
extern crate clap;
extern crate sdl2;

mod yuv;
mod sdlui;

use std::process::exit;

use sdlui::SdlUi;

pub fn main() {
    let matches = clap_app!(yuvdiff =>
        (version: "0.1")
        (about: "Diff YUV files")
        (@arg WIDTH: -w --width +takes_value +required "Width")
        (@arg HEIGHT: -h --height +takes_value +required "Height")
        (@arg CHANNEL: -c --channel +takes_value "Channel (y, u, v, c)")
        (@arg VIEW: -v --view +takes_value "View (a, b, d)")
        (@arg FILEA: +required "YUV file A")
        (@arg FILEB: +required "YUV file B")
        (@arg MULTIPLIER: -m --multiplier +takes_value "Diff multiplier (default: 5)")
    )
        .get_matches();

    let width: u32 = matches.value_of("WIDTH").unwrap().parse().unwrap_or_else(|e| {
        println!("Invalid width: {}", e);
        exit(1)
    });

    let height: u32 = matches.value_of("HEIGHT").unwrap().parse().unwrap_or_else(|e| {
        println!("Invalid height: {}", e);
        exit(1)
    });

    let multiplier: u32 =
        matches.value_of("MULTIPLIER").unwrap_or("5").parse().unwrap_or_else(|e| {
            println!("Invalid multiplier: {}", e);
            exit(1)
        });

    let file_a = matches.value_of("FILEA").unwrap();
    let file_b = matches.value_of("FILEB").unwrap();

    let view = matches.value_of("VIEW").unwrap_or("a");
    let channel = matches.value_of("CHANNEL").unwrap_or("c");

    let mut ui_handle = SdlUi::new(width, height, file_a, file_b).unwrap_or_else(|e| {
        println!("{}", e);
        exit(1)
    });

    ui_handle.set_diff_multiplier(multiplier);

    ui_handle.set_view(view).unwrap_or_else(|e| {
        println!("{}", e);
        exit(1)
    });

    ui_handle.set_channel(channel).unwrap_or_else(|e| {
        println!("{}", e);
        exit(1)
    });

    ui_handle.run();
}
