use crate::cell::{Cell, CellMovement, CellProperty};

pub struct Sandbox {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    gravity: f32,
    max_velocity: f32,
    update_counter: u8,
}

impl Sandbox {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            width,
            height,
            gravity: 0.3,
            max_velocity: 8.0,
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

    pub fn get(&self, x: isize, y: isize) -> Option<Cell> {
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
        let Some(cell) = self.get(x, y) else { return };

        match cell.movement() {
            CellMovement::None => {}
            CellMovement::Powder => self.move_with_velocity(x, y),
            CellMovement::Liquid => self.move_with_velocity(x, y),
            CellMovement::Gas => {}
        }
    }

    fn move_with_velocity(&mut self, x: isize, y: isize) {
        let Some(mut cell) = self.get(x, y) else {
            return;
        };

        cell.vy = (cell.vy + self.gravity * cell.gravity_factor())
            .clamp(-self.max_velocity, self.max_velocity);

        let mut current = (x, y);

        let vy_dir = cell.vy.signum() as isize;
        for _ in 0..cell.vy.abs().floor() as usize {
            let next = (current.0, current.1 + vy_dir);
            if self.can_displace(cell, next) {
                self.swap_cells(current, next);
                current = next;
            } else {
                self.push_slide_down(&mut cell, current, vy_dir);
                self.push_blocker_vertical(current, next, cell.vy * 0.5);
                cell.vy *= 0.5;
                break;
            }
        }

        let vx_dir = cell.vx.signum() as isize;
        for _ in 0..cell.vx.abs().floor() as usize {
            let next = (current.0 + vx_dir, current.1);
            if self.can_displace(cell, next) {
                self.swap_cells(current, next);
                current = next;
            } else {
                self.push_blocker_horizontal(current, cell.vx * 0.5);
                cell.vx *= 0.5;
                break;
            }
        }

        let surface_friction = self
            .get(current.0, current.1 + 1)
            .map(|below| below.slide_speed_factor())
            .unwrap_or(0.5);
        cell.vx *= surface_friction;

        if let Some(c) = self.get_mut(current.0, current.1) {
            c.vx = cell.vx;
            c.vy = cell.vy;
        }
    }

    fn push_slide_down(&self, cell: &mut Cell, pos: (isize, isize), vy_dir: isize) {
        let Some(blocker) = self.get(pos.0, pos.1 + vy_dir) else {
            return;
        };

        let slide = cell.slide_speed_factor() * blocker.slide_speed_factor();
        let transfer = cell.vy.abs() * slide;

        if let Some(dir) = self.find_open_direction(*cell, pos, vy_dir) {
            cell.vx += transfer * dir as f32;
            return;
        }

        if cell.spread_impulse() > 0.0
            && let Some(dir) = self.find_open_direction(*cell, pos, 0)
        {
            cell.vx += cell.spread_impulse() * dir as f32;
        }
    }

    fn push_blocker_vertical(&mut self, _from: (isize, isize), to: (isize, isize), impulse: f32) {
        let Some(blocker) = self.get_mut(to.0, to.1) else {
            return;
        };
        if blocker.is_empty() {
            return;
        }

        blocker.vy += impulse;
        blocker.vx += impulse * 0.2 * if fastrand::bool() { 1.0 } else { -1.0 };
    }

    fn push_blocker_horizontal(&mut self, to: (isize, isize), impulse: f32) {
        let Some(blocker) = self.get_mut(to.0, to.1) else {
            return;
        };

        if blocker.is_empty() {
            return;
        }

        blocker.vx += impulse;
    }

    fn find_open_direction(&self, cell: Cell, pos: (isize, isize), dy: isize) -> Option<isize> {
        let left = self.can_displace(cell, (pos.0 - 1, pos.1 + dy));
        let right = self.can_displace(cell, (pos.0 + 1, pos.1 + dy));

        match (left, right) {
            (true, true) => Some(if fastrand::bool() { 1 } else { -1 }),
            (true, false) => Some(-1),
            (false, true) => Some(1),
            (false, false) => None,
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
