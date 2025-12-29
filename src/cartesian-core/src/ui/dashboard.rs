use iced::widget::{column, container, row, button, text};
use iced::{Element, Length};

use crate::{Cartesian, Message};
use crate::lobotomy::AppCategory;
// Import the chat view
use super::chat; 
use super::style::{Palette, style_glass_card, style_background, label_header, label_main};

pub fn view<'a>(state: &'a Cartesian) -> Element<'a, Message> {
    
    // ... (Color Logic Unchanged) ...
    let context_color = match state.current_context {
        AppCategory::Game => Palette::RED,
        AppCategory::Production => Palette::YELLOW,
        AppCategory::Development => Palette::BLUE,
        _ => Palette::GREEN,
    };
    let brain_color = if state.brain_state.contains("GOD") { Palette::PURPLE } else { Palette::YELLOW };
    let vision_color = if state.vision_status.contains("INPUT") { Palette::BLUE } else { Palette::ORANGE };
    let audio_state = state.mixer.get_state();
    let mic_color = if audio_state.mic_muted { Palette::RED } else { Palette::GREEN };

    // Helper strings
    let cpu_ram_text = format!("CPU: {:.1}% | RAM: {:.1} GB FREE", state.cpu_usage, state.free_ram);
    
    // --- LEFT COLUMN: TELEMETRY ---
    let telemetry_col = column![
        label_header("CARTESIAN OS // CORE".to_string()).size(24),
        label_header(cpu_ram_text),

        // A. GOVERNOR
        container(column![
            label_header("AI GOVERNOR".to_string()),
            label_main(state.brain_state.clone(), brain_color), 
        ].spacing(5)).padding(20).style(style_glass_card(brain_color)),

        // B. CONTEXT
        container(column![
            label_header("SYSTEM CONTEXT".to_string()),
            label_main(state.status.clone(), context_color),
        ].spacing(5)).padding(20).style(style_glass_card(context_color)),

        // C. WITNESS
        container(column![
            label_header("WITNESS PROTOCOL".to_string()),
            label_main(state.vision_status.clone(), vision_color),
        ].spacing(5)).padding(20).style(style_glass_card(vision_color)),

        // D. AUDIO
        container(column![
            label_header("AUDIO MIXER".to_string()),
            row![
                text(format!("GAME: {:.0}%", audio_state.game_vol * 100.0)).size(16).color(Palette::TEXT_MAIN),
                text(format!("MIC: {}", if audio_state.mic_muted {"MUT"} else {"ON"})).size(16).color(mic_color),
            ].spacing(20)
        ].spacing(10)).padding(20).style(style_glass_card(Palette::TEXT_DIM)),

        button(if state.debug_override { "STOP SIM" } else { "SIMULATE GAME" })
            .on_press(Message::ToggleOverride)
            .padding(10)
    ]
    .spacing(20)
    .width(Length::FillPortion(1)); // Take 1/3 width

    // --- RIGHT COLUMN: CHAT ---
    let chat_col = container(
        chat::view(&state.chat_history, &state.input_value)
    )
    .width(Length::FillPortion(2)) // Take 2/3 width
    .height(Length::Fill);

    // --- MAIN LAYOUT ---
    let content = row![
        telemetry_col,
        chat_col
    ]
    .spacing(20)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style_background)
        .into()
}