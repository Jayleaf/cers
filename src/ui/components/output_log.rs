use ratatui::{
    style::{Color, Modifier, Style, Stylize}, text::Span, widgets::{Block, List, ListDirection}
};

use crate::ui::main::AMApp;

///
/// Gets the progress message list from the App state, styles it, and returns the list.
/// 
pub async fn output_log(app: AMApp) -> List<'static> {
    
    let results = &app.get_progress_msg().await;
    let results_styled = results.clone().into_iter().map(|p| {
        Span::styled(p, Style::default().fg(Color::from_u32(0x00FF00)))
    
    }).collect::<Vec<Span<'_>>>();

    let list = List::new(results_styled)
        .direction(ListDirection::TopToBottom)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("").bg(Color::from_u32(0x00151414))
        .repeat_highlight_symbol(false)
        .block(Block::bordered().title("Output Log"));
    list
    
}