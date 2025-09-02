use bevy::{
    color::palettes::{css, tailwind},
    prelude::*,
};

use crate::backend::{Class, ThemeColorId};

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>();
    }
}
#[derive(Debug, Resource, Default)]
pub enum Theme {
    #[default]
    Default,
}

pub trait ColorTools {
    fn color(&self, theme: &Theme) -> Color {
        let srgba = match theme {
            Theme::Default => css::MAGENTA, // replace this branch with a match arm for self.
        };

        srgba.into()
    }
}

impl ColorTools for ThemeColorId {
    fn color(&self, theme: &Theme) -> Color {
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

impl ColorTools for Class {
    fn color(&self, theme: &Theme) -> Color {
        let srgba = match theme {
            Theme::Default => match self {
                Class::Class1 => tailwind::VIOLET_800,
                Class::Class2 => tailwind::TEAL_700,
                Class::Class3 => tailwind::GREEN_800,
            },
        };

        srgba.into()
    }
}
