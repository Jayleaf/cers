use std::{iter, rc::Rc};

use ratatui::{
    layout::Rect, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{List, ListDirection}, Frame
};

use crate::ui::main::AMApp;

pub async fn mem_view_window(area: Rect, frame: &mut Frame<'_>, chunks: Rc<[Rect]>, app: AMApp) {
    
    let process = String::from("Address");
    let process_id = String::from("Value");
    
    let query = app.get_query().await.1;
    let line_width: usize = process.len() + process_id.len();
    let space_count = area.width as usize - line_width;

    let spaces: String = iter::repeat(' ').take(space_count - 3).collect::<String>();
    let title_lines: Vec<Span<'_>> = vec![
        "  ".into(),
        process.into(),
        spaces.into(),
        process_id.into(),
    ];
    let title = Line::from(title_lines).bg(Color::from_u32(0x00151414));
    frame.render_widget(title, chunks[0]);
    let results: &Vec<String> = &app.get_query_results(0..200).await;
    let results_styled = results.clone().into_iter().map(|p| {
        let line_width: usize = p.len() + query.len();
        let space_count = area.width as usize - line_width;
        let spaces: String = iter::repeat(' ').take(space_count - 3).collect::<String>();
        let addr = p.to_string();
        let val = &query;
        let process_lines: Vec<Span<'_>> = vec![
            " ".into(),
            addr.clone().into(),
            spaces.into(),
            val.clone().into(),

        ];
        if results.iter().position(|q| *q == p).unwrap() % 2 == 0 { Line::from(process_lines).bg(Color::from_u32(0x00363636)) }
        else { Line::from(process_lines).bg(Color::from_u32(0x00252525)) }
    
    }).collect::<Vec<Line<'_>>>();
    if results_styled.len() == 0 {
        let empty = Line::from("No Results Available").centered().bg(Color::from_u32(0x00252525));
        frame.render_widget(empty, chunks[1]);
    }
    else {
    let list = List::new(results_styled)
        .direction(ListDirection::TopToBottom)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("❚").bg(Color::from_u32(0x00151414))
        .repeat_highlight_symbol(false);
    app.modify_mem_view_list("set", Some(list.clone())).await;
    frame.render_stateful_widget(list, chunks[1], &mut app.app.lock().await.mem_view_list.state); // oh god oh fuck
    }


    let lines: Vec<Span<'_>> = vec![
        "[m]: Modify Value".dark_gray(),
        "  |  ".fg(Color::from_u32(0x00151414)),
        "[c]: Copy Address".dark_gray(),
    ];
    let bar = Line::from(lines).centered().bg(Color::from_u32(0x00212121));
    frame.render_widget(bar, chunks[2]);

}