use clap::Parser;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, poll},
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Bar, BarChart, BarGroup},
};
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::task::JoinSet;
use tokio_postgres::NoTls;

#[derive(Parser)]
struct Args {
    #[clap(default_value_t = 90)]
    fast_tasks: usize,
    #[clap(default_value_t = 5)]
    slow_tasks: usize,
}

const POLL_TIME: Duration = Duration::from_millis(250);

async fn postgres_worker(counter: Arc<AtomicU32>, sleep_range: Range<u64>) -> Result<()> {
    let (mut client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    loop {
        let sleep_value = rand::random_range(sleep_range.clone());

        let tx = client.transaction().await?;
        tx.query("LOCK TABLE customers IN ACCESS SHARE MODE", &[])
            .await?;

        tokio::time::sleep(Duration::from_millis(sleep_value)).await;
        counter.fetch_add(1, Ordering::Relaxed);
        tx.commit().await?;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    color_eyre::install()?;

    let res = run_app(args).await;
    ratatui::restore();
    res
}

async fn run_app(args: Args) -> Result<()> {
    let counter = Arc::new(AtomicU32::new(0));

    let mut tasks = JoinSet::new();

    for _ in 0..args.fast_tasks {
        tasks.spawn(tokio::task::spawn(postgres_worker(
            counter.clone(),
            100..2_000,
        )));
    }
    for _ in 0..args.slow_tasks {
        tasks.spawn(tokio::task::spawn(postgres_worker(
            counter.clone(),
            10_000..15_000,
        )));
    }

    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();

    let terminal_app =
        tokio::task::spawn_blocking(move || run_terminal_app(&counter, should_exit_clone.clone()));

    tokio::select! {
        res = terminal_app => {
            return res?
        },

        res = tasks.join_next() => {
            if let Some(res) = res {
                should_exit.store(true, Ordering::SeqCst);
                return res??;
            }
        }
    }

    Ok(())
}

fn run_terminal_app(counter: &AtomicU32, should_exit: Arc<AtomicBool>) -> Result<()> {
    let terminal = ratatui::init();
    let app_result = App::new(counter, &should_exit).run(terminal);
    ratatui::restore();
    app_result
}

struct App<'a> {
    should_exit: &'a AtomicBool,
    counter: &'a AtomicU32,
    values: Vec<u32>,
}

impl<'a> App<'a> {
    fn new(counter: &'a AtomicU32, should_exit: &'a AtomicBool) -> Self {
        Self {
            should_exit,
            counter,
            values: vec![],
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit.load(Ordering::Relaxed) {
            let current_counter = self.counter.swap(0, Ordering::Relaxed);
            self.values.push(current_counter);
            let current_width = (terminal.size()?.width / 3) as usize;
            if self.values.len() > (current_width - 2) {
                self.values.remove(0);
            }

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if poll(POLL_TIME)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('q')
        {
            self.should_exit.store(true, Ordering::Relaxed);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let [title, vertical] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .spacing(1)
            .areas(frame.area());

        frame.render_widget("Query Rate".bold().into_centered_line(), title);
        frame.render_widget(vertical_barchart(&self.values), vertical);
    }
}

fn vertical_barchart(data: &[u32]) -> BarChart<'_> {
    let bars: Vec<Bar> = data.iter().map(|value| vertical_bar(*value)).collect();
    // let title = Line::from("Query Rate").centered();
    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(2)
        .bar_gap(1)
    // .block(Block::new().title(title))
}

fn vertical_bar(value: u32) -> Bar<'static> {
    Bar::default()
        .value(value as u64)
        .text_value("".to_string())
        .label(Line::from(format!("{value}")))
}
