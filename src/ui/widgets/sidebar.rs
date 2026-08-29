use crate::appearance::{self, tokens};
use crate::nyaa_app::{NyaaMessage, ScreenKind};
use iced::Length::Fill;
use iced::border::Radius;
use iced::time::Instant;
use iced::widget::image::Handle;
use iced::widget::text::Wrapping;
use iced::widget::{button, center, column, container, image, row, rule, space, text};
use iced::{Alignment, Animation, Border, Element, Theme};
use iced_fonts::lucide::{library, menu, search, settings};
use std::sync::LazyLock;

static NYAA_ICON: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_bytes(include_bytes!("../../../assets/nyaa-icon.png").as_slice())
});

const SIDEBAR_COLLAPSED: f32 = 40.0;
const SIDEBAR_EXPANDED: f32 = 190.0;
const ICON_BUTTON_SIZE: f32 = SIDEBAR_COLLAPSED;

pub(crate) fn sidebar<'a>(
    animation: &Animation<bool>,
    current_screen: ScreenKind,
) -> Element<'a, NyaaMessage> {
    let now = Instant::now();
    let width = animation.interpolate(SIDEBAR_COLLAPSED, SIDEBAR_EXPANDED, now);
    let show_labels = !animation.is_animating(now) && animation.value();

    container(
        column![
            sidebar_header(show_labels),
            space().height(tokens::SPACING_LARGE),
            nav_button(
                search(),
                "Nyaa Search",
                NyaaMessage::Navigate(ScreenKind::Search),
                show_labels,
                current_screen == ScreenKind::Search
            ),
            nav_button(
                library(),
                "Library",
                NyaaMessage::Navigate(ScreenKind::Library),
                show_labels,
                current_screen == ScreenKind::Library
            ),
            space().height(Fill),
            rule::horizontal(tokens::BORDER_THIN),
            sidebar_footer(show_labels, current_screen),
        ]
        .spacing(tokens::SPACING_SMALL)
        .width(width),
    )
    .padding(tokens::SIDEBAR_PADDING)
    .style(appearance::container::weakest)
    .into()
}

fn sidebar_header<'a>(show_labels: bool) -> Element<'a, NyaaMessage> {
    row![
        image(NYAA_ICON.clone())
            .width(ICON_BUTTON_SIZE)
            .height(ICON_BUTTON_SIZE)
            .border_radius(tokens::RADIUS_LARGE),
        show_labels.then(|| text("Nyaa.rs")
            .size(20)
            .center()
            .width(Fill)
            .wrapping(Wrapping::None))
    ]
    .align_y(Alignment::Center)
    .spacing(tokens::SPACING_SMALL)
    .into()
}

fn sidebar_footer<'a>(show_labels: bool, current_screen: ScreenKind) -> Element<'a, NyaaMessage> {
    column![
        nav_button(
            settings(),
            "Settings",
            NyaaMessage::Navigate(ScreenKind::Settings),
            show_labels,
            current_screen == ScreenKind::Settings
        ),
        nav_button(
            menu(),
            "Collapse",
            NyaaMessage::ToggleSidebar,
            show_labels,
            false
        ),
    ]
    .spacing(tokens::SPACING_SMALL)
    .into()
}

fn nav_button<'a>(
    icon: impl Into<Element<'a, NyaaMessage>>,
    label: &'static str,
    msg: NyaaMessage,
    show_labels: bool,
    is_selected: bool,
) -> Element<'a, NyaaMessage> {
    let icon_box = center(icon.into())
        .width(ICON_BUTTON_SIZE)
        .height(ICON_BUTTON_SIZE);

    let button = if show_labels {
        button(
            row![icon_box, text(label)]
                .align_y(Alignment::Center)
                .spacing(tokens::SPACING_SMALL),
        )
        .width(Fill)
        .padding(tokens::BTN_PADDING)
    } else {
        button(icon_box).width(ICON_BUTTON_SIZE).padding(0)
    };

    button
        .on_press(msg)
        .height(ICON_BUTTON_SIZE)
        .style(nav_button_style(is_selected))
        .into()
}

pub fn nav_button_style(is_selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();

        let (bg_color, text_color) = match (status, is_selected) {
            (button::Status::Hovered | button::Status::Pressed, true) => (
                palette.background.strong.color,
                palette.background.base.text,
            ),
            (button::Status::Hovered, false) => (
                palette.background.neutral.color,
                palette.background.base.text,
            ),
            (_, true) => (palette.background.base.color, palette.background.base.text),
            (_, false) => (
                palette.background.weakest.color,
                palette.background.base.text,
            ),
        };

        button::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: Border {
                radius: Radius::from(tokens::RADIUS_LARGE),
                ..Default::default()
            },
            text_color,
            ..Default::default()
        }
    }
}
