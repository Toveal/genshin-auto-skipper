//! Global hotkeys via `RegisterHotKey`, serviced by a dedicated message thread.
//! Unlike a low-level keyboard hook this does not observe every keystroke.

use std::{
    io, mem, ptr,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use windows_sys::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::{
        Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT},
        WindowsAndMessaging::{
            GetMessageW, PeekMessageW, PostThreadMessageW, MSG, PM_NOREMOVE, WM_APP, WM_HOTKEY,
            WM_QUIT, WM_USER,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotkeyAction {
    Start,
    Pause,
    Exit,
}

impl HotkeyAction {
    const ALL: [HotkeyAction; 3] = [Self::Start, Self::Pause, Self::Exit];

    fn id(self) -> i32 {
        self as i32 + 1
    }

    fn from_id(id: usize) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.id() as usize == id)
    }
}

#[derive(Debug)]
pub enum HotkeyEvent {
    Pressed(HotkeyAction),
    /// The key is probably taken by another application.
    RegisterFailed(HotkeyAction, u16),
}

/// Virtual-key code per action; `None` disables the hotkey.
pub type Bindings = Vec<(HotkeyAction, Option<u16>)>;

const WM_REBIND: u32 = WM_APP + 1;

pub struct Hotkeys {
    thread_id: u32,
    bindings: Arc<Mutex<Bindings>>,
    handle: Option<JoinHandle<()>>,
}

impl Hotkeys {
    pub fn spawn(
        bindings: Bindings,
        on_event: impl Fn(HotkeyEvent) + Send + 'static,
    ) -> io::Result<Self> {
        let bindings = Arc::new(Mutex::new(bindings));
        let (ready_tx, ready_rx) = mpsc::channel();
        let thread_bindings = Arc::clone(&bindings);

        let handle = thread::Builder::new()
            .name("hotkeys".into())
            .spawn(move || {
                let mut msg: MSG = unsafe { mem::zeroed() };
                // Force creation of the thread's message queue before anyone
                // posts to it.
                unsafe { PeekMessageW(&mut msg, ptr::null_mut(), WM_USER, WM_USER, PM_NOREMOVE) };
                let _ = ready_tx.send(unsafe { GetCurrentThreadId() });

                let mut registered = register_all(&thread_bindings, &on_event);
                loop {
                    match unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } {
                        0 | -1 => break,
                        _ => {}
                    }
                    match msg.message {
                        WM_HOTKEY => {
                            if let Some(action) = HotkeyAction::from_id(msg.wParam) {
                                on_event(HotkeyEvent::Pressed(action));
                            }
                        }
                        WM_REBIND => {
                            unregister_all(&registered);
                            registered = register_all(&thread_bindings, &on_event);
                        }
                        _ => {}
                    }
                }
                unregister_all(&registered);
            })?;

        let thread_id = ready_rx
            .recv()
            .map_err(|_| io::Error::other("hotkey thread exited during startup"))?;

        Ok(Self {
            thread_id,
            bindings,
            handle: Some(handle),
        })
    }

    pub fn rebind(&self, bindings: Bindings) {
        *self.bindings.lock().unwrap_or_else(|e| e.into_inner()) = bindings;
        unsafe { PostThreadMessageW(self.thread_id, WM_REBIND, 0, 0) };
    }
}

impl Drop for Hotkeys {
    fn drop(&mut self) {
        unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0) };
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn register_all(bindings: &Mutex<Bindings>, on_event: &impl Fn(HotkeyEvent)) -> Vec<HotkeyAction> {
    let bindings = bindings.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut registered = Vec::new();
    for (action, vk) in bindings {
        let Some(vk) = vk else { continue };
        let ok =
            unsafe { RegisterHotKey(ptr::null_mut(), action.id(), MOD_NOREPEAT, vk as u32) } != 0;
        if ok {
            registered.push(action);
        } else {
            on_event(HotkeyEvent::RegisterFailed(action, vk));
        }
    }
    registered
}

fn unregister_all(registered: &[HotkeyAction]) {
    for action in registered {
        unsafe { UnregisterHotKey(ptr::null_mut(), action.id()) };
    }
}
