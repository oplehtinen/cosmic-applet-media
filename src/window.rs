use crate::config;
use crate::mpris_subscription;
use cosmic::applet::padded_control;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::widget::column;
use cosmic::iced::Alignment;
use cosmic::iced::Length;
use cosmic::iced::Subscription;
use cosmic::theme;
use cosmic::widget::container;
use cosmic::widget::divider;
use cosmic::widget::icon;
use cosmic::widget::rectangle_tracker::rectangle_tracker_subscription;
use cosmic::widget::text;
use cosmic::widget::Id;
use cosmic::{
    applet::cosmic_panel_config::PanelAnchor,
    cosmic_theme::Spacing,
    iced::Rectangle,
    iced_core::window,
    iced_widget::row,
    widget::{autosize, button, rectangle_tracker::RectangleUpdate, RectangleTracker},
    Element, Task,
};
use mpris2_zbus::player::PlaybackStatus;
use mpris_subscription::{MprisRequest, MprisUpdate};
use once_cell::sync::Lazy;
static AUTOSIZE_MAIN_ID: Lazy<Id> = Lazy::new(|| Id::new("autosize-main"));
const GO_BACK: &str = "media-skip-backward-symbolic";
const GO_NEXT: &str = "media-skip-forward-symbolic";
const PAUSE: &str = "media-playback-pause-symbolic";
const PLAY: &str = "media-playback-start-symbolic";
pub struct Window {
    core: cosmic::app::Core,
    popup: Option<window::Id>,
    rectangle_tracker: Option<RectangleTracker<u32>>,
    rectangle: Rectangle,
    player_status: Option<mpris_subscription::PlayerStatus>,
}
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    CloseRequested(window::Id),
    Tick,
    Rectangle(RectangleUpdate<u32>),
    Mpris(mpris_subscription::MprisUpdate),
    MprisRequest(MprisRequest),
    ConfigChanged,
}
use crate::fl;
impl Window {}

