// Imports
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::prelude::*;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
//Block types
enum BlockTypes {
    FIRST,
    LAST,
    PUNISH,
    GOOD,
    NORMAL,
}
//Define App structure, to use variables inside the main function.
struct App {
    players: Vec<Player>,
    current_index: i16,
    info: Vec<String>,
    finished: bool,
    winner: String,
}
impl App {
    //Function to get player
    fn get_player(&self, index: i16) -> &Player {
        return self.players.get(index as usize).unwrap();
    }
    //Default values for App
    fn default() -> App {
        App {
            players: vec![
                init_player(String::from("J1"), 0),
                init_player(String::from("J2"), 0),
                init_player(String::from("J3"), 0),
                init_player(String::from("J4"), 0),
            ],
            current_index: 0,
            info: vec![],
            finished: false,
            winner: "".to_string(),
        }
    }
}

//Player Structure
pub struct Player {
    name: String,
    cell: i16,
}

//Functions/Implementations for Player
impl Player {
    fn get_name(&self) -> String {
        return self.name.clone();
    }
}

//Quick way to init a player
fn init_player(username: String, cell: i16) -> Player {
    Player {
        name: username,
        cell: cell,
    }
}
//Main function
fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

//Run function
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    //Main game loop
    while !app.finished {
        //draw ui
        terminal.draw(|f| tablero(f, &app))?;
        //On key press
        if let Event::Key(key) = event::read()? {
            //Key == Q then quit
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            } else if let KeyCode::Char('c') = key.code {
                //Get random value (1 to 6 )
                let n: i16 = rand::thread_rng().gen_range(1..7);
                //Get current player information
                let current_player = app.get_player(app.current_index);
                //Player name
                let player_name = current_player.get_name();
                //Check if player is over limits
                if app.players[app.current_index as usize].cell + n > 63 {
                    app.info.push(String::from(format!(
                        " El {} acaba de sacar el numero {}. Pero necesita un {} para ganar.",
                        player_name,
                        n,
                        63 - app.players[app.current_index as usize].cell
                    )));
                } else {
                    //If not, run normally
                    app.players[app.current_index as usize].cell += n;
                    app.info.push(String::from(format!(
                        " El {} acaba de sacar el numero {}.",
                        player_name, n
                    )));
                }
                //Check if player won
                if app.players[app.current_index as usize].cell == 63 {
                    //Set player name as winner
                    app.winner = player_name.clone();
                    app.info
                        .push(String::from(format!("El jugador {} ganó.", player_name)));
                    break;
                }
                // Special cells
                if app.players[app.current_index as usize].cell % 5 == 0 {
                    app.players[app.current_index as usize].cell -= 2;
                    app.info.push(String::from(format!(
                        " El {} esta en un casillero de castigo. Retrocede 2 casilleros.",
                        player_name
                    )));
                } else if app.players[app.current_index as usize].cell % 7 == 0 {
                    app.players[app.current_index as usize].cell += 2;
                    app.info.push(String::from(format!(
                        " El {} esta en un casillero de castigo. Avanza 2 casilleros.",
                        player_name
                    )));
                }
                //Update current index
                if app.current_index >= (app.players.len() as i16 - 1) {
                    app.current_index = 0;
                } else {
                    app.current_index += 1;
                }
            }
        }
    }
    //When game's finished, wait until user inputs Q to quit the program.
    loop {
        //Draw winners screen
        terminal
            .draw(|f| winner_screen(f, String::from(format!("Ganó el jugador {}", app.winner))))?;
        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                //Quit
                return Ok(());
            }
        };
    }
}
//Function to generate percentage on screen
fn cells_to_percentage(cells: i16) -> Vec<Constraint> {
    let mut vector: Vec<Constraint> = vec![];
    let percentage = 100 / cells;
    for _n in 0..=cells {
        vector.push(Constraint::Percentage(percentage as u16))
    }
    return vector;
}

