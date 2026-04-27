use crate::app::{AppContext, AppError};
use crate::core::store::Store;
use crate::output::ErrorCode;
use crate::tui::{Effect, Model, Msg, editor, effect, keymap, update, view};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::collections::VecDeque;
use std::io::{self, IsTerminal};
use std::time::Duration;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

type TuiTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub fn run(ctx: AppContext) -> Result<(), AppError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(());
    }

    let mut store = ctx.store;
    let mut model = Model::new(ctx.config.clone());
    let mut queue = VecDeque::new();

    let mut terminal = setup_terminal()?;
    let _guard = TerminalGuard::enter()?;

    let (width, height) =
        terminal::size().map_err(tui_io_error("failed to query terminal size"))?;
    queue.push_back(Msg::Resize(width, height));
    queue.extend(effect::execute(Effect::LoadIssues, &mut store));

    process_queue(&mut model, &mut store, &mut queue, &mut terminal)?;
    draw(&mut terminal, &model)?;

    while !model.quit {
        if event::poll(POLL_TIMEOUT).map_err(tui_io_error("failed to poll for terminal events"))? {
            if let Some(msg) = read_event(&model)? {
                queue.push_back(msg);
            }
        } else {
            queue.push_back(Msg::Tick);
        }

        process_queue(&mut model, &mut store, &mut queue, &mut terminal)?;
        draw(&mut terminal, &model)?;
    }

    Ok(())
}

fn setup_terminal() -> Result<TuiTerminal, AppError> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(tui_io_error("failed to initialize terminal backend"))
}

fn process_queue(
    model: &mut Model,
    store: &mut Store,
    queue: &mut VecDeque<Msg>,
    terminal: &mut TuiTerminal,
) -> Result<(), AppError> {
    while let Some(msg) = queue.pop_front() {
        match msg {
            Msg::EditorRequested(request) => {
                let result =
                    open_requested_editor(store, &request.id).map_err(|error| error.message);
                terminal
                    .clear()
                    .map_err(tui_io_error("failed to clear terminal after editor"))?;
                queue.push_back(Msg::EditorReturned(result));
                queue.extend(effect::execute(Effect::LoadIssues, store));
            }
            Msg::Followup(effect) => {
                queue.extend(effect::execute(effect, store));
            }
            msg => {
                let current = std::mem::replace(model, Model::new(model.config.clone()));
                let (next_model, effects) = update::update(current, msg);
                *model = next_model;

                for effect in effects {
                    queue.extend(run_effect(effect, store));
                }
            }
        }
    }
    Ok(())
}

fn run_effect(effect_to_run: Effect, store: &mut Store) -> Vec<Msg> {
    effect::execute(effect_to_run, store)
}

fn open_requested_editor(store: &Store, id: &str) -> Result<(), AppError> {
    let issue = store.get(id).ok_or_else(|| {
        AppError::new(
            ErrorCode::NotFound,
            format!("unable to open editor for missing issue `{id}`"),
        )
    })?;
    let path = store.root().join(&issue.path);
    editor::open_editor(&path)
}

fn read_event(model: &Model) -> Result<Option<Msg>, AppError> {
    let event = event::read().map_err(tui_io_error("failed to read terminal event"))?;

    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => {
            Ok(model
                .screens
                .last()
                .and_then(|screen| keymap::map_key(screen, key)))
        }
        Event::Resize(width, height) => Ok(Some(Msg::Resize(width, height))),
        _ => Ok(None),
    }
}

fn draw(terminal: &mut TuiTerminal, model: &Model) -> Result<(), AppError> {
    terminal
        .draw(|frame| view::draw(frame, model))
        .map(|_| ())
        .map_err(tui_io_error("failed to draw tui frame"))
}

fn tui_io_error(context: &'static str) -> impl Fn(std::io::Error) -> AppError {
    move |error| AppError::new(ErrorCode::FileError, format!("{context}: {error}"))
}

struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    fn enter() -> Result<Self, AppError> {
        terminal::enable_raw_mode().map_err(tui_io_error("failed to enable raw mode"))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)
            .map_err(tui_io_error("failed to enter alternate screen"))?;
        Ok(Self { active: true })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let _ = terminal::disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
    }
}
