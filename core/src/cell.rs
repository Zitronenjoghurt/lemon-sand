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
                lerp_u8(
                    245,
                    195,
                    self.moisture / self.property_capacity(CellProperty::Moisture),
                ),
                lerp_u8(
                    237,
                    174,
                    self.moisture / self.property_capacity(CellProperty::Moisture),
                ),
                lerp_u8(
                    190,
                    142,
                    self.moisture / self.property_capacity(CellProperty::Moisture),
                ),
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

    pub fn get_property(&self, property: CellProperty) -> f32 {
        match property {
            CellProperty::Moisture => self.moisture,
        }
    }

    pub fn set_property(&mut self, property: CellProperty, value: f32) {
        match property {
            CellProperty::Moisture => self.moisture = value,
        }
    }

    /// How much of a property a cell can hold.
    pub fn property_capacity(&self, property: CellProperty) -> f32 {
        match property {
            CellProperty::Moisture => match self.get_type() {
                CellType::Empty => 0.0,
                CellType::Sand => 1.5,
                CellType::Water => 0.0,
            },
        }
    }

    /// How much of a property a cell wants to hold before propagating excess.
    pub fn property_min_saturation(&self, property: CellProperty) -> f32 {
        match property {
            CellProperty::Moisture => match self.get_type() {
                CellType::Empty => 0.0,
                CellType::Sand => 0.5,
                CellType::Water => 0.0,
            },
        }
    }

    /// How fast the property can propagate out of a cell.
    pub fn property_diffusion_rate(&self, property: CellProperty) -> f32 {
        match property {
            CellProperty::Moisture => match self.get_type() {
                CellType::Empty => 0.0,
                CellType::Sand => 0.01,
                CellType::Water => 1.0,
            },
        }
    }

    /// How fast the property can propagate into a cell.
    pub fn property_accept_rate(&self, property: CellProperty) -> f32 {
        match property {
            CellProperty::Moisture => match self.get_type() {
                CellType::Empty => 0.0,
                CellType::Sand => 0.05,
                CellType::Water => 0.0,
            },
        }
    }

    /// How much of the property can be removed from this cell right now.
    pub fn property_diffuse_potential(&self, property: CellProperty) -> f32 {
        let value = self.get_property(property);

        if value == 0.0 {
            return 0.0;
        }

        let diffusion_rate = self.property_diffusion_rate(property);
        if value > diffusion_rate {
            diffusion_rate
        } else {
            self.moisture
        }
    }

    /// How much of the property can be added to this cell right now.
    pub fn property_accept_potential(&self, property: CellProperty) -> f32 {
        let value = self.get_property(property);

        let raw_potential = self.property_capacity(property) - value;
        if raw_potential <= 0.0 {
            return 0.0;
        }

        let accept_rate = self.property_accept_rate(property);
        if raw_potential > accept_rate {
            accept_rate
        } else {
            raw_potential
        }
    }

    pub fn is_pure_source(&self, property: CellProperty) -> bool {
        match property {
            CellProperty::Moisture => matches!(self.get_type(), CellType::Water),
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

#[derive(Debug, Clone, Copy)]
pub enum CellProperty {
    Moisture,
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}
