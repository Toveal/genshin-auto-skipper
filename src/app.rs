use color_eyre::{eyre::Context, Report, Result};

use std::{
    sync::{
        mpsc::{channel, Sender, TryRecvError},
        Arc, Mutex, MutexGuard, PoisonError,
    },
    thread::JoinHandle,
    time::Duration,
};
use thiserror::Error;

use crate::{
    genshin::{Window, WindowProps},
    global_states::{EventType, EVENT_LISTENER_CHANNEL},
    winapi_bindings::{
        hooks::{ApplicationShutdown, ChangeForegroundWindow, DestroyWindow, KeyboardEvent},
        message_manager::MessageManager,
    },
};

#[derive(Error, Debug)]
enum AppErr {
    #[error("Error when locking mutex: {0}")]
    MutexBlockErr(String),
    #[error("Error setting global event listener")]
    SendEventListenerError,
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for AppErr {
    fn from(error: PoisonError<MutexGuard<'_, T>>) -> Self {
        AppErr::MutexBlockErr(error.to_string())
    }
}

#[derive(Debug)]
pub struct App;

impl App {
    pub fn run() -> Result<()> {
        let (event_sender, event_receiver) = channel();
        // To receive messages from Windows Hook
        if EVENT_LISTENER_CHANNEL.set(event_sender).is_err() {
            return Err(AppErr::SendEventListenerError.into());
        };

        Self::draw_menu();
        // Receives shutdown messages
        ApplicationShutdown::set()?;

        // To see if the stream is alive
        let (stream_state_event_sender, stream_state_event_reciever) = channel();
        let _event_listener = Self::start_event_listener(stream_state_event_sender)?;

        stream_state_event_reciever
            .recv()
            .wrap_err("Thread listener disconnected")?
            .wrap_err("Error inside the event listener")?;

        // Auxiliary data for working with genshin in a separate thread
        let send_message = Arc::new(Mutex::new(false));
        let genshin_window_props = Arc::new(Mutex::new(None));
        let genshin_window = Arc::new(Mutex::new(None));

        let (stream_state_message_sender, stream_state_message_reciever) = channel();

        let _message_sender = Self::start_message_sender(
            stream_state_message_sender,
            send_message.clone(),
            genshin_window_props.clone(),
            genshin_window.clone(),
        )?;

        let find_genshin = || -> Result<()> {
            let hwnd_result = Window::new();
            match hwnd_result {
                Ok(w) => *genshin_window.lock().map_err(AppErr::from)? = Some(w),
                _ => *genshin_window.lock().map_err(AppErr::from)? = None,
            };
            Ok(())
        };

        let calculate_props = || -> Result<()> {
            if let Some(w) = *genshin_window.lock().map_err(AppErr::from)? {
                let window_props = WindowProps::new(&w)?;
                *genshin_window_props.lock().map_err(AppErr::from)? = Some(window_props);
            }
            Ok(())
        };

        // First attempt to find the window and calculate the properties
        find_genshin()?;
        calculate_props()?;
        let mut enable_send_message = false;
        while let Ok(event) = event_receiver.recv() {
            match event {
                EventType::KeyPress(key) => match key {
                    // F9
                    120 if !enable_send_message => {
                        enable_send_message = true;
                    }
                    // F10
                    121 if enable_send_message => {
                        enable_send_message = false;
                    }
                    // F11
                    122 => break,
                    _ => {
                        continue;
                    }
                },
                EventType::ChangeForegroundWindow => {
                    find_genshin()?;
                    calculate_props()?;
                }
                EventType::DestroyWindow => {
                    find_genshin()?;
                    calculate_props()?;
                }
                EventType::Shutdown => break,
            }

            // If an error occurred in the event listener or it is no longer available
            match stream_state_event_reciever.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    return Err(Report::new(TryRecvError::Disconnected)
                        .wrap_err("Thread listener disconnected"));
                }
                Ok(e) => e?,
                _ => {}
            }

            // If an error occurred in the messeg sender thread or it is no longer available
            match stream_state_message_reciever.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    return Err(Report::new(TryRecvError::Disconnected)
                        .wrap_err("Thread send message disconnected"))
                }
                Ok(e) => e?,
                _ => {}
            }

            // Checking whether the genshin is working and whether the key is pressed
            *send_message.lock().map_err(AppErr::from)? = if enable_send_message {
                if let Some(w) = &*genshin_window.lock().map_err(AppErr::from)? {
                    genshin_window_props.lock().map_err(AppErr::from)?.is_some() && w.is_active()
                } else {
                    false
                }
            } else {
                false
            };
        }

        Ok(())
    }

    fn start_event_listener(sender: Sender<Result<()>>) -> Result<JoinHandle<Result<()>>> {
        let handle = std::thread::Builder::new()
            .spawn(move || -> Result<(), Report> {
                let run = || -> Result<(), Report> {
                    let _h1 = KeyboardEvent::new()?;
                    let _h2 = ChangeForegroundWindow::new()?;
                    let _h3 = DestroyWindow::new()?;

                    sender.send(Ok(()))?;
                    let mut messenger = MessageManager::new();

                    while messenger.get_message().is_ok() {
                        messenger.translate_message()?;
                        messenger.dispatch_message();
                    }

                    Ok(())
                };

                if let Err(e) = run() {
                    sender.send(Err(e))?;
                }

                Ok(())
            })
            .wrap_err("Error starting the event listener")?;
        Ok(handle)
    }

    fn start_message_sender(
        sender: Sender<Result<()>>,
        send_message: Arc<Mutex<bool>>,
        genshin_window_props: Arc<Mutex<Option<WindowProps>>>,
        genshin_window: Arc<Mutex<Option<Window>>>,
    ) -> Result<JoinHandle<Result<()>>> {
        let handle = std::thread::Builder::new()
            .spawn(move || -> Result<(), Report> {
                let run = || -> Result<()> {
                    loop {
                        if !*send_message.lock().map_err(AppErr::from)? {
                            std::thread::sleep(Duration::from_millis(50));
                            continue;
                        };

                        let genshin_window = *genshin_window.lock().map_err(AppErr::from)?;
                        let window_props = *genshin_window_props.lock().map_err(AppErr::from)?;

                        if let Some(window) = genshin_window {
                            if let Some(win_props) = window_props {
                                if window.dialog_played(&win_props)? {
                                    // If the character is talking and you don't need to select anything,
                                    // we send the space bar to avoid pulling the cursor.
                                    if window.is_dialog_without_option(&win_props)? {
                                        window.click_space()?;
                                    } else {
                                        window.click_left_m_button_random_pos(&win_props)?;
                                    }
                                } else {
                                    std::thread::sleep(Duration::from_millis(50));
                                    continue;
                                }
                            }
                        }
                    }
                };

                if let Err(e) = run() {
                    sender.send(Err(e))?;
                }

                Ok(())
            })
            .wrap_err("Error starting the message sender")?;

        Ok(handle)
    }

    fn draw_menu() {
        let menu = r"
        ==================================
        |    Genshin Auto-Skip Dialogs   |
        ==================================
        
        **********************************
        *          KEY BINDINGS          *
        **********************************
        * F9  - Run                      *
        * F10 - Pause                    *
        * F11 - Exit                     *
        **********************************
        ";

        for line in menu.trim().lines() {
            println!("{}", line.trim());
        }
    }
}
