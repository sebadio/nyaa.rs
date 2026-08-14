use crate::nyaa_app::NyaaMessage;
use iced::Length::Fill;
use iced::border::Radius;
use iced::time::{Duration, Instant};
use iced::widget::{button, column, container, opaque, row, space, text};
use iced::{Border, Element, Theme};
use iced_fonts::lucide::x;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ToastId(u64);

impl ToastId {
    pub(crate) fn unique() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ToastId {
    fn default() -> Self {
        Self::unique()
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[expect(dead_code)]
pub(crate) enum ToastKind {
    Success,
    Error,
    Warning,
    #[default]
    Info,
}

impl ToastKind {
    fn lifetime(&self) -> Duration {
        match self {
            ToastKind::Error => Duration::from_secs(999),
            ToastKind::Warning => Duration::from_secs(6),
            ToastKind::Success | ToastKind::Info => Duration::from_secs(4),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Toast {
    pub(crate) id: ToastId,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) kind: ToastKind,
    pub(crate) created_at: Instant,
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast {
    pub(crate) fn new() -> Self {
        Self {
            id: ToastId::unique(),
            created_at: Instant::now(),
            kind: ToastKind::default(),
            message: String::new(),
            title: String::new(),
        }
    }

    pub(crate) fn id(&self) -> ToastId {
        self.id
    }

    pub(crate) fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub(crate) fn set_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub(crate) fn set_kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    pub(crate) fn lifetime(&self) -> Duration {
        self.kind.lifetime()
    }

    pub(crate) fn view(&self) -> Element<'_, NyaaMessage> {
        opaque(
            container(column![
                row![
                    text(self.title.clone()),
                    space().width(Fill),
                    button(x())
                        .on_press(NyaaMessage::DismissToast(self.id()))
                        .style(button_style)
                ],
                row![text(self.message.clone()).size(12)]
            ])
            .style(|theme: &Theme| container_style(theme, &self.kind))
            .padding(8)
            .height(120)
            .width(320),
        )
    }
}

fn button_style(theme: &Theme, status: button::Status) -> button::Style {
    let bg_style = match status {
        button::Status::Hovered => iced::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.2,
        },
        _ => iced::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
    };

    button::Style {
        background: Some(iced::Background::Color(bg_style)),
        border: iced::Border {
            radius: Radius::from(4),
            ..Default::default()
        },
        text_color: theme.palette().primary.strong.color,
        ..Default::default()
    }
}

fn container_style(theme: &Theme, kind: &ToastKind) -> container::Style {
    let bg = match kind {
        ToastKind::Info => theme.palette().primary.weak.color,
        ToastKind::Success => theme.palette().success.weak.color,
        ToastKind::Warning => theme.palette().warning.weak.color,
        ToastKind::Error => theme.palette().danger.weak.color,
    };

    let border_color = match kind {
        ToastKind::Info => theme.palette().primary.strong.color,
        ToastKind::Success => theme.palette().success.strong.color,
        ToastKind::Warning => theme.palette().warning.strong.color,
        ToastKind::Error => theme.palette().danger.strong.color,
    };

    container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            width: 2.0,
            color: border_color,
            radius: Radius::from(4),
        },
        ..Default::default()
    }
}
