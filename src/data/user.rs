use crate::data::battle::LivingBuilder;
use crate::{data::battle::Living, data::items::InventoryItem};
use ab_glyph::{FontRef, PxScale};
use eyre::Result;
use image::{
    imageops::{overlay, FilterType},
    GenericImage, Rgba, RgbaImage,
};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use serenity::all::{CreateAttachment, User};
use std::{
    fmt::{Display, Formatter},
    path::Path,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct DBUser {
    pub this_levels_xp: u64,
    pub xp_until_next_level: u64,
    pub level: u64,
    pub items: Vec<InventoryItem>,
    pub life: Living,
}

impl Default for DBUser {
    fn default() -> Self {
        Self {
            this_levels_xp: 0,
            xp_until_next_level: 100,
            level: 1,
            items: vec![],
            life: LivingBuilder::new().health(150).build().unwrap(),
        }
    }
}

#[derive(Error, Debug)]
pub enum DBUserError {
    UserDoesNotHaveItem(InventoryItem),
    FontFailedToParse,
}

impl Display for DBUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DBUserError::UserDoesNotHaveItem(item) => {
                f.write_str(&format!("user does not have item {:?}", item))
            }
            DBUserError::FontFailedToParse => f.write_str("Font failed to parse"),
        }
    }
}

macro_rules! rect {
    ($pos: expr, $size: expr) => {
        Rect::at($pos.0 as i32, $pos.1 as i32).of_size($size.0 as u32, $size.1 as u32)
    };
}

impl DBUser {
    pub fn update_required_xp(&mut self) {
        self.xp_until_next_level = 100 + self.level.pow(2) / 2
    }

    pub fn gain_xp(&mut self, xp: u64) {
        if self.level < 1 {
            self.level = 1
        }

        self.this_levels_xp += xp;
        self.check_level_up();
    }

    pub fn check_level_up(&mut self) {
        loop {
            self.update_required_xp();

            if self.this_levels_xp < self.xp_until_next_level {
                break;
            }

            self.level += 1;

            self.this_levels_xp -= self.xp_until_next_level;
        }
    }

    pub fn give_item(&mut self, item: InventoryItem) {
        self.items.push(item);
    }

    pub fn drop_item(&mut self, item: InventoryItem) -> Result<(), DBUserError> {
        let idx = self.items.iter().position(|x| *x == item);

        if idx.is_none() {
            return Err(DBUserError::UserDoesNotHaveItem(item));
        }

        let idx = idx.unwrap();

        self.items.remove(idx);

        Ok(())
    }

    pub async fn image(&self, user: &User) -> Result<RgbaImage> {
        let size = (550, 244);

        let mut img = RgbaImage::new(size.0, size.1);
        let bg = Rgba([17, 17, 17, 255]);

        for x in 0..size.0 {
            // Fill background
            for y in 0..size.1 {
                *img.get_pixel_mut(x, y) = bg; // Set BG Color
            }
        }

        // Get font
        let light_bytes = std::fs::read("./fonts/light.ttf")?;

        let font_light = FontRef::try_from_slice(light_bytes.as_slice())
            .ok()
            .ok_or(DBUserError::FontFailedToParse)?;

        // Draw display name
        let display_name = (if let Some(x) = &user.global_name {
            x
        } else {
            &user.name
        })
        .to_uppercase();

        let text_scale_light = 1.333f32;

        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            224,
            40,
            PxScale::from(40f32 * text_scale_light),
            &font_light,
            &display_name,
        );

        let level = format!(
            "Level {} ({}/{})",
            self.level, self.this_levels_xp, self.xp_until_next_level
        );

        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            224,
            98,
            PxScale::from(20f32 * text_scale_light),
            &font_light,
            &level,
        );

        let health = format!("{} / {} Health", self.life.health(), self.life.max_health());

        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            224,
            129,
            PxScale::from(20f32 * text_scale_light),
            &font_light,
            &health,
        );

        // Leveling bar
        let leveling_bar_pos = (224f32, 168f32);
        let leveling_bar_size = (286f32, 30f32);
        let filled_percentage = self.this_levels_xp as f32 / self.xp_until_next_level as f32;
        let filled_size = (
            if leveling_bar_size.0 * filled_percentage > 1f32 {
                leveling_bar_size.0 * filled_percentage
            } else {
                1f32
            },
            leveling_bar_size.1,
        );

        // Fill the outline
        draw_filled_rect_mut(
            &mut img,
            rect!(leveling_bar_pos, leveling_bar_size),
            Rgba([255, 255, 255, 255]),
        );

        // Hollow out the outline
        let outline_thickness = 1f32;
        let outline_hole_pos = (
            leveling_bar_pos.0 + outline_thickness,
            leveling_bar_pos.1 + outline_thickness,
        );
        let outline_hole_size = (
            leveling_bar_size.0 - 2f32 * outline_thickness,
            leveling_bar_size.1 - 2f32 * outline_thickness,
        );

        draw_filled_rect_mut(&mut img, rect!(outline_hole_pos, outline_hole_size), bg);

        // Draw the filled portion
        draw_filled_rect_mut(
            &mut img,
            rect!(leveling_bar_pos, filled_size),
            Rgba([255, 255, 255, 255]),
        );

        // Health bar
        let health_bar_pos = (224f32, 203f32);
        let health_bar_size = (286f32, 1f32);

        draw_filled_rect_mut(
            &mut img,
            rect!(health_bar_pos, health_bar_size),
            Rgba([255, 117, 117, 255]),
        );

        let health_filled_percentage = self.life.health() as f32 / self.life.max_health() as f32;
        let health_filled_size = (
            health_bar_size.0 * health_filled_percentage,
            health_bar_size.1,
        );

        draw_filled_rect_mut(
            &mut img,
            rect!(health_bar_pos, health_filled_size),
            Rgba([255, 255, 255, 255]),
        );

        // Render user's profile picture
        let avatar_url = user.avatar_url().unwrap_or(user.default_avatar_url());
        let avatar_bytes = reqwest::get(avatar_url).await?.bytes().await?;

        let avatar_image = image::load_from_memory(&avatar_bytes)?;

        // 1. Resize the image to be 156x156
        let size = 164f32;
        let mut tiny = avatar_image.resize(size as u32, size as u32, FilterType::CatmullRom);

        // 2. Go through every pixel and clip
        for x in 0..tiny.width() {
            for y in 0..tiny.height() {
                let pos = (x as f32 / size, y as f32 / size);
                let distance =
                    (1f32 - (pos.0 * 2f32)).powf(2f32) + (1f32 - (pos.1 * 2f32)).powf(2f32);

                if distance > 1f32 {
                    tiny.put_pixel(x, y, bg);
                }
            }
        }

        // 3. Draw it at 40, 40
        overlay(&mut img, &tiny, 40, 40);

        Ok(img)
    }

    pub async fn attachment_image(&self, user: &User) -> Result<CreateAttachment> {
        let image = self.image(user).await?;

        let file_path = "./temp.png";

        image.save(Path::new(file_path))?;

        Ok(CreateAttachment::path(Path::new(file_path)).await?)
    }
}
