use bevy::{color::palettes::tailwind, prelude::*};

use crate::backend::ThemeColorId;

pub enum Theme {
    Default,
}

pub trait ColorTools {
    fn color(&self, theme: Theme) -> Color {
        todo!()
    }
}

impl ColorTools for ThemeColorId {
    fn color(&self, theme: Theme) -> Color {
        let srgba = match theme {
            Theme::Default => match self {
                ThemeColorId::Choice1 => tailwind::ROSE_700,
                ThemeColorId::Choice2 => tailwind::PURPLE_800,
                ThemeColorId::Choice3 => tailwind::BLUE_800,
                ThemeColorId::Choice4 => tailwind::CYAN_500,
                ThemeColorId::Choice5 => tailwind::EMERALD_900,
                ThemeColorId::Choice6 => tailwind::LIME_500,
                ThemeColorId::Choice7 => tailwind::YELLOW_500,
                ThemeColorId::Choice8 => tailwind::RED_900,
                ThemeColorId::Choice9 => tailwind::ZINC_800,
                ThemeColorId::Choice10 => tailwind::SLATE_900,
            },
        };

        srgba.into()
    }
}
