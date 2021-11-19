use {
    super::{dark, light},
    iced::{button, container, progress_bar, radio, rule, scrollable, slider, text_input, toggler},
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

    pub fn danger_btn(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Self::Light => light::button::Danger.into(),
            Self::Dark => dark::button::Danger.into(),
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

impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::ProgressBar.into(),
            Theme::Dark => dark::ProgressBar.into(),
        }
    }
}

impl<'a> From<Theme> for Box<dyn radio::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::Radio.into(),
            Theme::Dark => dark::Radio.into(),
        }
    }
}

impl From<Theme> for Box<dyn rule::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::Rule.into(),
            Theme::Dark => dark::Rule.into(),
        }
    }
}

impl<'a> From<Theme> for Box<dyn scrollable::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::Scrollable.into(),
            Theme::Dark => dark::Scrollable.into(),
        }
    }
}

impl<'a> From<Theme> for Box<dyn slider::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::Slider.into(),
            Theme::Dark => dark::Slider.into(),
        }
    }
}

impl<'a> From<Theme> for Box<dyn text_input::StyleSheet + 'a> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::TextInput.into(),
            Theme::Dark => dark::TextInput.into(),
        }
    }
}

impl From<Theme> for Box<dyn toggler::StyleSheet> {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => light::Toggler.into(),
            Theme::Dark => dark::Toggler.into(),
        }
    }
}
