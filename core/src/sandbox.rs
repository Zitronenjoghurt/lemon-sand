use crate::cell::{Cell, CellMovement, CellProperty};

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

        self.update_property(x, y, CellProperty::Moisture);
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

    fn update_property(&mut self, x: isize, y: isize, property: CellProperty) {
        let Some(source) = self.get(x, y) else { return };
        if source.is_empty() {
            return;
        }

        if source.get_property(property) < source.property_min_saturation(property) {
            return;
        }

        let mut candidates = [
            (x, y + 1),
            (x - 1, y + 1),
            (x + 1, y + 1),
            (x - 1, y),
            (x + 1, y),
            (x, y - 1),
            (x - 1, y - 1),
            (x + 1, y - 1),
        ];

        fastrand::shuffle(&mut candidates);
        self.try_spread_property(x, y, &candidates, property);
    }

    fn try_spread_property(
        &mut self,
        x: isize,
        y: isize,
        candidates: &[(isize, isize)],
        property: CellProperty,
    ) -> bool {
        let Some(source) = self.get(x, y) else {
            return false;
        };

        let source_value = source.get_property(property);
        let source_is_pure = source.is_pure_source(property);

        let Some(&(tx, ty)) = candidates.iter().find(|&&(tx, ty)| {
            self.get(tx, ty)
                .map(|t| {
                    !t.is_empty() && source_is_pure
                        || (t.get_property(property) < source_value
                            && t.property_accept_potential(property) > 0.0)
                })
                .unwrap_or(false)
        }) else {
            return false;
        };

        let Some(target) = self.get(tx, ty) else {
            return false;
        };

        let diffuse = source.property_diffuse_potential(property);
        let accept = target.property_accept_potential(property);
        let transfer = diffuse.min(accept);

        if let Some(target) = self.get_mut(tx, ty) {
            target.set_property(property, target.get_property(property) + transfer);
        }

        if let Some(source) = self.get_mut(x, y) {
            source.set_property(property, source.get_property(property) - transfer);
        }

        self.check_depletion(x, y, property);

        true
    }

    fn check_depletion(&mut self, x: isize, y: isize, property: CellProperty) {
        let Some(cell) = self.get(x, y) else { return };

        if cell.is_pure_source(property) && cell.get_property(property) <= 0.05 {
            self.place(x, y, Cell::default());
        }
    }
}
