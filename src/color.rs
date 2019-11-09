#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn clear() -> Color {
        Color::new(0, 0, 0, 0)
    }

    pub fn white() -> Color {
        Color::new(255, 255, 255, 255)
    }

    pub fn black() -> Color {
        Color::new(0, 0, 0, 255)
    }

    pub fn parse(string: &str) -> Result<Color, ()> {
        let components = string
            .split(',')
            .map(|s| s.parse::<u8>().map_err(|_| ()))
            .collect::<Result<Vec<u8>, ()>>()?;
        if components.len() == 4 {
            Ok(Color::new(
                components[0],
                components[1],
                components[2],
                components[3],
            ))
        } else if components.len() == 3 {
            Ok(Color::new(components[0], components[1], components[2], 255))
        } else {
            Err(())
        }
    }
}
