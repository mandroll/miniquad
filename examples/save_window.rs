/// Example of saving window position to enable "hot reloading".
/// 
/// Run this example with cargo watch -x "run --example save_window" to see the effect.
/// 
/// Could also save window size, fullscreen state, game state, etc.
/// 
use miniquad::*;
use miniquad::window::*;
use std::fs;
use std::io::Read;

struct Stage { 
    position: (u32, u32),
}

/// Read the window position from a file if it exists.
/// Manually parsing file to avoid adding dependencies.
fn read_window_position() -> Option<(u32, u32)> {
    let mut file = match fs::File::open("target/window_position.txt") {
        Ok(file) => file,
        Err(_err) => {
            return None;
        }
    };

    let mut contents = String::new();
    if let Err(_err) = file.read_to_string(&mut contents) {
        return None;
    }

    let position: Vec<u32> = contents
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    println!("Read window position: {:?}", position);

    if position.len() == 2 {
        Some((position[0], position[1]))
    } else {
        None
    }
}

impl Stage {
    fn new() -> Stage {
        // Load the window position from a file if it exists.
        let (x, y) = read_window_position().unwrap_or((0, 0));
        set_window_position(x, y);
        // read position from a text file if it exists
        Stage { position: (x, y)}
    }
}

impl EventHandler for Stage {

    fn update(&mut self) { 
        let position = get_window_position();

        if position == self.position {
            return;
        }

        // write the position to a file
        println!("Updating window_position.txt with new position {:?}", position);
        std::fs::write("target/window_position.txt", format!("{:?},{:?}", position.0, position.1)).unwrap();
        self.position = position;
    }

    fn draw(&mut self) { }

}

fn main() {
    miniquad::start(conf::Conf::default(), || Box::new(Stage::new()));
}
