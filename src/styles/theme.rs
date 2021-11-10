use {
    super::{dark, light},
    iced::{button, container},
};

#[derive(Copy, Clone)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn symbol(&self) -> &str {
        match self {
            Self::Light => "\u{f0de}",
            Self::Dark => "\u{f0dd}",
        }
    }

    pub fn primary_btn(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Self::Light => light::button::Primary.into(),
            Self::Dark => dark::button::Primary.into(),
        }
    }

    pub fn secondary_btn(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Self::Light => light::button::Secondary.into(),
            Self::Dark => dark::button::Secondary.into(),
        }
    }

    pub fn transparency_btn(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Self::Light => light::button::Transparency.into(),
            Self::Dark => dark::button::Transparency.into(),
        }
    }

    pub fn inner_cont(&self) -> Box<dyn container::StyleSheet> {
        match self {
            Self::Light => light::container::Inner.into(),
            Self::Dark => dark::container::Inner.into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::Light
    }
}

impl<'a> From<Theme> for Box<dyn container::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => Default::default(),
            Theme::Dark => dark::container::Outer.into(),
        }
    }
}
