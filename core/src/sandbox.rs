use crate::cell::{Cell, CellType};

pub struct Sandbox {
    cells: Vec<Cell>,
    updated: Vec<bool>,
    width: usize,
    height: usize,
}

impl Sandbox {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            updated: vec![false; width * height],
            width,
            height,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    fn coords_to_index(&self, x: isize, y: isize) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
            return None;
        }
        Some((y * self.width as isize + x) as usize)
    }

    fn get(&self, x: isize, y: isize) -> Option<Cell> {
        self.coords_to_index(x, y).map(|i| self.cells[i])
    }

    fn can_displace(&self, x: isize, y: isize, density: u8) -> bool {
        self.get(x, y)
            .map(|c| c.density() < density)
            .unwrap_or(false)
    }

    fn swap_cells(&mut self, a: (isize, isize), b: (isize, isize)) {
        if a == b {
            return;
        }

        let ai = self.coords_to_index(a.0, a.1).unwrap();
        let bi = self.coords_to_index(b.0, b.1).unwrap();
        self.cells.swap(ai, bi);
        self.updated[ai] = true;
        self.updated[bi] = true;
    }

    pub fn update(&mut self) {
        self.updated.fill(false);

        for y in (0..self.height as isize).rev() {
            for x in 0..self.width as isize {
                let i = self.coords_to_index(x, y).unwrap();
                if self.updated[i] {
                    continue;
                }
                self.update_cell(x, y);
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            pixel.copy_from_slice(&cell.color_rgba());
        }
    }

    pub fn set(&mut self, x: isize, y: isize, cell: Cell) {
        if let Some(i) = self.coords_to_index(x, y) {
            self.cells[i] = cell;
        }
    }
}

// Cell Updates
impl Sandbox {
    fn update_cell(&mut self, x: isize, y: isize) {
        let Some(cell) = self.get(x, y) else { return };
        match cell.get_type() {
            CellType::Empty => {}
            CellType::Sand => self.update_sand(x, y, cell),
            CellType::Water => self.update_water(x, y, cell),
        }
    }

    fn update_sand(&mut self, x: isize, y: isize, cell: Cell) {
        self.update_gravity(x, y, cell);
    }

    fn update_water(&mut self, x: isize, y: isize, cell: Cell) {
        self.update_liquid(x, y, cell);
    }

    fn update_liquid(&mut self, x: isize, y: isize, cell: Cell) {
        let d = cell.density();
        let target = if self.can_displace(x, y + 1, d) {
            (x, y + 1)
        } else if fastrand::bool() {
            if self.can_displace(x - 1, y, d) {
                (x - 1, y)
            } else if self.can_displace(x + 1, y, d) {
                (x + 1, y)
            } else {
                (x, y)
            }
        } else {
            if self.can_displace(x + 1, y, d) {
                (x + 1, y)
            } else if self.can_displace(x - 1, y, d) {
                (x - 1, y)
            } else {
                (x, y)
            }
        };

        self.swap_cells((x, y), target);
    }

    fn update_gravity(&mut self, x: isize, y: isize, cell: Cell) {
        let d = cell.density();
        let target = if self.can_displace(x, y + 1, d) {
            (x, y + 1)
        } else if self.can_displace(x - 1, y + 1, d) {
            (x - 1, y + 1)
        } else if self.can_displace(x + 1, y + 1, d) {
            (x + 1, y + 1)
        } else {
            (x, y)
        };
        self.swap_cells((x, y), target);
    }
}
