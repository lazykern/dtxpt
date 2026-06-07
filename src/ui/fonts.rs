use bevy::prelude::*;

const SYSTEM_UI_FONTS: &[&str] = &[
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
];

#[derive(Resource, Clone)]
pub struct UiFonts {
    regular: Option<Handle<Font>>,
}

impl FromWorld for UiFonts {
    fn from_world(world: &mut World) -> Self {
        let mut fonts = world.resource_mut::<Assets<Font>>();
        for path in SYSTEM_UI_FONTS {
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            match Font::try_from_bytes(bytes) {
                Ok(font) => {
                    info!("loaded UI font {path}");
                    return Self {
                        regular: Some(fonts.add(font)),
                    };
                }
                Err(err) => warn!("failed to parse UI font {path}: {err}"),
            }
        }

        warn!("no CJK-capable UI font found; falling back to Bevy default font");
        Self { regular: None }
    }
}

pub fn text_font(fonts: &UiFonts, size: f32) -> TextFont {
    let mut font = TextFont::from_font_size(size);
    if let Some(handle) = &fonts.regular {
        font.font = handle.clone();
    }
    font
}
