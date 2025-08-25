// maze.rs 

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    if !Path::new(filename).exists() {
        eprintln!("WARNING: File '{}' not found, creating default maze", filename);
        return create_default_maze();
    }

    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let maze: Maze = reader
                .lines()
                .map(|line| line.unwrap_or_default().chars().collect())
                .collect();
            
            if maze.is_empty() {
                eprintln!("WARNING: Empty maze file, using default maze");
                create_default_maze()
            } else {
                println!("Successfully loaded maze from '{}'", filename);
                maze
            }
        },
        Err(e) => {
            eprintln!("ERROR loading '{}': {}", filename, e);
            eprintln!("Using default maze instead");
            create_default_maze()
        }
    }
}

fn create_default_maze() -> Maze {
    let maze_str = r#"+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
|                                                                 |
|  +--+--+  +--+--+--+  +--+--+--+  +--+--+--+  +--+--+--+--+  |
|  |     |  |        |  |        |  |        |  |              |
|  |  M  |  |  +--+  +--+  +--+  +--+  +--+  |  |  +--+--+--+  |
|  |     |  |  |     |     |  |     |  |  |  |  |  |           |
|  +--+--+  |  |  +--+  +--+  |  M  |  +--+  |  +--+  +--+--+  |
|           |  |  |     |     |     |        |        |        |
|  +--+--+--+  |  +--+  |  +--+--+  +--+--+--+--+--+  |  +--+  |
|  |        |  |        |                             |  |  |  |
|  |  +--+  |  +--+--+--+--+--+--+  +--+--+--+--+  +--+  |  |  |
|  |  |  |  |                    |  |           |  |     |  |  |
|  +--+  |  +--+  +--+--+--+--+  |  |  +--+--+  |  |  +--+  |  |
|        |        |           |  |  |  |     |  |  |  |     |  |
|  +--+--+--+--+  |  M  +--+  |  |  +--+  +--+  |  +--+  +--+  |
|  |           |  |     |  |  |  |        |     |        |     |
|  |  +--+--+  |  +--+--+  |  |  +--+--+--+  +--+--+--+--+  +--+
|  |        |  |           |  |                             |   |
|  +--+--+  |  +--+--+--+--+  +--+--+--+--+--+--+--+--+--+  |  |
|        |  |                                                |  |
+--+--+  |  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+  |
|        |                                                      |
|  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+  |
|                                                               g|
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+"#;

    maze_str
        .lines()
        .map(|line| line.chars().collect())
        .collect()
}