//Scalable function to generate blocks by their type.
fn generate_block(block_type: BlockTypes, title: String) -> Block<'static> {
    fn default_block(title: String, color: Color, border_type: BorderType) -> Block<'static> {
        return Block::default()
            .title(title)
            .border_style(Style::default().fg(color))
            .borders(Borders::ALL)
            .border_type(border_type);
    }
    //FIRST
    if matches!(block_type, BlockTypes::FIRST) {
        return default_block(title, Color::Green, BorderType::Thick);
    //LAST
    } else if matches!(block_type, BlockTypes::LAST) {
        return default_block(title, Color::Red, BorderType::Double);
    } else if matches!(block_type, BlockTypes::PUNISH) {
        return default_block(title, Color::LightRed, BorderType::Rounded);
    } else if matches!(block_type, BlockTypes::GOOD) {
        return default_block(title, Color::Yellow, BorderType::Plain);
    //NORMAL
    } else {
        return default_block(title, Color::Cyan, BorderType::Plain);
    }
}
//Main game UI
fn tablero<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let rows = 8;
    let max_columns = 8;
    // Chunks to render blocks.
    let rows_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints(cells_to_percentage(rows).as_ref())
        .margin(4)
        .vertical_margin(2)
        .split(f.size());
    let columns_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(4)
        .constraints(cells_to_percentage(max_columns).as_ref())
        .split(f.size());
    // Main block
    let block = Block::default()
        .borders(Borders::ALL)
        .title("   Presione q para salir                            El juego de la OCA -- Rust v1                            Presione C para utilizar el turno   ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    //Render main block
    f.render_widget(block, size);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(83),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(f.size());
    //iterate over cells to render them in screen
    for n_rows in 0..rows {
        for n_col in 0..max_columns {
            let current_index = n_col + n_rows * max_columns;
            //Generate block by their index
            let block = generate_block(
                if n_rows == 0 && n_col == 0 {
                    BlockTypes::FIRST
                } else if n_rows == rows - 1 && n_col == max_columns - 1 {
                    BlockTypes::LAST
                } else if current_index % 5 == 0 {
                    BlockTypes::PUNISH
                } else if current_index % 7 == 0 {
                    BlockTypes::GOOD
                } else {
                    BlockTypes::NORMAL
                },
                format!(
                    "{}{}",
                    current_index,
                    if current_index == 0 {
                        "-Inicio"
                    } else if current_index == 63 {
                        "-Fin"
                    } else if current_index % 5 == 0 {
                        "-Castigo"
                    } else if current_index % 7 == 0 {
                        "-Suerte"
                    } else {
                        ""
                    }
                )
                .to_string(),
            );
            //Convert players names in the same cell into a string separed by spaces
            let mut text_to_render = String::from("");
            for c_player in app.players.iter() {
                if c_player.cell == current_index {
                    text_to_render = text_to_render + " " + &*c_player.name
                }
            }
            //Generate paragraph block by the text created
            let paragraph = Paragraph::new(text_to_render)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: false })
                .block(block)
                .alignment(Alignment::Center);
            let row_area = rows_chunk[n_rows as usize];
            let col_area = columns_chunk[n_col as usize];
            f.render_widget(
                paragraph,
                Rect::new(col_area.x, row_area.y, col_area.width, row_area.height),
            );
        }

        //Render information about the game in the bottom
        let information_lines: Vec<Spans> = app
            .info
            .iter()
            .map(|text| Spans::from(text.clone()))
            .rev()
            .collect();

        let information_block = Paragraph::new(information_lines)
            .block(Block::default().title("Information").borders(Borders::ALL));
        f.render_widget(information_block, chunks[2]);
    }
}

//Winner screen
fn winner_screen<B: Backend>(f: &mut Frame<B>, text: String) {
    let size = f.size();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Ganador   Presione Q para salir")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(
        Paragraph::new(Spans::from(Span::styled(
            text,
            Style::default().bg(Color::Blue),
        )))
        .block(block),
        size,
    );
}
