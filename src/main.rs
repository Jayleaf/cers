use std::{io, process::exit, ptr::NonNull, thread};

use backend::components::get_mem_from_query::get_mem_from_query;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{executor::block_on, pin_mut};
use ratatui::{prelude::CrosstermBackend, Terminal};
use ui::main::{App, DataType};

mod backend;
mod ui;

#[tokio::main]
async fn main() {
    let app = App::new();

    

    // spawn the thread for data reception over threads
    tokio::spawn({
        let app = app.clone();
        let recv = app.app.lock().await.rx.clone();
        let bounds = app.get_bounds().await;
        async move {
            while let Some(msg) = recv.recv().ok() {
                match msg.data_type {
                    DataType::BeginMemoryScan => {
                        let lower_bound = usize::from_str_radix(&bounds.0 .1, 16).unwrap();
                        let upper_bound = usize::from_str_radix(&bounds.1 .1, 16).unwrap();
                        get_mem_from_query(upper_bound, lower_bound, app.clone()).await;
                    }
                    _ => {}
                }
            }
        }
    });

    color_eyre::install();
    let mut terminal = ratatui::init();
    let app_result = ui::main::run_app(&mut terminal, app.clone()).await;
    ratatui::restore();

}
