use std::io::stdout;
use std::sync::Arc;
use std::time::Duration;

use futures::{StreamExt, pin_mut};

use app::{App, AppReturn};
use eyre::{eyre, WrapErr, Result};
use inputs::events::Events;
use inputs::InputEvent;
use io::IoEvent;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use log::warn;

use crate::app::ui;

pub mod app;
pub mod inputs;
pub mod io;

use matrix_sdk::{Client, SlidingSyncState};

pub async fn run_sliding_sync(client: Client, sliding_sync_proxy: String, app: Arc<tokio::sync::Mutex<App>>) -> Result<()> {

    warn!("Starting sliding sync now");
    let mut view = client.sliding_sync();
    view.set_homeserver(Some(sliding_sync_proxy.parse().wrap_err("can't parse sync proxy")?));
    let stream = view.stream().expect("we can build the stream");
    pin_mut!(stream);
    {
        let mut app = app.lock().await;
        app.state_mut().start_sliding(view.clone());
    }
    let first_poll = stream.next().await;
    if  !matches!(first_poll, Some(Ok(SlidingSyncState::CatchingUp))) {
        warn!("Sliding Query failed: {:#?}", first_poll);
        return Ok(())
    }

    {
        let mut app = app.lock().await;
        let mut sliding = app.state_mut().get_sliding_mut().expect("we started this before!");
        sliding.set_first_render_now();
    }
    warn!("Done initial sliding sync");

    loop {
        match stream.next().await {
            Some(Ok(SlidingSyncState::Live)) => {
                // we are switching into live updates mode next. ignoring
                warn!("Reached live sync");
                break
            }
            Some(Err(e)) => {
                warn!("Error: {:}", e);
                break
            }
            Some(_) => { }
            None => {
                warn!("Never reached live state");
                break;
            }
        }
    }

    {
        let mut app = app.lock().await;
        let mut sliding = app.state_mut().get_sliding_mut().expect("we started this before!");
        sliding.set_full_sync_now();
    }
    Ok(())
}

pub async fn run_client(client: Client, sliding_sync_proxy: String, app: Arc<tokio::sync::Mutex<App>>) -> Result<()> {

    let username = match client.account().get_display_name().await? {
        Some(u) => u,
        None => client.user_id().await.ok_or_else(||eyre!("Looks like you didn't login"))?.to_string()
    };

    let homeserver = client.homeserver().await;

    {
        let mut app = app.lock().await;
        app.set_title(Some(format!("{} on {} via {}", username, homeserver, sliding_sync_proxy))).await;
    }

    run_sliding_sync(client, sliding_sync_proxy, app).await?;
    Ok(())
}

pub async fn run_syncv2(client: Client,  app: Arc<tokio::sync::Mutex<App>>) -> Result<()> {
    {
        let mut app = app.lock().await;
        app.state_mut().start_v2();
    }

    warn!("Starting v2 sync now");
    let res = client.sync_once(Default::default()).await?;
    warn!("Done v2 sync");

    {
        let mut app = app.lock().await;
        let v2 = app.state_mut().get_v2_mut().expect("we started this before!");
        v2.set_first_render_now();
        let total_rooms = res.rooms.join.len() + res.rooms.leave.len() + res.rooms.invite.len();
        v2.set_rooms_count(total_rooms as u32); 
    }

    Ok(())
}

pub async fn start_ui(app: &Arc<tokio::sync::Mutex<App>>) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // User event handler
    let tick_rate = Duration::from_millis(450); // render twice per second
    let mut events = Events::new(tick_rate);

    // Trigger state change from Init to Initialized
    {
        let mut app = app.lock().await;
        // Here we assume the the first load is a long task
        app.dispatch(IoEvent::Initialize).await;
    }

    loop {
        let mut app = app.lock().await;

        // Render
        terminal.draw(|rect| ui::draw(rect, &app))?;

        // Handle inputs
        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key).await,
            InputEvent::Tick => AppReturn::Continue,
        };
        // Check if we should exit
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}