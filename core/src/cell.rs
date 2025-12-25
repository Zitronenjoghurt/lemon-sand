#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    type_: CellType,
    pub moisture: f32,
    pub last_updated: u8,
}

impl Cell {
    pub fn new(type_: CellType) -> Self {
        Self {
            type_,
            moisture: type_.inherent_wetness(),
            last_updated: 0,
        }
    }

    pub fn get_type(&self) -> CellType {
        self.type_
    }

    pub fn color_rgba(&self) -> [u8; 4] {
        match self.type_ {
            CellType::Empty => [0, 0, 0, 255],
            CellType::Sand => [
                lerp_u8(210, 153, self.moisture / self.moisture_capacity()),
                lerp_u8(170, 125, self.moisture / self.moisture_capacity()),
                lerp_u8(109, 81, self.moisture / self.moisture_capacity()),
                255,
            ],
            CellType::Water => [109, 109, 210, 255],
        }
    }

    pub fn movement(&self) -> CellMovement {
        match self.get_type() {
            CellType::Empty => CellMovement::None,
            CellType::Sand => CellMovement::Powder,
            CellType::Water => CellMovement::Liquid,
        }
    }

    pub fn density(&self) -> u8 {
        match self.get_type() {
            CellType::Empty => 0,
            CellType::Sand => 10,
            CellType::Water => 1,
        }
    }

    /// How much moisture a cell can hold before it starts to propagate it.
    pub fn moisture_capacity(&self) -> f32 {
        match self.get_type() {
            CellType::Empty => 0.0,
            CellType::Sand => 2.0,
            CellType::Water => 0.0,
        }
    }

    /// How fast moisture can propagate from/to a cell.
    pub fn moisture_diffusion_rate(&self) -> f32 {
        match self.get_type() {
            CellType::Empty => 0.0,
            CellType::Sand => 0.025,
            CellType::Water => 1.0,
        }
    }

    /// How much moisture can be removed from a cell right now.
    pub fn moisture_diffuse_potential(&self) -> f32 {
        if self.moisture == 0.0 {
            return 0.0;
        }

        if self.moisture > self.moisture_diffusion_rate() {
            self.moisture_diffusion_rate()
        } else {
            self.moisture
        }
    }

    /// How much moisture can be added to a cell right now.
    pub fn moisture_accept_potential(&self) -> f32 {
        if self.moisture_capacity() == 0.0 {
            return 0.0;
        }

        let raw_potential = self.moisture_capacity() - self.moisture;
        if raw_potential <= 0.0 {
            return 0.0;
        }

        if raw_potential > self.moisture_diffusion_rate() {
            self.moisture_diffusion_rate()
        } else {
            raw_potential
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.get_type(), CellType::Empty)
    }

    pub fn is_water(&self) -> bool {
        matches!(self.get_type(), CellType::Water)
    }

    pub fn sand() -> Self {
        Self::new(CellType::Sand)
    }

    pub fn water() -> Self {
        Self::new(CellType::Water)
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CellType {
    #[default]
    Empty,
    Sand,
    Water,
}

impl CellType {
    pub fn inherent_wetness(&self) -> f32 {
        match self {
            CellType::Empty => 0.0,
            CellType::Sand => 0.0,
            CellType::Water => 1.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum CellMovement {
    #[default]
    None,
    Powder,
    Liquid,
    Gas,
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}
