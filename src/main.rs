mod event;

use pam::Authenticator;

use libc::{endpwent, getpwnam};
use std::ffi::CString;

use log::{debug, info, warn};

use std::{
    error::Error,
    io::{self, Write},
};
use termion::{
    cursor::Goto, event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen,
};
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

use clap::{App, Arg};
use tui::backend::CrosstermBackend;

enum FocussedWidget {
    Login,
    Pass,
}

struct AppState {
    login: String,
    pass: String,
    focussed_widget: FocussedWidget,
    available_des: Vec<String>,
    selected_de_index: usize,
    failed_logins: usize,
    running: bool,
}

impl AppState {
    fn auth(&mut self) {
        // let mut authenticator =
        //     Authenticator::with_password("system-auth").expect("Failed to init PAM client.");
        // authenticator
        //     .get_handler()
        //     .set_credentials(&self.login, &self.pass);

        // self.pass.clear();

        // match authenticator.authenticate() {
        //     Ok(()) => {}
        //     Err(err) => {
        //         warn!("Authentication failed! {}", err);
        //         self.failed_logins += 1;
        //         return;
        //     }
        // };

        // authenticator
        //     .open_session()
        //     .expect("Failed to open a session!");

        self.running = false;

        let test;
        test = unsafe {
            getpwnam(
                CString::new(self.login.as_bytes())
                    .expect("CString: new failed")
                    .as_ptr(),
            );
            endpwent();
        };
        debug!("{:?}", test);
    }
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            login: String::new(),
            pass: String::new(),
            focussed_widget: FocussedWidget::Login,
            available_des: ["test1".to_string(), "sway".to_string(), "i3".to_string()].to_vec(),
            selected_de_index: 0,
            failed_logins: 0,
            running: true,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = App::new("ry")
        .version("1.0") // TODO: Handle the version
        .author("MatejKastak <matej.kastak@gmail.com>")
        .about("")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .about("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    // let backend = TermionBackend::new(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let events = event::Events::new();

    // Create default app state
    let mut app = AppState::default();

    while app.running {
        // Draw UI
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let msg = app.available_des.get(app.selected_de_index).unwrap();
            let text = [Text::raw(msg)];
            let help_message = Paragraph::new(text.iter());
            f.render_widget(help_message, chunks[0]);

            let login_text = [Text::raw(&app.login)];
            let login_input = Paragraph::new(login_text.iter())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(login_input, chunks[1]);

            let star_len = app.pass.len();
            let mut star_text = String::with_capacity(star_len);
            for _ in 0..star_len {
                star_text.push('*');
            }
            let pass_text = [Text::raw(&star_text)];
            let pass_input = Paragraph::new(pass_text.iter())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(pass_input, chunks[2]);
        })?;

        // Calculate the correct possition of the cursor
        match app.focussed_widget {
            FocussedWidget::Login => {
                write!(
                    terminal.backend_mut(),
                    "{}",
                    Goto(4 + app.login.width() as u16, 5)
                )?;
            }
            FocussedWidget::Pass => {
                write!(
                    terminal.backend_mut(),
                    "{}",
                    Goto(4 + app.pass.width() as u16, 8)
                )?;
            }
        };
        // stdout is buffered, flush it to see the effect immediately when hitting backspace
        io::stdout().flush().ok();

        // Handle input
        if let event::Event::Input(input) = events.next()? {
            match input {
                Key::F(1) => {
                    // Shutdown
                    app.running = false;
                }
                Key::F(2) => {
                    // Reboot
                }
                Key::Right => {
                    app.selected_de_index = (app.selected_de_index + 1) % app.available_des.len()
                }
                Key::Left => {
                    app.selected_de_index = if app.selected_de_index != 0 {
                        app.selected_de_index - 1
                    } else {
                        app.available_des.len() - 1
                    };
                }
                _ => {}
            }

            match app.focussed_widget {
                FocussedWidget::Login => match input {
                    Key::Char('\n') | Key::Char('\t') => {
                        app.focussed_widget = FocussedWidget::Pass;
                    }
                    Key::Char(c) => {
                        app.login.push(c);
                    }
                    Key::Backspace => {
                        app.login.pop();
                    }
                    _ => {}
                },
                FocussedWidget::Pass => match input {
                    Key::Char('\n') => {
                        app.auth();
                        // Attempt to login
                    }
                    Key::Char('\t') => {
                        app.focussed_widget = FocussedWidget::Login;
                    }
                    Key::Char(c) => {
                        app.pass.push(c);
                    }
                    Key::Backspace => {
                        app.pass.pop();
                    }





                    _ => {}
                },
            }
        }
    }
    Ok(())
}
