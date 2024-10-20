#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use macroquad::camera::{set_camera, set_default_camera, Camera3D, Projection};
use macroquad::color::{Color, WHITE};
use macroquad::input::is_mouse_button_pressed;
use macroquad::math::{vec3, Rect, Vec3};
use macroquad::models::{draw_cylinder_wires, draw_line_3d, draw_plane};
use macroquad::texture::{draw_texture_ex, render_target, DrawTextureParams, Image, RenderTarget};
use macroquad::ui::{Style, Ui};
use macroquad::{
    input::{is_mouse_button_released, mouse_position_local, MouseButton},
    prelude::FilterMode,
    text::{load_ttf_font_from_bytes, measure_text, Font, TextDimensions},
    ui::{
        hash, root_ui,
        widgets::{Texture, Window},
        Skin,
    },
};
use rand::distributions::{Distribution, Standard};
use rand::{random, Rng};
use retro_wicket_macros::{hex, include_textures};
use std::time::Instant;
use std::{
    cmp::Ordering,
    collections::HashMap,
    mem::swap,
    ops::{Deref, DerefMut},
};
use strum::Display;

use macroquad::{
    main,
    math::{vec2, Vec2},
    prelude::ImageFormat,
    shapes::draw_rectangle,
    texture::Texture2D,
    window::{clear_background, next_frame, screen_height, screen_width},
};

macro_rules! include_texture {
    ($name:literal) => {{
        let texture = Texture2D::from_file_with_format(
            include_bytes!(concat!(
                env!("OUT_DIR"),
                concat!("/", concat!($name, ".png"))
            )),
            Some(ImageFormat::Png),
        );
        texture.set_filter(FilterMode::Nearest);
        texture
    }};
}
#[main("Retro Wicket")]
async fn main() {
    Game::new().run().await;
}

#[derive(Debug, Clone)]
struct Game<'n> {
    state: State<'n>,
    font: Font,
    text_measurer: TextMeasurer,
    render_target: RenderTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
enum CoinSide {
    Heads,
    Tails,
}

impl Distribution<CoinSide> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CoinSide {
        if rng.gen() {
            CoinSide::Heads
        } else {
            CoinSide::Tails
        }
    }
}

