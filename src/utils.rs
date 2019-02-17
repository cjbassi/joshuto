use std::path::{Path, PathBuf};

// searches a list of folders for a given file in order of preference
pub fn search_directories<P>(filename: &str, directories: &[P]) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    for path in directories.iter() {
        let filepath = path.as_ref().join(filename);
        if filepath.exists() {
            return Some(filepath);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub min: Point,
    pub max: Point,
}

impl Rectangle {
    pub fn new(min: Point, max: Point) -> Rectangle {
        Rectangle { min, max }
    }
    pub fn width(&self) -> u32 {
        (self.max.x - self.min.x) as u32
    }
    pub fn height(&self) -> u32 {
        (self.max.y - self.min.y) as u32
    }
}