impl cosmic::Application for Window {
    type Message = Message;
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    const APP_ID: &'static str = "com.github.oplehtinen.CosmicAppletMedia";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        (
            Self {
                core,
                popup: None,
                rectangle: Rectangle::default(),
                rectangle_tracker: None,
                player_status: None,
            },
            Task::none(),
        )
    }
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::CloseRequested(id))
    }
    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::Mpris(mpris_subscription::MprisUpdate::Player(p)) => {
                self.player_status = Some(p);
            }
            Message::Mpris(MprisUpdate::Finished) => {
                self.player_status = None;
            }
            Message::Mpris(MprisUpdate::Setup) => {
                self.player_status = None;
            }
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);
                    let Rectangle {
                        x,
                        y,
                        width,
                        height,
                    } = self.rectangle;
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.anchor_rect = Rectangle::<i32> {
                        x: x.max(1.) as i32,
                        y: y.max(1.) as i32,
                        width: width.max(1.) as i32,
                        height: height.max(1.) as i32,
                    };

                    popup_settings.positioner.size = Some((300, 500));
                    return get_popup(popup_settings);
                }
            }
            Message::MprisRequest(r) => {
                let Some(player_status) = self.player_status.as_ref() else {
                    tracing::error!("No player found");
                    return Task::none();
                };
                let player = player_status.player.clone();

                match r {
                    MprisRequest::Play => tokio::spawn(async move {
                        let res = player.play().await;
                        if let Err(err) = res {
                            tracing::error!("Error playing: {}", err);
                        }
                    }),
                    MprisRequest::Pause => tokio::spawn(async move {
                        let res = player.pause().await;
                        if let Err(err) = res {
                            tracing::error!("Error pausing: {}", err);
                        }
                    }),
                    MprisRequest::Next => tokio::spawn(async move {
                        let res = player.next().await;
                        if let Err(err) = res {
                            tracing::error!("Error playing next: {}", err);
                        }
                    }),
                    MprisRequest::Previous => tokio::spawn(async move {
                        let res = player.previous().await;
                        if let Err(err) = res {
                            tracing::error!("Error playing previous: {}", err);
                        }
                    }),
                };
            }
            Message::ConfigChanged => (),
            Message::CloseRequested(id) => {
                if Some(id) == self.popup {
                    self.popup = None;
                }
            }
            Message::Rectangle(u) => match u {
                RectangleUpdate::Rectangle(r) => {
                    self.rectangle = r.1;
                }
                RectangleUpdate::Init(tracker) => {
                    self.rectangle_tracker = Some(tracker);
                }
            },
            Message::Tick => (),
        }
        Task::none()
    }
    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            rectangle_tracker_subscription(0).map(|e| Message::Rectangle(e.1)),
            self.core
                .watch_config::<config::Config>(Self::APP_ID)
                .map(|u| {
                    for err in u.errors {
                        tracing::error!(?err, "Error watching config");
                    }
                    Message::ConfigChanged /* (u.config) */
                }),
            mpris_subscription::mpris_subscription(0).map(Message::Mpris),
        ])
    }
    fn view(&self) -> cosmic::Element<Self::Message> {
        let horizontal = matches!(
            self.core.applet.anchor,
            PanelAnchor::Top | PanelAnchor::Bottom
        );
        if let Some(s) = self.player_status.as_ref() {
            let title = if let Some(title) = s.title.as_ref() {
                if title.chars().count() > 52 {
                    let mut title_trunc = title.chars().take(50).collect::<String>();
                    title_trunc.push_str("...");
                    title_trunc
                } else {
                    title.to_string()
                }
            } else {
                String::new()
            };

            let artists = if let Some(artists) = s.artists.as_ref() {
                let artists = artists.join(", ");
                if artists.chars().count() > 57 {
                    let mut artists_trunc = artists.chars().take(55).collect::<String>();
                    artists_trunc.push_str("...");
                    artists_trunc
                } else {
                    artists
                }
            } else {
                fl!("unknown-artist")
            };
            let button = button::custom(if horizontal {
                Element::from(
                    row![
                        text::body(title).width(Length::Shrink),
                        text::body("–").width(Length::Shrink),
                        text::body(artists).width(Length::Shrink),
                    ]
                    .spacing(8),
                )
            } else {
                Element::from(column![
                    text::body(title).width(Length::Shrink),
                    text::caption(artists).width(Length::Shrink),
                ])
            })
            .padding(if horizontal {
                [0, self.core.applet.suggested_padding(true)]
            } else {
                [self.core.applet.suggested_padding(true), 0]
            })
            .on_press_down(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);
            autosize::autosize(
                if let Some(tracker) = self.rectangle_tracker.as_ref() {
                    Element::from(tracker.container(0, button).ignore_bounds(true))
                } else {
                    button.into()
                },
                AUTOSIZE_MAIN_ID.clone(),
            )
            .into()
        } else {
            let button = button::custom(if horizontal {
                Element::from(row!(self.core.applet.text("test")))
            } else {
                Element::from(row!(self.core.applet.text("Vertical widget")))
            })
            .padding(if horizontal {
                [0, self.core.applet.suggested_padding(true)]
            } else {
                [self.core.applet.suggested_padding(true), 0]
            })
            .on_press_down(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);
            autosize::autosize(
                if let Some(tracker) = self.rectangle_tracker.as_ref() {
                    Element::from(tracker.container(0, button).ignore_bounds(true))
                } else {
                    button.into()
                },
                AUTOSIZE_MAIN_ID.clone(),
            )
            .into()
        }
    }
    fn view_window(&self, id: cosmic::iced::window::Id) -> cosmic::Element<Self::Message> {
        let Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;
        fn control_element(control_type: MprisRequest) -> Option<Element<'static, Message>> {
            let control_icon = match control_type {
                MprisRequest::Next => Some(GO_NEXT),
                MprisRequest::Play => Some(PLAY),
                MprisRequest::Previous => Some(GO_BACK),
                MprisRequest::Pause => Some(PAUSE),
            };
            if let Some(icon_name) = control_icon {
                Some(
                    button::icon(icon::from_name(icon_name).size(16).symbolic(true))
                        .extra_small()
                        .class(cosmic::theme::Button::AppletIcon)
                        .on_press(Message::MprisRequest(control_type))
                        .into(),
                )
            } else {
                Some(text::body("").into())
            }
        }
        let mut control_elements = row![].width(Length::Shrink).spacing(space_s);
        if let Some(player) = &self.player_status {
            if player.can_go_previous {
                let prev_icon = control_element(MprisRequest::Previous);
                if let Some(prev_icon) = prev_icon {
                    control_elements = control_elements.push(prev_icon);
                }
            }
            if player.can_pause && player.status == PlaybackStatus::Playing {
                let pause_icon = control_element(MprisRequest::Pause);
                if let Some(pause_icon) = pause_icon {
                    control_elements = control_elements.push(pause_icon);
                }
            } else if player.can_play {
                let play_icon = control_element(MprisRequest::Play);
                if let Some(play_icon) = play_icon {
                    control_elements = control_elements.push(play_icon);
                }
            }
            if player.can_go_next {
                let next_icon = control_element(MprisRequest::Next);
                if let Some(next_icon) = next_icon {
                    control_elements = control_elements.push(next_icon);
                }
            }
        }

        let content = Element::from(
            column![
                padded_control(control_elements),
                padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            ]
            .align_x(Alignment::Center)
            .padding([8, 0]),
        );

        self.core.applet.popup_container(container(content)).into()
    }
}
