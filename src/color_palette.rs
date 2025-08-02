pub enum ColorPalette {
    Party,
    Pastel,
    Earth,
    Neon,
    Cool,
    Sunset,
    Ocean,
    Retro,
    Forest,
    Candy,
}

type ColorVec = Vec<[f32; 3]>;
impl ColorPalette {
    pub fn get_colors(&self) -> Vec<[f32; 3]> {
        match &self {
            Self::Party => Self::party(),
            Self::Pastel => Self::pastel(),
            Self::Earth => Self::earth_tones(),
            Self::Neon => Self::neon(),
            Self::Cool => Self::cool(),
            Self::Sunset => Self::sunset(),
            Self::Ocean => Self::ocean(),
            Self::Retro => Self::retro(),
            Self::Forest => Self::forest(),
            Self::Candy => Self::candy(),
        }
    }
    fn party() -> ColorVec {
        vec![
            [1.0, 0.0, 0.0], // bright red
            [1.0, 0.5, 0.0], // vivid orange
            [1.0, 1.0, 0.0], // bright yellow
            [0.0, 1.0, 0.0], // neon green
            [0.0, 1.0, 1.0], // bright cyan
            [0.0, 0.0, 1.0], // electric blue
            [0.7, 0.0, 1.0], // vibrant purple
            [1.0, 0.0, 1.0], // hot pink
            [1.0, 0.2, 0.5], // neon pink
            [1.0, 0.8, 0.0], // gold yellow
        ]
    }

    fn pastel() -> ColorVec {
        vec![
            [1.0, 0.8, 0.8],   // pastel pink
            [0.8, 1.0, 0.8],   // pastel green
            [0.8, 0.8, 1.0],   // pastel blue
            [1.0, 0.9, 0.7],   // cream
            [0.9, 0.8, 1.0],   // lavender
            [1.0, 0.85, 0.85], // light coral
            [0.85, 1.0, 0.85], // mint
            [0.85, 0.85, 1.0], // light periwinkle
        ]
    }

    fn neon() -> ColorVec {
        vec![
            [1.0, 0.1, 0.1], // neon red
            [1.0, 0.5, 0.0], // neon orange
            [1.0, 1.0, 0.0], // neon yellow
            [0.0, 1.0, 0.0], // neon green
            [0.0, 1.0, 1.0], // neon cyan
            [0.0, 0.1, 1.0], // neon blue
            [0.6, 0.0, 1.0], // neon purple
            [1.0, 0.0, 1.0], // neon magenta
        ]
    }

    fn earth_tones() -> ColorVec {
        vec![
            [0.5, 0.3, 0.1], // brown
            [0.6, 0.4, 0.2], // tan
            [0.4, 0.5, 0.3], // olive green
            [0.2, 0.3, 0.1], // dark olive
            [0.8, 0.7, 0.5], // sand
            [0.3, 0.2, 0.1], // dark brown
        ]
    }

    fn cool() -> ColorVec {
        vec![
            [0.0, 0.5, 1.0], // sky blue
            [0.0, 0.7, 0.9], // turquoise
            [0.0, 0.4, 0.6], // teal
            [0.3, 0.6, 0.8], // steel blue
            [0.2, 0.3, 0.5], // navy
            [0.5, 0.7, 0.9], // light blue
        ]
    }
    fn sunset() -> ColorVec {
        vec![
            [1.0, 0.4, 0.0], // orange
            [1.0, 0.7, 0.4], // light orange
            [0.9, 0.2, 0.3], // deep pink
            [0.6, 0.0, 0.3], // maroon
            [0.9, 0.5, 0.1], // gold
            [1.0, 0.3, 0.0], // fiery red-orange
        ]
    }

    fn ocean() -> ColorVec {
        vec![
            [0.0, 0.5, 0.7], // deep sea blue
            [0.0, 0.7, 0.9], // aqua
            [0.2, 0.8, 0.8], // light teal
            [0.0, 0.3, 0.5], // navy blue
            [0.1, 0.6, 0.8], // sky blue
            [0.3, 0.9, 1.0], // bright cyan
        ]
    }

    fn retro() -> ColorVec {
        vec![
            [1.0, 0.3, 0.5], // pink
            [1.0, 0.6, 0.0], // orange
            [0.9, 0.8, 0.2], // mustard yellow
            [0.3, 0.7, 0.6], // teal
            [0.6, 0.3, 0.6], // purple
            [0.8, 0.4, 0.2], // burnt sienna
        ]
    }

    fn forest() -> ColorVec {
        vec![
            [0.0, 0.3, 0.0], // dark green
            [0.1, 0.5, 0.1], // moss green
            [0.2, 0.6, 0.2], // pine green
            [0.4, 0.8, 0.4], // leaf green
            [0.1, 0.4, 0.1], // olive green
            [0.3, 0.5, 0.3], // fern green
        ]
    }

    fn candy() -> ColorVec {
        vec![
            [1.0, 0.7, 0.8], // cotton candy pink
            [1.0, 0.9, 0.6], // pale yellow
            [0.8, 1.0, 0.7], // light lime
            [0.7, 0.8, 1.0], // baby blue
            [1.0, 0.6, 0.7], // bubblegum pink
            [0.9, 0.7, 1.0], // lavender pink
        ]
    }
}
impl std::str::FromStr for ColorPalette {
    type Err = String; // or a custom error type
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "party" => Ok(ColorPalette::Party),
            "pastel" => Ok(ColorPalette::Pastel),
            "earth" => Ok(ColorPalette::Earth),
            "neon" => Ok(ColorPalette::Neon),
            "cool" => Ok(ColorPalette::Cool),
            "sunset" => Ok(ColorPalette::Sunset),
            "ocean" => Ok(ColorPalette::Ocean),
            "retro" => Ok(ColorPalette::Retro),
            "forest" => Ok(ColorPalette::Forest),
            "candy" => Ok(ColorPalette::Candy),
            _ => Err(format!("Unknown color pallette: {}", s)),
        }
    }
}
impl Default for ColorPalette {
    fn default() -> Self {
        Self::Retro
    }
}