impl CoinSide {
    fn texture(self) -> Texture2D {
        match self {
            Self::Heads => include_texture!("heads"),
            Self::Tails => include_texture!("tails"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum State<'n> {
    PickingSide,
    TossingCoin {
        bet: CoinSide,
        mouse_down_y: Option<f32>,
    },
    FlippingCoin {
        bet: CoinSide,
        start: Instant,
    },
    ShowingCoinResult {
        bet: CoinSide,
        result: CoinSide,
        opponent_choice: Role,
    },
    Playing {
        innings: usize,
        teams: Teams<'n>,
    },
}

impl<'n> State<'n> {
    const fn start() -> Self {
        Self::PickingSide
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Teams<'n> {
    batting: Team<'n>,
    fielding: Team<'n>,
}

impl<'n> Teams<'n> {
    fn switch(&mut self) {
        swap(&mut self.batting, &mut self.fielding);
    }

    const fn new(names: [&'n str; 2]) -> Self {
        Self {
            batting: Team { name: names[0] },
            fielding: Team { name: names[1] },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Team<'n> {
    name: &'n str,
}

#[derive(Debug, Clone)]
struct TextMeasurer {
    font_data: &'static [u8],
    cache: HashMap<TextMeasureInput, TextDimensions>,
}

impl TextMeasurer {
    fn new(font_data: &'static [u8]) -> Self {
        Self {
            cache: HashMap::new(),
            font_data,
        }
    }

    fn measure(&mut self, input: TextMeasureInput) -> TextDimensions {
        if let Some(dimensions) = self.cache.get(&input) {
            return *dimensions;
        }
        let dimensions = measure_text(
            &input.text,
            Some(&load_ttf_font_from_bytes(self.font_data).unwrap()),
            input.size,
            1.,
        );
        self.cache.insert(input, dimensions);
        dimensions
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextMeasureInput {
    text: String,
    size: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Batting,
    Fielding,
}

impl Distribution<Role> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Role {
        if rng.gen() {
            Role::Batting
        } else {
            Role::Fielding
        }
    }
}

impl Role {
    fn texture(self) -> Texture2D {
        match self {
            Self::Batting => include_texture!("bat"),
            Self::Fielding => include_texture!("field"),
        }
    }
}

enum ScreenSide {
    Left,
    Right,
}

impl ScreenSide {
    fn from_mouse_position() -> Self {
        match mouse_position_local().x.total_cmp(&0.) {
            Ordering::Less | Ordering::Equal => Self::Left,
            Ordering::Greater => Self::Right,
        }
    }
}

#[allow(unused)]
enum Colour {
    Black,
    DarkBlue,
    DarkPurple,
    DarkGreen,
    Brown,
    DarkGrey,
    LightGrey,
    White,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Lavender,
    Pink,
    LightPeach,
}

impl Colour {
    const fn colour(self) -> Color {
        match self {
            Self::Black => hex!(000000),
            Self::DarkBlue => hex!(1D2B53),
            Self::DarkPurple => hex!(7E2553),
            Self::DarkGreen => hex!(008751),
            Self::Brown => hex!(AB5236),
            Self::DarkGrey => hex!(5F574F),
            Self::LightGrey => hex!(C2C3C7),
            Self::White => hex!(FFF1E8),
            Self::Red => hex!(FF004D),
            Self::Orange => hex!(FFA300),
            Self::Yellow => hex!(FFEC27),
            Self::Green => hex!(00E436),
            Self::Blue => hex!(29ADFF),
            Self::Lavender => hex!(83769C),
            Self::Pink => hex!(FF77A8),
            Self::LightPeach => hex!(FFCCAA),
        }
    }
}

macro_rules! colour {
    ($colour:ident) => {
        Colour::$colour.colour()
    };
}

impl<'n> Game<'n> {
    fn new() -> Self {
        let font_data = include_bytes!("fonts/Quinque Five Font.ttf");
        let font = load_ttf_font_from_bytes(font_data).unwrap();
        let render_target = render_target(Self::SIZE.x as u32 * 2, Self::SIZE.y as u32 * 2);
        render_target.texture.set_filter(FilterMode::Nearest);
        Self {
            state: State::start(),
            font,
            text_measurer: TextMeasurer::new(font_data),
            render_target,
        }
    }

    fn draw_borders() {
        let size_pixels = Self::transform_size(Self::SIZE);
        let position = Self::transform_point(Vec2::ZERO);

        draw_rectangle(0., 0., position.x, size_pixels.y, colour!(Black));
        draw_rectangle(
            position.x + size_pixels.x,
            0.,
            position.x,
            size_pixels.y,
            colour!(Black),
        );
        draw_rectangle(0., 0., size_pixels.x, position.y, colour!(Black));
        draw_rectangle(
            0.,
            position.y + size_pixels.y,
            size_pixels.x,
            position.y,
            colour!(Black),
        );
    }

    fn screen_size() -> Vec2 {
        vec2(screen_width(), screen_height())
    }

    fn scale() -> f32 {
        let screen_size = Self::screen_size();
        (screen_size / Self::SIZE).min_element()
    }

    fn transform_point(point: Vec2) -> Vec2 {
        Self::scale() * point + Self::screen_size() / 2. - Self::scale() * Self::SIZE / 2.
    }

    fn transform_size(size: Vec2) -> Vec2 {
        Self::scale() * size
    }

    fn transform_length(length: f32) -> f32 {
        Self::scale() * length
    }

    fn untransform_length(length: f32) -> f32 {
        length / Self::scale()
    }

    async fn run(&mut self) {
        loop {
            set_default_camera();
            Self::draw_borders();
            match &mut self.state {
                State::PickingSide => {
                    self.draw_picking_side();
                }
                State::TossingCoin { .. } => {
                    self.draw_tossing_coin();
                }
                State::FlippingCoin { .. } => {
                    self.draw_flipping_coin();
                }
                State::ShowingCoinResult { .. } => {
                    self.draw_showing_coin_result();
                }
                State::Playing { .. } => {
                    self.draw_playing();
                }
            }
            next_frame().await;
        }
    }

    fn draw_playing(&self) {
        self.draw_playing_to_render_texture();

        set_default_camera();
        let position = Self::transform_point(Vec2::ZERO);
        draw_texture_ex(
            &self.render_target.texture,
            position.x,
            position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Self::transform_size(Self::SIZE)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    fn draw_playing_to_render_texture(&self) {
        const TARGET: Vec3 = vec3(0., 0., 0.);
        const POSITION: Vec3 = vec3(0., 2., 14.);
        const BOWLING_CREASE_TO_END: f32 = 1.22;
        const PITCH_WIDTH: f32 = 3.05;
        const BOWLING_CREASE_TO_POPPING_CREASE: f32 = 1.22;
        const STUMP_DIAMETER: f32 = 0.034;
        const BETWEEN_STUMPS: f32 = 0.054;
        const STUMP_HEIGHT: f32 = 0.71;
        const BETWEEN_WICKETS: f32 = 20.12;
        const POPPING_CREASE_LENGTH: f32 = 3.66;
        const BOWLING_CREASE_LENGTH: f32 = 2.64;

        const STUMP_DISTANCE: f32 = STUMP_DIAMETER + BETWEEN_STUMPS;
        const PITCH_LENGTH: f32 = BETWEEN_WICKETS + 2. * BOWLING_CREASE_TO_END;

        const LINE_COLOUR: Color = colour!(White);
        const GRASS_COLOUR: Color = colour!(DarkGreen);
        const PITCH_COLOUR: Color = colour!(LightPeach);

        set_camera(&Camera3D {
            aspect: Some(Self::SIZE.x / Self::SIZE.y),
            target: TARGET,
            position: POSITION,
            up: Vec3::Y,
            fovy: 60_f32.to_radians(),
            projection: Projection::Perspective,
            viewport: None,
            render_target: Some(self.render_target.clone()),
        });
        clear_background(colour!(Blue));
        draw_plane(Vec3::ZERO, vec2(1000., 1000.), None, GRASS_COLOUR);
        draw_plane(
            Vec3::ZERO,
            vec2(PITCH_WIDTH / 2., PITCH_LENGTH / 2.),
            None,
            PITCH_COLOUR,
        );
        for side in [-1., 1.] {
            for stump in [-1., 0., 1.] {
                draw_cylinder_wires(
                    vec3(
                        stump * STUMP_DISTANCE,
                        STUMP_HEIGHT / 2.,
                        side * BETWEEN_WICKETS / 2.,
                    ),
                    STUMP_DIAMETER / 2.,
                    STUMP_DIAMETER / 2.,
                    STUMP_HEIGHT,
                    None,
                    LINE_COLOUR,
                );
            }

            draw_line_3d(
                vec3(
                    -POPPING_CREASE_LENGTH / 2.,
                    0.,
                    side * (BETWEEN_WICKETS / 2. - BOWLING_CREASE_TO_POPPING_CREASE),
                ),
                vec3(
                    POPPING_CREASE_LENGTH / 2.,
                    0.,
                    side * (BETWEEN_WICKETS / 2. - BOWLING_CREASE_TO_POPPING_CREASE),
                ),
                LINE_COLOUR,
            );
            draw_line_3d(
                vec3(
                    -BOWLING_CREASE_LENGTH / 2.,
                    0.,
                    side * (BETWEEN_WICKETS / 2.),
                ),
                vec3(
                    BOWLING_CREASE_LENGTH / 2.,
                    0.,
                    side * (BETWEEN_WICKETS / 2.),
                ),
                LINE_COLOUR,
            );
            for return_crease in [-1., 1.] {
                draw_line_3d(
                    vec3(
                        return_crease * BOWLING_CREASE_LENGTH / 2.,
                        0.,
                        side * (BETWEEN_WICKETS / 2. - BOWLING_CREASE_TO_POPPING_CREASE),
                    ),
                    vec3(
                        return_crease * BOWLING_CREASE_LENGTH / 2.,
                        0.,
                        side * (BETWEEN_WICKETS / 2. + BOWLING_CREASE_TO_END),
                    ),
                    LINE_COLOUR,
                );
            }
        }
    }

    fn draw_showing_coin_result(&mut self) {
        const TEXTURE_SIZE: f32 = 40.;
        const TEXT_GAP: f32 = 2.;
        const TEXT_SIZE: u16 = 2;
        const X_GAP: f32 = 100.;
        const HEADING_TEXT_GAP: f32 = 10.;
        let [heading_style, text_style] = self.skins([Self::HEADING_TEXT_SIZE, TEXT_SIZE]);
        let State::ShowingCoinResult {
            bet,
            result,
            opponent_choice,
        } = self.state
        else {
            unreachable!()
        };
        Self::window(|ui| {
            if bet == result {
                self.draw_choose_role(ui, &text_style, TEXTURE_SIZE, X_GAP, TEXT_GAP, TEXT_SIZE);
            }
            ui.push_skin(&heading_style);
            Texture::new(result.texture())
                .position(Self::transform_size(
                    Self::SIZE / 2. - Vec2::splat(TEXTURE_SIZE / 2.),
                ))
                .size(
                    Self::transform_length(TEXTURE_SIZE),
                    Self::transform_length(TEXTURE_SIZE),
                )
                .ui(ui);
            let text = format!("{result}!");
            let dimensions = self.text_measurer.measure(TextMeasureInput {
                text: text.clone(),
                size: Self::transform_length(Self::HEADING_TEXT_SIZE as f32) as u16,
            });
            ui.label(
                Self::transform_size(vec2(
                    Self::SIZE.x / 2. - Self::untransform_length(dimensions.width / 2.),
                    Self::SIZE.y / 2. + TEXTURE_SIZE / 2. + HEADING_TEXT_GAP,
                )),
                &text,
            );
            let text = if bet == result {
                "Choose to bat or field first".to_string()
            } else {
                format!(
                    "Opponent chose to {} first! Click to continue",
                    match opponent_choice {
                        Role::Batting => "bat",
                        Role::Fielding => "field",
                    }
                )
            };
            ui.push_skin(&text_style);
            let sub_dimensions = self.text_measurer.measure(TextMeasureInput {
                text: text.clone(),
                size: Self::transform_length(TEXT_SIZE as f32) as u16,
            });
            ui.label(
                Self::transform_size(vec2(
                    Self::SIZE.x / 2. - Self::untransform_length(sub_dimensions.width / 2.),
                    Self::SIZE.y / 2.
                        + TEXTURE_SIZE / 2.
                        + HEADING_TEXT_GAP
                        + Self::untransform_length(dimensions.height)
                        + TEXT_GAP,
                )),
                &text,
            );
        });
        if is_mouse_button_released(MouseButton::Left) {
            if bet == result {
                match ScreenSide::from_mouse_position() {
                    ScreenSide::Left => {
                        self.state = State::Playing {
                            innings: 0,
                            teams: Teams::new(["You", "Opponent"]),
                        }
                    }
                    ScreenSide::Right => {
                        self.state = State::Playing {
                            innings: 0,
                            teams: Teams::new(["Opponent", "You"]),
                        }
                    }
                }
            } else {
                self.state = State::Playing {
                    innings: 0,
                    teams: match opponent_choice {
                        Role::Batting => Teams::new(["Opponent", "You"]),
                        Role::Fielding => Teams::new(["You", "Opponent"]),
                    },
                }
            }
        }
    }

    fn draw_choose_role(
        &mut self,
        ui: &mut Ui,
        text_style: &Skin,
        texture_size: f32,
        x_gap: f32,
        text_gap: f32,
        text_size: u16,
    ) {
        ui.push_skin(text_style);
        let position = Self::transform_point(vec2(
            match mouse_position_local().x.total_cmp(&0.) {
                Ordering::Less | Ordering::Equal => 0.,
                Ordering::Greater => Self::SIZE.x / 2.,
            },
            0.,
        ));
        let size = Self::transform_size(vec2(Self::SIZE.x / 2., Self::SIZE.y));
        ui.canvas().rect(
            Rect::new(position.x, position.y, size.x, size.y),
            None,
            Self::HIGHLIGHT_COLOUR,
        );

        for (x_side, role, lines) in [
            (
                -1.,
                Role::Batting,
                ["You bat first", "Opponent fields first"],
            ),
            (
                1.,
                Role::Fielding,
                ["You field first", "Opponent bats first"],
            ),
        ] {
            let total_height = text_gap.mul_add(2., texture_size)
                + Self::untransform_length(
                    lines
                        .into_iter()
                        .map(|line| {
                            self.text_measurer
                                .measure(TextMeasureInput {
                                    text: line.to_string(),
                                    size: Self::transform_length(text_size as f32) as u16,
                                })
                                .height
                        })
                        .sum(),
                );
            Texture::new(role.texture())
                .position(Self::transform_size(vec2(
                    Self::SIZE.x / 2. + x_side * x_gap / 2. - texture_size / 2.,
                    Self::SIZE.y / 2. - total_height / 2.,
                )))
                .size(
                    Self::transform_length(texture_size),
                    Self::transform_length(texture_size),
                )
                .ui(ui);

            let mut total_text_height = 0.;
            for (index, line) in lines.into_iter().enumerate() {
                let dimensions = self.text_measurer.measure(TextMeasureInput {
                    text: line.to_string(),
                    size: Self::transform_length(text_size as f32) as u16,
                });
                ui.label(
                    Self::transform_size(vec2(
                        Self::SIZE.x / 2. + x_side * x_gap / 2.
                            - Self::untransform_length(dimensions.width / 2.),
                        (index as f32).mul_add(
                            text_gap,
                            Self::SIZE.y / 2. - total_height / 2. + texture_size + text_gap,
                        ) + total_text_height
                            - Self::untransform_length(dimensions.height / 2.),
                    )),
                    line,
                );
                total_text_height += Self::untransform_length(dimensions.height);
            }
        }
        ui.pop_skin();
    }

    const SIZE: Vec2 = vec2(160., 100.);

    fn draw_flipping_coin(&mut self) {
        const ANIMATION_FRAMES_PER_SECOND: f32 = 10.;
        const TOP: f32 = 10.;
        const BOTTOM: f32 = 10.;
        const EXTRA_FRAMES: usize = 3;
        let State::FlippingCoin { bet, start } = self.state else {
            unreachable!()
        };
        Self::window(|ui| {
            fn inner<const N: usize>(
                textures: [Texture2D; N],
                f: impl FnOnce([Texture2D; N], usize),
            ) {
                f(textures, N);
            }
            inner(include_textures!("coin-flip", 1..=18), |textures, len| {
                let index = (Instant::now()
                    .saturating_duration_since(start)
                    .as_secs_f32()
                    * ANIMATION_FRAMES_PER_SECOND) as usize;
                let texture = textures.into_iter().nth(index.min(len - 1)).unwrap();
                let height = Self::SIZE.y - TOP - BOTTOM;
                let width = height * texture.width() / texture.height();
                let size = Self::transform_size(vec2(width, height));
                Texture::new(texture)
                    .position(Self::transform_size(vec2(
                        Self::SIZE.x / 2. - width / 2.,
                        TOP,
                    )))
                    .size(size.x, size.y)
                    .ui(ui);
                if index >= len + EXTRA_FRAMES {
                    self.state = State::ShowingCoinResult {
                        bet,
                        result: random(),
                        opponent_choice: random(),
                    }
                }
            });
        });
    }

    fn draw_tossing_coin(&mut self) {
        const TEXTURE_SIZE: f32 = 40.;
        const TEXTURE_TOP: f32 = 50.;
        const TEXT_GAP: f32 = 2.;
        let State::TossingCoin { bet, mouse_down_y } = &mut self.state else {
            unreachable!()
        };
        Self::window(|ui| {
            let texture = include_texture!("coin-drag");
            let position =
                Self::transform_size(vec2(Self::SIZE.x / 2. - TEXTURE_SIZE / 2., TEXTURE_TOP));
            let size = Self::transform_size(Vec2::splat(TEXTURE_SIZE));
            Texture::new(texture)
                .position(position)
                .size(size.x, size.y)
                .ui(ui);
            let text = "Swipe up to flip";
            let dimensions = self.text_measurer.measure(TextMeasureInput {
                text: text.to_string(),
                size: Self::transform_length(Self::TEXT_SIZE as f32) as u16,
            });
            ui.label(
                vec2(
                    Self::transform_length(Self::SIZE.x) / 2. - dimensions.width / 2.,
                    Self::transform_length(TEXTURE_TOP + TEXTURE_SIZE + TEXT_GAP),
                ),
                text,
            );
        });
        if is_mouse_button_pressed(MouseButton::Left) {
            *mouse_down_y = Some(mouse_position_local().y);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(mouse_down_y) = mouse_down_y {
                let delta = mouse_position_local().y - *mouse_down_y;
                if delta < 0. {
                    self.state = State::FlippingCoin {
                        bet: *bet,
                        start: Instant::now(),
                    }
                }
            }
        }
    }

    fn window(f: impl FnOnce(&mut Ui)) {
        Window::new(
            hash!(),
            Self::transform_point(Vec2::ZERO),
            Self::transform_size(Self::SIZE),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), f);
    }

    fn make_skin(style: &Style) -> Skin {
        Skin {
            label_style: style.clone(),
            button_style: style.clone(),
            tabbar_style: style.clone(),
            combobox_style: style.clone(),
            window_style: style.clone(),
            editbox_style: style.clone(),
            window_titlebar_style: style.clone(),
            scrollbar_style: style.clone(),
            scrollbar_handle_style: style.clone(),
            checkbox_style: style.clone(),
            group_style: style.clone(),
            margin: 0.,
            title_height: 0.,
            scroll_width: 0.,
            scroll_multiplier: 0.,
        }
    }

    fn skins<const N: usize>(&self, sizes: [u16; N]) -> [Skin; N] {
        sizes.map(|size| {
            let style = root_ui()
                .style_builder()
                .with_font(&self.font)
                .unwrap()
                .font_size(Self::transform_length(size as f32) as u16)
                .background(Image::gen_image_color(1, 1, Self::BACKGROUND_COLOUR))
                .build();
            Self::make_skin(&style)
        })
    }

    const BACKGROUND_COLOUR: Color = colour!(LightPeach);

    const HEADING_TEXT_SIZE: u16 = 10;
    const TEXT_SIZE: u16 = 5;
    const HIGHLIGHT_COLOUR: Color = colour!(Yellow);

    fn draw_picking_side(&mut self) {
        const GAP: f32 = 80.;
        const HEADING_TOP: f32 = 10.;
        const TEXTURE_TOP: f32 = 30.;
        const TEXT_GAP: f32 = 2.;
        const TEXTURE_SIZE: f32 = 40.;
        let [heading_style, text_style] = self.skins([Self::HEADING_TEXT_SIZE, Self::TEXT_SIZE]);

        Self::window(|ui| {
            let position = Self::transform_point(vec2(
                match ScreenSide::from_mouse_position() {
                    ScreenSide::Left => 0.,
                    ScreenSide::Right => Self::SIZE.x / 2.,
                },
                0.,
            ));
            let size = Self::transform_size(vec2(Self::SIZE.x / 2., Self::SIZE.y));
            ui.canvas().rect(
                Rect::new(position.x, position.y, size.x, size.y),
                None,
                Self::HIGHLIGHT_COLOUR,
            );

            ui.push_skin(&heading_style);
            let text = "Pick a side";
            let dimensions = self.text_measurer.measure(TextMeasureInput {
                text: text.to_string(),
                size: Self::transform_length(Self::HEADING_TEXT_SIZE as f32) as u16,
            });
            ui.label(
                vec2(
                    Self::transform_length(Self::SIZE.x) / 2. - dimensions.width / 2.,
                    HEADING_TOP + dimensions.height,
                ),
                "Pick a side",
            );

            ui.pop_skin();
            ui.push_skin(&text_style);
            for (position, side) in [(-1., CoinSide::Heads), (1., CoinSide::Tails)] {
                Texture::new(side.texture())
                    .position(Self::transform_size(vec2(
                        Self::SIZE.x / 2. + position * GAP / 2. - TEXTURE_SIZE / 2.,
                        TEXTURE_TOP,
                    )))
                    .size(
                        Self::transform_length(TEXTURE_SIZE),
                        Self::transform_length(TEXTURE_SIZE),
                    )
                    .ui(ui);
                let dimensions = self.text_measurer.measure(TextMeasureInput {
                    text: side.to_string(),
                    size: Self::transform_length(Self::TEXT_SIZE as f32) as u16,
                });
                ui.label(
                    vec2(
                        Self::transform_length(Self::SIZE.x / 2. + position * GAP / 2.)
                            - dimensions.width / 2.,
                        Self::transform_length(TEXTURE_TOP + TEXTURE_SIZE + TEXT_GAP),
                    ),
                    &side.to_string(),
                );
            }

            if is_mouse_button_released(MouseButton::Left) {
                self.state = State::TossingCoin {
                    bet: match mouse_position_local().x.total_cmp(&0.) {
                        Ordering::Less | Ordering::Equal => CoinSide::Heads,
                        Ordering::Greater => CoinSide::Tails,
                    },
                    mouse_down_y: None,
                }
            }
        });
    }
}

impl<'n> Deref for Game<'n> {
    type Target = State<'n>;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'n> DerefMut for Game<'n> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
