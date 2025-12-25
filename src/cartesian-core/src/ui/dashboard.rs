use iced::widget::{column, container, row, button, text};
use iced::{Element, Length};

use crate::{Cartesian, Message};
use crate::lobotomy::AppCategory;
use super::style::{Palette, style_glass_card, style_background, label_header, label_main};

/// The Main Dashboard View
/// Decoupled from the Controller (main.rs)
pub fn view<'a>(state: &Cartesian) -> Element<'a, Message> {
    
    // 1. BUSINESS LOGIC (Colors)
    let context_color = match state.current_context {
        AppCategory::Game => Palette::RED,
        AppCategory::Production => Palette::YELLOW,
        AppCategory::Development => Palette::BLUE,
        _ => Palette::GREEN,
    };
    
    let brain_color = if state.brain_state.contains("GOD") { Palette::PURPLE }
    else if state.brain_state.contains("POTATO") { Palette::RED }
    else { Palette::YELLOW };

    let vision_color = if state.vision_status.contains("INPUT") { Palette::BLUE } else { Palette::ORANGE };
    
    let audio_state = state.mixer.get_state();
    let mic_color = if audio_state.mic_muted { Palette::RED } else { Palette::GREEN };

    // 2. LAYOUT
    let content = column![
        label_header("CARTESIAN OS // CORE").size(24),
        label_header(&format!("CPU: {:.1}% | RAM: {:.1} GB FREE", state.cpu_usage, state.free_ram)),

        if state.unknown_count > 0 {
            label_header(&format!("WARNING: {} UNKNOWN APPS", state.unknown_count)).color(Palette::RED)
        } else {
            label_header("REGISTRY CLEAN").color(Palette::GREEN)
        },

        // A. GOVERNOR
        container(column![
            label_header("AI GOVERNOR"),
            label_main(&state.brain_state, brain_color),
        ].align_x(iced::Alignment::Center).spacing(5))
        .padding(20)
        .style(style_glass_card(brain_color)),

        // B. CONTEXT
        container(column![
            label_header("SYSTEM CONTEXT"),
            label_main(&state.status, context_color),
        ].align_x(iced::Alignment::Center).spacing(5))
        .padding(20)
        .style(style_glass_card(context_color)),

        // C. WITNESS
        container(column![
            label_header("WITNESS PROTOCOL"),
            label_main(&state.vision_status, vision_color),
        ].align_x(iced::Alignment::Center).spacing(5))
        .padding(20)
        .style(style_glass_card(vision_color)),

        // D. AUDIO
        container(column![
            label_header("AUDIO MIXER"),
            row![
                text(format!("GAME: {:.0}%", audio_state.game_vol * 100.0)).size(16).color(Palette::TEXT_MAIN),
                text(format!("VOICE: {:.0}%", audio_state.voice_vol * 100.0)).size(16).color(Palette::TEXT_MAIN),
                text(if audio_state.mic_muted {"MIC: MUTED"} else {"MIC: LIVE"}).size(16).color(mic_color),
            ].spacing(20)
        ].align_x(iced::Alignment::Center).spacing(10))
        .padding(20)
        .style(style_glass_card(Palette::TEXT_DIM)),

        // E. CONTROLS
        button(if state.debug_override { "STOP SIMULATION" } else { "SIMULATE GAME CONTEXT" })
            .on_press(Message::ToggleOverride)
            .padding(10)
    ]
    .spacing(20)
    .align_x(iced::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(style_background)
        .into()
}