mod lobotomy; // Re-using your existing logic

use iced::widget::{column, container, text};
use iced::{Element, Length, Subscription, Task, Theme, time};
use std::time::Duration;
use lobotomy::SystemMonitor;

pub fn main() -> iced::Result {
    iced::application("Cartesian Core", Cartesian::update, Cartesian::view)
    .subscription(Cartesian::subscription)
    .theme(Cartesian::theme)
    .run()
}

struct Cartesian {
    monitor: SystemMonitor,
    status: String,
    is_detected: bool,
    pid: String,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl Default for Cartesian {
    fn default() -> Self {
        Self {
            monitor: SystemMonitor::new(),
            status: "SEARCHING...".to_string(),
            is_detected: false,
            pid: "0".to_string(),
        }
    }
}

impl Cartesian {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let target = "firefox";
                // We use your existing lobotomy module
                match self.monitor.find_process(target) {
                    Some(pid) => {
                        self.is_detected = true;
                        self.pid = pid.to_string();
                        self.status = format!("DETECTED [PID: {}]", pid);
                    }
                    None => {
                        self.is_detected = false;
                        self.pid = "0".to_string();
                        self.status = "SAFE".to_string();
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Sci-Fi Colors
        let text_color = if self.is_detected {
            iced::Color::from_rgb8(255, 123, 114) // Red
        } else {
            iced::Color::from_rgb8(35, 134, 54)   // Green
        };

        let content = column![
            text("CARTESIAN CORE (NATIVE)")
            .size(40)
            .color(iced::Color::from_rgb8(88, 166, 255)), // Blue

            text(format!("SYSTEM STATUS: ONLINE"))
            .size(20)
            .color(iced::Color::from_rgb8(35, 134, 54)), // Green

            container(
                column![
                    text("TARGET: FIREFOX").size(20),
                      text(&self.status).size(30).color(text_color),
                ]
                .align_x(iced::Alignment::Center)
                .spacing(10)
            )
            .padding(20)
            .style(|_theme| {
                container::Style {
                    // FIX: Use iced::Border instead of container::Border
                    border: iced::Border {
                        color: iced::Color::from_rgb8(48, 54, 61),
                   width: 1.0,
                   radius: 8.0.into(),
                    },
                    background: Some(iced::Color::from_rgb8(22, 27, 34).into()),
                   ..Default::default()
                }
            })
        ]
        .spacing(20)
        .align_x(iced::Alignment::Center);

        container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme| {
            container::Style {
                background: Some(iced::Color::from_rgb8(13, 17, 23).into()),
               ..Default::default()
            }
        })
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // The Heartbeat: Run the update loop every 1000ms
        time::every(Duration::from_millis(1000)).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
