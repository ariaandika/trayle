use std::{fs, io::Read, time::Duration};

use anyhow::{Context, Result};
use xcursor::{parser::Image, CursorTheme};

static FALLBACK_CURSOR_DATA: &[u8] = include_bytes!("../resources/cursor.rgba");

pub struct Cursor {
    icons: Vec<Image>,
    size: u32,
}

impl Cursor {
    pub fn load() -> Cursor {
        let name = std::env::var("XCURSOR_THEME")
            .ok()
            .unwrap_or_else(||"default".into());
        let size = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s|s.parse().ok())
            .unwrap_or(24);

        let theme = CursorTheme::load(&name);
        let icons = match load_icon(&theme) {
            Ok(ok) => ok,
            Err(err) => {
                tracing::warn!("failed to load xcursor: {err}, using fallback");
                vec![Image {
                    size: 32,
                    width: 64,
                    height: 64,
                    xhot: 1,
                    yhot: 1,
                    delay: 1,
                    pixels_rgba: Vec::from(FALLBACK_CURSOR_DATA),
                    pixels_argb: vec![]
                }]
            },
        };

        Self { icons, size }
    }

    pub fn get_image(&self, scale: u32, time: Duration) -> Image {
        let size = self.size * scale;
        let mut millis = time.as_millis() as u32;
        let images: &[Image] = &self.icons;
        let total = nearest_image(size, images).fold(0, |acc,image|acc+image.delay);
        millis %= total;

        for img in nearest_image(size, images) {
            if millis < img.delay {
                return img.clone();
            }
            millis -= img.delay;
        }

        unreachable!()
    }
}

fn load_icon(theme: &CursorTheme) -> Result<Vec<Image>> {
    let icon_path = theme.load_icon("default").context("no default cursor")?;
    let mut cursor_file = fs::File::open(&icon_path)?;
    let mut cursor_data = vec![];
    cursor_file.read_to_end(&mut cursor_data)?;
    let images = xcursor::parser::parse_xcursor(&cursor_data).context("failed to parse cursor file")?;
    Ok(images)
}


fn nearest_image(size: u32, images: &[Image]) -> impl Iterator<Item = &Image> {
    let nearest_image = images
        .iter()
        .min_by_key(|image| (size as i32 - image.size as i32).abs())
        .unwrap();

    images.iter().filter(move |image| {
        image.width == nearest_image.width && image.height == nearest_image.height
    })
}


