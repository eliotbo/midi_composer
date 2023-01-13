use iced::{
    application,
    widget::{button, container, scrollable, text},
    Color,
};

use iced_native::widget::pane_grid;
use iced_native::widget::rule;

macro_rules! color {
    ($red:expr, $green:expr, $blue:expr) => {
        Color::from_rgba(
            $red as f32 / 255.0,
            $green as f32 / 255.0,
            $blue as f32 / 255.0,
            1.0,
        )
    };
    ($red:expr, $green:expr, $blue:expr, $opacity:expr) => {
        Color::from_rgba(
            $red as f32 / 255.0,
            $green as f32 / 255.0,
            $blue as f32 / 255.0,
            $opacity as f32 / 255.0,
        )
    };
}

pub struct PianoTheme {
    text: Color,

    pub piano_background: Color,
    pub background: Color,
    currant_line: Color,

    pub cyan: Color,

    pub orange: Color,
    pink: Color,
    purple: Color,
    red: Color,
    yellow: Color,
}

impl PianoTheme {
    pub const NORMAL: Self = Self {
        text: color!(120, 120, 120),

        piano_background: color!(30, 30, 33),
        background: color!(60, 60, 60),
        currant_line: color!(68, 71, 90),

        cyan: color!(139, 233, 253),

        // orange: color!(255, 184, 108),
        orange: Color {
            r: 1.0,
            g: 0.8,
            b: 0.5,
            a: 1.0,
        },
        pink: color!(255, 121, 198),
        purple: color!(189, 147, 249),
        red: color!(255, 85, 85),
        yellow: color!(241, 250, 140),
    };
}

impl Default for PianoTheme {
    fn default() -> Self {
        Self::NORMAL
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Application {
    #[default]
    Default,
}

impl application::StyleSheet for PianoTheme {
    type Style = Application;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        match style {
            Application::Default => application::Appearance {
                background_color: self.background.into(),
                text_color: self.text,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Button {
    #[default]
    Yellow,
    Black,
}

impl button::StyleSheet for PianoTheme {
    type Style = Button;

    fn active(&self, style: &Button) -> button::Appearance {
        let auto_fill = |background: Color, text: Color| button::Appearance {
            background: background.into(),
            text_color: text,
            border_radius: 2.0,
            ..button::Appearance::default()
        };

        match style {
            Button::Yellow => auto_fill(self.yellow, self.text),
            Button::Black => auto_fill(Color::BLACK, self.text),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        let difference = if &Button::Black == style {
            iced::Vector::new(0.0, 0.0)
        } else {
            iced::Vector::new(0.0, 1.0)
        };

        button::Appearance {
            shadow_offset: active.shadow_offset + difference,
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: iced::Vector::default(),
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                iced::Background::Color(color) => iced::Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

/*
 * Container
 */
#[derive(Clone, Copy, Default)]
pub enum Container {
    #[default]
    Transparent,
    Box,
    Custom(fn(&PianoTheme) -> container::Appearance),
}

impl From<fn(&PianoTheme) -> container::Appearance> for Container {
    fn from(f: fn(&PianoTheme) -> container::Appearance) -> Self {
        Self::Custom(f)
    }
}

impl container::StyleSheet for PianoTheme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => Default::default(),
            Container::Box => container::Appearance {
                text_color: None,
                background: self.piano_background.into(),
                border_radius: 2.0,
                border_width: 3.0,
                border_color: self.orange,
                // border_color: Color {
                //     r: 1.0,
                //     g: 1.0,
                //     b: 0.0,
                //     a: 1.0,
                // },
            },
            Container::Custom(f) => f(self),
        }
    }
}

/*
 * Text
 */
#[derive(Clone, Copy, Default)]
pub enum Text {
    #[default]
    Default,
    Color(Color),
    Custom(fn(&PianoTheme) -> text::Appearance),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Text::Color(color)
    }
}

impl text::StyleSheet for PianoTheme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Default => Default::default(),
            Text::Color(c) => text::Appearance { color: Some(c) },
            Text::Custom(f) => f(self),
        }
    }
}

/// The style of a scrollable.
#[derive(Default)]
pub enum Scrollable {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn scrollable::StyleSheet<Style = PianoTheme>>),
}

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Line {
//     /// The [`Color`] of the [`Line`].
//     pub color: Color,

//     /// The width of the [`Line`].
//     pub width: f32,
// }

impl pane_grid::StyleSheet for PianoTheme {
    /// The supported style of the [`StyleSheet`].
    type Style = Scrollable;

    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            Scrollable::Default => Some(pane_grid::Line {
                width: 0.5,
                color: self.orange,
            }),
            Scrollable::Custom(_) => None,
        }
    }

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            Scrollable::Default => Some(pane_grid::Line {
                width: 0.5,
                color: self.orange,
            }),
            Scrollable::Custom(_) => None,
        }
    }
}

impl iced::widget::rule::StyleSheet for PianoTheme {
    type Style = Scrollable;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        match style {
            Scrollable::Default => rule::Appearance {
                color: self.currant_line,
                width: 1,
                radius: 0.0,
                fill_mode: rule::FillMode::Full,
                // ..rule::Appearance::default()
            },
            Scrollable::Custom(_) => rule::Appearance {
                color: self.pink,
                width: 10,
                radius: 0.0,
                fill_mode: rule::FillMode::Full,
                // ..rule::Appearance::default()
            },
        }
    }
}

impl iced_native::widget::scrollable::StyleSheet for PianoTheme {
    type Style = Scrollable;

    fn active(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => {
                // let palette = self.extended_palette();

                scrollable::Scrollbar {
                    background: Some(self.background.into()),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    scroller: scrollable::Scroller {
                        color: self.orange.into(),
                        border_radius: 2.0,
                        border_width: 2.0,
                        border_color: self.red,
                    },
                }
            }
            Scrollable::Custom(custom) => custom.active(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => scrollable::Scrollbar {
                background: Some(self.background.into()),
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: self.orange.into(),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: self.purple,
                },
            },
            Scrollable::Custom(custom) => custom.hovered(self),
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => self.hovered(style),
            Scrollable::Custom(custom) => custom.dragging(self),
        }
    }
}
