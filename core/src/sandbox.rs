use crate::cell::{Cell, CellMovement};
use crate::reactions::REACTION_TABLE;

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
                self.try_reactions(x, y);
                self.update_movement(x, y);
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
    fn try_reactions(&mut self, x: isize, y: isize) {
        let Some(cell) = self.get(x, y) else { return };

        const NEIGHBORS: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in NEIGHBORS {
            let nx = x + dx;
            let ny = y + dy;

            let Some(neighbor) = self.get(nx, ny) else {
                continue;
            };

            if let Some(reactions) =
                REACTION_TABLE.get_reactions(cell.get_type(), neighbor.get_type())
            {
                for reaction in reactions {
                    if fastrand::f32() > reaction.probability {
                        continue;
                    }

                    if let Some(cond) = reaction.condition
                        && !cond(&cell, &neighbor)
                    {
                        continue;
                    }

                    if let Some(new_type) = reaction.products.0 {
                        self.set(x, y, Cell::new(new_type));
                        if let Some(i) = self.coords_to_index(x, y) {
                            self.updated[i] = true;
                        }
                    } else {
                        self.set(x, y, Cell::default());
                    }

                    if let Some(new_type) = reaction.products.1 {
                        self.set(nx, ny, Cell::new(new_type));
                        if let Some(i) = self.coords_to_index(nx, ny) {
                            self.updated[i] = true;
                        }
                    } else {
                        self.set(nx, ny, Cell::default());
                    }

                    return;
                }
            }
        }
    }

    fn update_movement(&mut self, x: isize, y: isize) {
        let Some(cell) = self.get(x, y) else { return };

        match cell.movement() {
            CellMovement::None => {}
            CellMovement::Powder => self.move_powder(x, y, cell),
            CellMovement::Liquid => self.move_liquid(x, y, cell),
            CellMovement::Gas => self.move_gas(x, y, cell),
        }
    }

    fn move_powder(&mut self, x: isize, y: isize, cell: Cell) {
        let d = cell.density();
        let target = if self.can_displace(x, y + 1, d) {
            (x, y + 1)
        } else {
            let (dx1, dx2) = if fastrand::bool() { (-1, 1) } else { (1, -1) };
            if self.can_displace(x + dx1, y + 1, d) {
                (x + dx1, y + 1)
            } else if self.can_displace(x + dx2, y + 1, d) {
                (x + dx2, y + 1)
            } else {
                return;
            }
        };
        self.swap_cells((x, y), target);
    }

    fn move_liquid(&mut self, x: isize, y: isize, cell: Cell) {
        let d = cell.density();

        if self.can_displace(x, y + 1, d) {
            self.swap_cells((x, y), (x, y + 1));
            return;
        }

        let (dx1, dx2) = if fastrand::bool() { (-1, 1) } else { (1, -1) };
        if self.can_displace(x + dx1, y + 1, d) {
            self.swap_cells((x, y), (x + dx1, y + 1));
            return;
        }
        if self.can_displace(x + dx2, y + 1, d) {
            self.swap_cells((x, y), (x + dx2, y + 1));
            return;
        }

        if self.can_displace(x + dx1, y, d) {
            self.swap_cells((x, y), (x + dx1, y));
        } else if self.can_displace(x + dx2, y, d) {
            self.swap_cells((x, y), (x + dx2, y));
        }
    }

    fn move_gas(&mut self, x: isize, y: isize, cell: Cell) {
        let d = cell.density();

        if self.can_displace(x, y - 1, d) {
            self.swap_cells((x, y), (x, y - 1));
            return;
        }

        let (dx1, dx2) = if fastrand::bool() { (-1, 1) } else { (1, -1) };
        if self.can_displace(x + dx1, y, d) {
            self.swap_cells((x, y), (x + dx1, y));
        } else if self.can_displace(x + dx2, y, d) {
            self.swap_cells((x, y), (x + dx2, y));
        }
    }
}
