

use std::sync::Arc;
use tokio::sync::Mutex;
use std::io;
use crate::backend::components::get_process_list::get_process_list;

use tokio::sync::mpsc::{self, Receiver, Sender};
use super::rendering::ui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::Backend;
use ratatui::widgets::{List, ListState};
use ratatui::Terminal;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum CurrentScreen {
    #[default]
    Main,
    SelectingProcess,
    Exiting,
}


#[derive(Debug, Default, PartialEq, Clone)]
pub enum InputMode {
    #[default]
    Normal,
    EditingQuery,
    EditingUpperBound,
    EditingLowerBound,
}

#[derive(Debug, Default, Clone)]
pub enum ScanTypes {
    #[default]
    Exact,
    Range,
    Unknown,
}

pub enum DataType {
    ProgressMsg,
    QueryResults,
    QueryProgress,
    BeginMemoryScan,
}
pub struct Data {
    pub data_type: DataType,
    pub data: String
}

impl ScanTypes {
    pub fn as_str(&self) -> &str {
        match self {
            ScanTypes::Exact => "Exact",
            ScanTypes::Range => "Range",
            ScanTypes::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct VList {
    pub list: List<'static>,
    pub state: ListState,
}

impl VList {
    fn new() -> Self {
        let state = ListState::default();
        let list = List::default();
        VList { list, state }
    }
}




#[derive(Debug)]
pub struct App {
    pub open_process: i32,
    pub current_screen: CurrentScreen,
    pub proc_list: VList, // i really wish i didnt have to put this here lmfao
    pub mem_view_list: VList,
    pub query: (i32, String), // i32 is the index of the character, necessary for cursor positioning
    pub bounds: ((i32, String), (i32, String)), // upper and lower bounds for memory scanning, respectively
    pub query_results: Vec<(String, String)>,
    pub query_progress: f64,
    pub tx: Sender<Data>,
    pub rx: Receiver<Data>,
    pub progress_msg : String,
    pub input_mode: InputMode,
    pub scan_type: ScanTypes,

}

#[derive(Debug, Clone)]
pub struct AMApp {
    pub app: Arc<Mutex<App>>
}

impl AMApp {
    //setters
    pub async fn modify_process(&self, process: i32) {
        let mut app = self.app.lock().await;
        app.open_process = process;
    }
    pub async fn modify_current_screen(&self, screen: CurrentScreen) {
        let mut app = self.app.lock().await;
        app.current_screen = screen;
    }
    pub async fn modify_query(&self, query: (i32, String)) {
        let mut app = self.app.lock().await;
        app.query = query;
    }
    pub async fn modify_bounds(&self, bounds: ((i32, String), (i32, String))) {
        let mut app = self.app.lock().await;
        app.bounds = bounds;
    }
    pub async fn modify_query_results(&self, results: Vec<(String, String)>) {
        let mut app = self.app.lock().await;
        app.query_results = results;
    }
    pub async fn modify_query_progress(&self, progress: f64) {
        let mut app = self.app.lock().await;
        app.query_progress = progress;
    }
    pub async fn modify_progress_msg(&self, msg: String) {
        let mut app = self.app.lock().await;
        app.progress_msg = msg;
    }
    pub async fn modify_input_mode(&self, mode: InputMode) {
        let mut app = self.app.lock().await;
        app.input_mode = mode;
    }
    pub async fn modify_scan_type(&self, scan_type: ScanTypes) {
        let mut app = self.app.lock().await;
        app.scan_type = scan_type;
    }
    pub async fn modify_mem_view_list(&self, action: &str, list: Option<List<'static>>) {
        match action {
            "prev" => {
                let mut app = self.app.lock().await;
                app.mem_view_list.state.select_previous();
            },
            "next" => {
                let mut app = self.app.lock().await;
                app.mem_view_list.state.select_next();
            },
            "set" => {
                let mut app = self.app.lock().await;
                app.mem_view_list.list = list.unwrap();
            },
            _ => {
                
            }
        };
    }

    pub async fn modify_proc_list(&self, action: &str, list: Option<List<'static>>) {
        match action {
            "prev" => {
                let mut app = self.app.lock().await;
                app.proc_list.state.select_previous();
            },
            "next" => {
                let mut app = self.app.lock().await;
                app.proc_list.state.select_next();
            },
            "set" => {
                let mut app = self.app.lock().await;
                app.proc_list.list = list.unwrap();
            },
            _ => {
                
            }
        };
    }

    //getters

    pub async fn get_process(&self) -> i32 {
        let app = self.app.lock().await;
        app.open_process
    }
    pub async fn get_current_screen(&self) -> CurrentScreen {
        let app = self.app.lock().await;
        app.current_screen.clone()
    }
    pub async fn get_query(&self) -> (i32, String) {
        let app = self.app.lock().await;
        app.query.clone()
    }
    pub async fn get_bounds(&self) -> ((i32, String), (i32, String)) {
        let app = self.app.lock().await;
        app.bounds.clone()
    }
    pub async fn get_query_results(&self) -> Vec<(String, String)> {
        let app = self.app.lock().await;
        app.query_results.clone()
    }
    pub async fn get_query_progress(&self) -> f64 {
        let app = self.app.lock().await;
        app.query_progress
    }
    pub async fn get_progress_msg(&self) -> String {
        let app = self.app.lock().await;
        app.progress_msg.clone()
    }
    pub async fn get_input_mode(&self) -> InputMode {
        let app = self.app.lock().await;
        app.input_mode.clone()
    }
    pub async fn get_scan_type(&self) -> ScanTypes {
        let app = self.app.lock().await;
        app.scan_type.clone()
    }
    pub async fn get_tx(&self) -> Sender<Data> {
        let app = self.app.lock().await;
        app.tx.clone()
    }
    pub async fn get_mem_view_list(&self) -> VList {
        let app = self.app.lock().await;
        app.mem_view_list.clone()
    }
    pub async fn get_proc_list(&self) -> VList {
        let app = self.app.lock().await;
        app.proc_list.clone()
    }

}

impl App {
    pub fn new() -> AMApp {
        let (tx, rx): (Sender<Data>, Receiver<Data>) = mpsc::channel(100);
        let mut app = App {
            open_process: 0,
            current_screen: CurrentScreen::Main,
            proc_list: VList::new(),
            mem_view_list: VList::new(),
            query: (0, String::new()),
            bounds: ((18, String::from("0000000000000000")), (16, String::from("00007fffffffffff"))),
            input_mode: InputMode::Normal,
            query_results: Vec::new(),
            tx,
            rx,
            scan_type: ScanTypes::Exact,
            query_progress: 0.0,
            progress_msg: String::from("Query not started..."),
        };
        app.proc_list.state.select(Some(0)); // set a default value so the list renders properly
        app.mem_view_list.state.select(Some(0));
        AMApp { app: Arc::new(Mutex::new(app)) }
    }

}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: AMApp) -> io::Result<bool> {

    
    loop {
        let handle = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| {
            let x = terminal.draw(|f| {
                handle.block_on(ui(f, app.clone()))
            });
        });
        
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            // todo: put key events into their own module, it's too cluttered in here
            match app.get_input_mode().await {
                InputMode::Normal => match key.code {
                    KeyCode::Char('s') => {
                        if app.get_current_screen().await != CurrentScreen::Main { continue };
                        app.modify_input_mode(InputMode::EditingQuery).await
                    }
                    KeyCode::Char('t') => {
                        continue;
                    }
                    KeyCode::Char('b') => {
                        if app.get_current_screen().await != CurrentScreen::Main { continue };
                        app.modify_input_mode(InputMode::EditingLowerBound).await;
                    }
                    KeyCode::Enter => {
                        app.modify_input_mode(InputMode::Normal).await;
                        if let Err(x) = app.get_tx().await.send(Data { data_type: DataType::BeginMemoryScan, data: String::new() }).await
                        { app.modify_progress_msg(format!("Error sending data: {}", x)).await; }
                        else { app.modify_progress_msg(String::from("Beginning memory scan...")).await; }
       
                    },
                    _ => {}
                },
                InputMode::EditingQuery if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char(to_insert) => {
                        app.modify_query((app.get_query().await.0 + 1, format!("{}{}", app.get_query().await.1, to_insert))).await;
                    },
                    KeyCode::Backspace => {
                        // saturating sub for overflow error prevention
                        app.modify_query(((app.get_query().await.0 - 1).clamp(0, std::i32::MAX), app.get_query().await.1[..app.get_query().await.1.len().saturating_sub(1)].to_string())).await;
                    },
                    KeyCode::Esc => app.modify_input_mode(InputMode::Normal).await,
                    _ => {}
                },
                InputMode::EditingUpperBound | InputMode::EditingLowerBound if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        app.modify_input_mode(InputMode::Normal).await;
                    },
                    KeyCode::Char(to_insert) => {
                        if let InputMode::EditingLowerBound = app.get_input_mode().await {
                            app.modify_bounds(((app.get_bounds().await.0.0 + 1, format!("{}{}", app.get_bounds().await.0.1, to_insert)), app.get_bounds().await.1)).await;
                        } else {
                            app.modify_bounds((app.get_bounds().await.0, (app.get_bounds().await.1.0 + 1, format!("{}{}", app.get_bounds().await.1.1, to_insert)))).await;
                        }
                    },
                    KeyCode::Backspace => {
                        if let InputMode::EditingLowerBound = app.get_input_mode().await {
                            app.modify_bounds((((app.get_bounds().await.0.0 - 1).clamp(0, std::i32::MAX), app.get_bounds().await.0.1[..app.get_bounds().await.0.1.len().saturating_sub(1)].to_string()), app.get_bounds().await.1)).await;
                        } else {
                            app.modify_bounds((app.get_bounds().await.0,((app.get_bounds().await.1.0 - 1).clamp(0, std::i32::MAX), app.get_bounds().await.1.1[..app.get_bounds().await.1.1.len().saturating_sub(1)].to_string()))).await;
                        }
                    },
                    KeyCode::Tab => { 
                        match app.get_input_mode().await {
                        InputMode::EditingUpperBound => app.modify_input_mode(InputMode::EditingLowerBound).await,
                        InputMode::EditingLowerBound => app.modify_input_mode(InputMode::EditingUpperBound).await,
                        _ => {}
                        }
                    },
                    _ => {}
                },
                InputMode::EditingQuery | InputMode::EditingLowerBound | InputMode::EditingUpperBound => {}
            }

            match app.get_current_screen().await {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('q') => {
                        if app.get_input_mode().await == InputMode::EditingQuery 
                        || app.get_input_mode().await == InputMode::EditingLowerBound
                        || app.get_input_mode().await == InputMode::EditingUpperBound 
                        { continue; } // editing should ALWAYS take input priority
                        app.modify_current_screen(CurrentScreen::Exiting).await;
                    }
                    KeyCode::Char('p') => {
                        if app.get_input_mode().await == InputMode::EditingQuery 
                        || app.get_input_mode().await == InputMode::EditingLowerBound
                        || app.get_input_mode().await == InputMode::EditingUpperBound 
                        { continue; }
                        app.modify_current_screen(CurrentScreen::SelectingProcess).await;
                    }
                    KeyCode::Char('j') | KeyCode::Up => {
                        app.modify_mem_view_list("prev", None).await;
                    }
                    KeyCode::Char('k') | KeyCode::Down => {
                        app.modify_mem_view_list("next", None).await;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    KeyCode::Char('c') => {
                        app.modify_current_screen(CurrentScreen::Main).await;
                    }
                    _ => {}
                }
                CurrentScreen::SelectingProcess => match key.code {
                    KeyCode::Char('q') => {
                        app.modify_current_screen(CurrentScreen::Main).await;
                    }
                    KeyCode::Char('j') | KeyCode::Up => {
                        app.modify_proc_list("prev", None).await;
                    }
                    KeyCode::Char('k') | KeyCode::Down => {
                        app.modify_proc_list("next", None).await;
                    }
                    KeyCode::Char('c') => {
                        let processes = get_process_list();
                        let Some(idx) = app.get_proc_list().await.state.selected()
                        else { continue; };
                        app.modify_process(processes[idx].1 as i32).await;
                        app.modify_current_screen(CurrentScreen::Main).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

