use iced::widget::{column, container, text, text_input, scrollable, Column};
use iced::widget::text_input::Status; 
use iced::{Element, Length, Color};
use super::style::{Palette, style_glass_card};
use crate::Message;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String, 
    pub content: String,
    pub timestamp: String,
}

// FIXED: Added lifetime 'a to match the borrowed slice
pub fn view<'a>(history: &'a [ChatMessage], current_input: &str) -> Element<'a, Message> {
    
    let messages: Element<Message> = scrollable(
        Column::with_children(
            history.iter().map(|msg| {
                let color = if msg.sender == "USER" { Palette::BLUE } else { Palette::PURPLE };
                
                container(
                    column![
                        text(&msg.sender).size(12).color(color),
                        text(&msg.content).size(16).color(Palette::TEXT_MAIN)
                    ].spacing(4)
                )
                .padding(10)
                .style(style_glass_card(Color::TRANSPARENT)) 
                .into()
            })
        )
        .spacing(10)
        .padding(10)
    )
    .height(Length::Fill)
    .into();

    let input = text_input("Enter command or query...", current_input)
        .on_input(Message::InputChanged)
        .on_submit(Message::SendChat)
        .padding(15)
        .style(|_theme, status| {
            let border_color = match status {
                Status::Focused { .. } => Palette::PURPLE, 
                _ => Palette::TEXT_DIM,
            };

            text_input::Style {
                background: iced::Background::Color(Palette::SURFACE),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                icon: Palette::TEXT_DIM,
                placeholder: Palette::TEXT_DIM,
                value: Palette::TEXT_MAIN,
                selection: Palette::BLUE,
            }
        });

    container(
        column![
            messages,
            input
        ].spacing(10)
    )
    .padding(20)
    .style(style_glass_card(Palette::PURPLE)) 
    .into()
}