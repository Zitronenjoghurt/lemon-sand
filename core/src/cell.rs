#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    type_: CellType,
    pub last_updated: u8,
}

impl Cell {
    pub fn new(type_: CellType) -> Self {
        Self {
            type_,
            last_updated: 0,
        }
    }

    pub fn get_type(&self) -> CellType {
        self.type_
    }

    pub fn color_rgba(&self) -> [u8; 4] {
        match self.type_ {
            CellType::Empty => [0, 0, 0, 255],
            CellType::Sand => [210, 170, 109, 255],
            CellType::Water => [109, 109, 210, 255],
            CellType::WetSand => [153, 125, 81, 255],
        }
    }

    pub fn movement(&self) -> CellMovement {
        match self.get_type() {
            CellType::Empty => CellMovement::None,
            CellType::Sand => CellMovement::Powder,
            CellType::Water => CellMovement::Liquid,
            CellType::WetSand => CellMovement::Powder,
        }
    }

    pub fn density(&self) -> u8 {
        match self.get_type() {
            CellType::Empty => 0,
            CellType::Sand => 10,
            CellType::Water => 1,
            CellType::WetSand => 15,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.get_type(), CellType::Empty)
    }

    pub fn sand() -> Self {
        Self {
            type_: CellType::Sand,
            ..Default::default()
        }
    }

    pub fn water() -> Self {
        Self {
            type_: CellType::Water,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CellType {
    #[default]
    Empty,
    Sand,
    Water,
    WetSand,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum CellMovement {
    #[default]
    None,
    Powder,
    Liquid,
    Gas,
}
