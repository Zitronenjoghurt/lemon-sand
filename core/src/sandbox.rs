use crate::cell::{Cell, CellMovement};

pub struct Sandbox {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    update_counter: u8,
}

impl Sandbox {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            width,
            height,
            update_counter: 0,
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

    fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut Cell> {
        self.coords_to_index(x, y).map(|i| &mut self.cells[i])
    }

    fn can_displace(&self, cell: Cell, to: (isize, isize)) -> bool {
        let Some(to) = self.get(to.0, to.1) else {
            return false;
        };
        cell.density() > to.density()
    }

    fn swap_cells(&mut self, from: (isize, isize), to: (isize, isize)) {
        let Some(i) = self.coords_to_index(from.0, from.1) else {
            return;
        };

        let Some(j) = self.coords_to_index(to.0, to.1) else {
            return;
        };

        self.cells[i].last_updated = self.update_counter;
        self.cells[j].last_updated = self.update_counter;
        self.cells.swap(i, j);
    }

    #[tracing::instrument(skip_all)]
    pub fn update(&mut self) {
        self.update_counter = self.update_counter.wrapping_add(1);
        for y in (0..self.height as isize).rev() {
            let scan_right = fastrand::bool();
            for i in 0..self.width as isize {
                let x = if scan_right {
                    i
                } else {
                    (self.width as isize - 1) - i
                };
                self.update_cell(x, y);
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            pixel.copy_from_slice(&cell.color_rgba());
        }
    }

    pub fn place(&mut self, x: isize, y: isize, cell: Cell) {
        if let Some(index) = self.coords_to_index(x, y) {
            self.cells[index] = cell;
        }
    }
}

// Cell Updates
impl Sandbox {
    fn update_cell(&mut self, x: isize, y: isize) {
        let Some(cell) = self.get(x, y) else {
            return;
        };

        if cell.is_empty() || cell.last_updated == self.update_counter {
            return;
        }

        self.update_moisture(x, y);
        self.update_movement(x, y);
    }

    fn update_movement(&mut self, x: isize, y: isize) {
        let Some(cell) = self.get(x, y) else {
            return;
        };

        match cell.movement() {
            CellMovement::None | CellMovement::Gas => {}
            CellMovement::Powder => self.move_powder(cell, x, y),
            CellMovement::Liquid => self.move_liquid(cell, x, y),
        }
    }

    fn move_powder(&mut self, cell: Cell, x: isize, y: isize) {
        let target = if self.can_displace(cell, (x, y + 1)) {
            (x, y + 1)
        } else {
            let (dx1, dx2) = if fastrand::bool() { (-1, 1) } else { (1, -1) };
            if self.can_displace(cell, (x + dx1, y + 1)) {
                (x + dx1, y + 1)
            } else if self.can_displace(cell, (x + dx2, y + 1)) {
                (x + dx2, y + 1)
            } else {
                return;
            }
        };
        self.swap_cells((x, y), target);
    }

    fn move_liquid(&mut self, cell: Cell, x: isize, y: isize) {
        if self.can_displace(cell, (x, y + 1)) {
            self.swap_cells((x, y), (x, y + 1));
            return;
        }

        let (dx1, dx2) = if fastrand::bool() { (-1, 1) } else { (1, -1) };
        if self.can_displace(cell, (x + dx1, y + 1)) {
            self.swap_cells((x, y), (x + dx1, y + 1));
            return;
        };
        if self.can_displace(cell, (x + dx2, y + 1)) {
            self.swap_cells((x, y), (x + dx2, y + 1));
            return;
        };

        if self.can_displace(cell, (x + dx1, y)) {
            self.swap_cells((x, y), (x + dx1, y));
        } else if self.can_displace(cell, (x + dx2, y)) {
            self.swap_cells((x, y), (x + dx2, y));
        }
    }

    fn update_moisture(&mut self, x: isize, y: isize) {
        let Some(source) = self.get(x, y) else { return };
        if source.moisture < source.moisture_capacity() {
            return;
        }

        let (dx1, dx2) = if fastrand::bool() { (-1, 1) } else { (1, -1) };
        let down_candidates = if fastrand::bool() {
            [(x, y + 1), (x + dx1, y + 1), (x + dx2, y + 1)]
        } else {
            [(x + dx1, y + 1), (x, y + 1), (x + dx2, y + 1)]
        };

        self.try_spread_moisture(x, y, &down_candidates, 1.0);

        let side_up_candidates = if fastrand::bool() {
            [
                (x + dx1, y),
                (x + dx2, y),
                (x, y - 1),
                (x + dx1, y - 1),
                (x + dx2, y - 1),
            ]
        } else {
            [
                (x, y - 1),
                (x + dx1, y - 1),
                (x + dx2, y - 1),
                (x + dx1, y),
                (x + dx2, y),
            ]
        };

        self.try_spread_moisture(x, y, &side_up_candidates, 1.0);
    }

    fn try_spread_moisture(
        &mut self,
        x: isize,
        y: isize,
        candidates: &[(isize, isize)],
        efficiency: f32,
    ) -> bool {
        let Some(source) = self.get(x, y) else {
            return false;
        };

        let Some(&(tx, ty)) = candidates.iter().find(|&&(tx, ty)| {
            self.get(tx, ty)
                .map(|t| {
                    source.is_water()
                        || (t.moisture < source.moisture && t.moisture_accept_potential() > 0.0)
                })
                .unwrap_or(false)
        }) else {
            return false;
        };

        let Some(target) = self.get(tx, ty) else {
            return false;
        };

        let diffuse = source.moisture_diffuse_potential() * efficiency;
        let accept = target.moisture_accept_potential();
        let transfer = diffuse.min(accept);

        if let Some(target) = self.get_mut(tx, ty) {
            target.moisture += transfer;
        }

        if let Some(source) = self.get_mut(x, y) {
            source.moisture -= transfer;
            if source.is_water() && source.moisture <= 0.05 {
                self.place(x, y, Cell::default());
            }
        }

        true
    }
}
