use iced::{
    application,
    widget::{button, container, scrollable, text},
    Color,
};

use iced_native::widget::pane_grid;
use iced_native::widget::rule;

macro_rules! color {
    ($red:expr, $green:expr, $blue:expr) => {
        Color::from_rgba($red as f32 / 255.0, $green as f32 / 255.0, $blue as f32 / 255.0, 1.0)
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

pub struct TrackTheme {
    pub text: Color,
    pub button: Color,
    pub button_pressed: Color,

    pub grid_background: Color,

    pub grid_beat_line: Color,
    pub grid_subbeat_line: Color,
    pub grid_row_line: Color,

    pub grid_piano_light_row: Color,
    pub grid_piano_dark_row: Color,

    pub grid_light_column: Color,
    pub grid_dark_column: Color,

    pub piano_background: Color,
    pub background: Color,
    pub player_head: Color,

    currant_line: Color,

    pub track_contour: Color,
    pink: Color,
    purple: Color,
    red: Color,
    yellow: Color,

    pub note: Color,
    pub selected_note: Color,
    pub note_contour: Color,
}

impl TrackTheme {
    pub fn x(c: Color, m: f32) -> Color {
        Color { r: c.r * m, g: c.g * m, b: c.b * m, a: c.a }
    }

    pub fn xa(c: Color, m: f32) -> Color {
        Color { r: c.r, g: c.g, b: c.b, a: c.a * m }
    }

    // // convert a rgb color to linear space
    // pub fn to_linear(c: Color) -> Color {
    //     Color { r: c.r.powf(2.2), g: c.g.powf(2.2), b: c.b.powf(2.2), a: c.a }
    // }

    // release TODO: make thid a const
    pub fn get_fall() -> Self {
        let solid_dark = Color::from_rgb8(44, 54, 57);
        let solid_dark2 = Color::from_rgb8(63, 78, 79);

        let alpha = 0.25;
        let button = Color::from_rgba8(44, 54, 57, 1.0);
        let button_pressed = Color::from_rgba8(44, 54, 57, 0.5);

        let transparent_dark = Self::xa(solid_dark, alpha);
        let transparent_dark2 = Self::xa(solid_dark2, alpha); // Color::from_rgba8(63, 78, 79, alpha);
        let white_bg = Color::from_rgba8(220, 215, 201, alpha / 2.0);

        let selected = Color::from_rgba8(162, 5, 92, 0.5);
        let black = Color::from_rgb8(2, 5, 3);
        let green = Color::from_rgb8(105, 130, 105);

        let brown = Color::from_rgb8(162, 123, 92);
        let red = Color::from_rgb8(170, 86, 86);
        let beige = Color::from_rgb8(241, 219, 191);
        let sand = Color::from_rgb8(185, 155, 107);

        let ggg = 60;
        let grey = Color::from_rgba8(10, ggg, ggg, 1.0);

        Self {
            text: color!(120, 120, 120),
            button,
            button_pressed,

            grid_background: transparent_dark,

            grid_beat_line: Self::x(transparent_dark2, 2.0),
            grid_subbeat_line: Self::x(transparent_dark2, 1.5),
            grid_row_line: Self::x(transparent_dark2, 1.1),

            grid_light_column: white_bg,
            grid_dark_column: Self::x(white_bg, 0.5),

            grid_piano_light_row: Self::x(transparent_dark, 2.0),
            grid_piano_dark_row: Self::x(transparent_dark, 0.8),

            note: green,
            selected_note: selected,
            note_contour: grey,

            piano_background: color!(30, 30, 33),
            player_head: color!(30, 180, 200),

            background: color!(60, 60, 60),
            currant_line: color!(68, 71, 255),

            track_contour: black,

            pink: color!(255, 121, 198),
            purple: color!(189, 147, 249),
            red: color!(255, 85, 85),
            yellow: color!(241, 250, 140),
        }
    }
}

impl Default for TrackTheme {
    fn default() -> Self {
        Self::get_fall()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Application {
    #[default]
    Default,
}

impl application::StyleSheet for TrackTheme {
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

impl button::StyleSheet for TrackTheme {
    type Style = Button;

    fn active(&self, style: &Button) -> button::Appearance {
        let auto_fill = |background: Color, text: Color| button::Appearance {
            background: background.into(),
            text_color: text,
            border_radius: 2.0,
            ..button::Appearance::default()
        };

        match style {
            Button::Yellow => auto_fill(self.button, self.text),
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

        button::Appearance { shadow_offset: active.shadow_offset + difference, ..active }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let auto_fill = |background: Color, text: Color| button::Appearance {
            background: background.into(),
            text_color: text,
            border_radius: 2.0,
            ..button::Appearance::default()
        };

        match style {
            Button::Yellow => auto_fill(self.button_pressed, self.text),
            Button::Black => auto_fill(Color::BLACK, self.text),
        }

        // button::Appearance { shadow_offset: iced::Vector::default(), ..self.active(style) }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                iced::Background::Color(color) => {
                    iced::Background::Color(Color { a: color.a * 0.5, ..color })
                }
            }),
            text_color: Color { a: active.text_color.a * 0.5, ..active.text_color },
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
    Custom(fn(&TrackTheme) -> container::Appearance),
}

impl From<fn(&TrackTheme) -> container::Appearance> for Container {
    fn from(f: fn(&TrackTheme) -> container::Appearance) -> Self {
        Self::Custom(f)
    }
}

impl container::StyleSheet for TrackTheme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => Default::default(),
            Container::Box => container::Appearance {
                text_color: None,
                background: self.piano_background.into(),
                border_radius: 2.0,
                border_width: 3.0,
                border_color: self.track_contour,
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
    Custom(fn(&TrackTheme) -> text::Appearance),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Text::Color(color)
    }
}

impl text::StyleSheet for TrackTheme {
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
    Custom(Box<dyn scrollable::StyleSheet<Style = TrackTheme>>),
}

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Line {
//     /// The [`Color`] of the [`Line`].
//     pub color: Color,

//     /// The width of the [`Line`].
//     pub width: f32,
// }

impl pane_grid::StyleSheet for TrackTheme {
    /// The supported style of the [`StyleSheet`].
    type Style = Scrollable;

    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            Scrollable::Default => Some(pane_grid::Line { width: 0.5, color: self.track_contour }),
            Scrollable::Custom(_) => None,
        }
    }

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            Scrollable::Default => Some(pane_grid::Line { width: 0.5, color: self.track_contour }),
            Scrollable::Custom(_) => None,
        }
    }
}

impl iced::widget::rule::StyleSheet for TrackTheme {
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

impl iced_native::widget::scrollable::StyleSheet for TrackTheme {
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
                        color: self.track_contour.into(),
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
                    color: self.track_contour.into(),
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
