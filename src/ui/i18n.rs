//! UI strings in English and Russian.

use crate::{
    engine::state::LogMsg,
    platform::{hotkeys::HotkeyAction, keys},
    settings::Language,
};

pub struct Texts {
    pub status_paused: &'static str,
    pub status_running: &'static str,
    pub status_break: &'static str,
    pub game_not_found: &'static str,
    pub game_minimized: &'static str,
    pub game_focused: &'static str,
    pub game_unfocused: &'static str,
    pub profile_wide: &'static str,
    pub profile_measured: &'static str,
    pub start: &'static str,
    pub pause: &'static str,
    pub stat_lines: &'static str,
    pub stat_presses: &'static str,
    pub stat_choices: &'static str,
    pub stat_time: &'static str,
    pub not_admin: &'static str,
    pub help: &'static str,
    pub help_steps: &'static [&'static str],
    pub settings: &'static str,
    pub group_actions: &'static str,
    pub skip_mode: &'static str,
    pub skip_mode_detect: &'static str,
    pub skip_mode_detect_hint: &'static str,
    pub skip_mode_press: &'static str,
    pub skip_mode_press_hint: &'static str,
    pub ms: &'static str,
    pub mode: &'static str,
    pub mode_keyboard: &'static str,
    pub mode_mouse: &'static str,
    pub interact_key: &'static str,
    pub confirm_key: &'static str,
    pub group_hotkeys: &'static str,
    pub hotkey_start: &'static str,
    pub hotkey_pause: &'static str,
    pub hotkey_exit: &'static str,
    pub press_key: &'static str,
    pub unbound: &'static str,
    pub clear: &'static str,
    pub hotkey_conflict: &'static str,
    pub hotkey_reserved: &'static str,
    pub group_timing: &'static str,
    pub humanize: &'static str,
    pub humanize_hint: &'static str,
    pub interval: &'static str,
    pub breaks: &'static str,
    pub breaks_hint: &'static str,
    pub break_length: &'static str,
    pub group_other: &'static str,
    pub tolerance: &'static str,
    pub tolerance_hint: &'static str,
    pub sound: &'static str,
    pub always_on_top: &'static str,
    pub reset: &'static str,
    pub open_folder: &'static str,
    pub log: &'static str,
    pub log_empty: &'static str,
    pub already_running: &'static str,
    pub fatal: &'static str,
}

pub const EN: Texts = Texts {
    status_paused: "PAUSED",
    status_running: "RUNNING",
    status_break: "BREAK",
    game_not_found: "Game not found — start Genshin Impact",
    game_minimized: "Game is minimized",
    game_focused: "in focus",
    game_unfocused: "not in focus — waiting",
    profile_wide: "ultrawide profile",
    profile_measured: "measured profile",
    start: "Start",
    pause: "Pause",
    stat_lines: "Lines",
    stat_presses: "Presses",
    stat_choices: "Choices",
    stat_time: "Active",
    not_admin: "Not running as administrator: the game will ignore input. \
                Restart the program as administrator.",
    help: "How to use",
    help_steps: &[
        "In the game open Settings > Other and set “Auto-Play Story” to Off.",
        "Start the game, then press the start hotkey or the button above.",
        "Dialogues are advanced only while the game window is in focus.",
        "Lines are advanced with the interaction key; for options the bottom one is chosen.",
    ],
    settings: "Settings",
    group_actions: "Actions",
    skip_mode: "Mode",
    ms: "ms",
    skip_mode_detect: "Detect dialogues",
    skip_mode_detect_hint: "Press only when a dialogue is on screen.",
    skip_mode_press: "Just press the key",
    skip_mode_press_hint: "Press the interaction key at the random interval without \
                           looking at the screen. Works only while the game is in focus.",
    mode: "Choosing options",
    mode_keyboard: "Confirm key",
    mode_mouse: "Mouse click",
    interact_key: "Interaction key",
    confirm_key: "Confirm key",
    group_hotkeys: "Hotkeys",
    hotkey_start: "Start",
    hotkey_pause: "Pause",
    hotkey_exit: "Exit",
    press_key: "Press a key… (Esc — cancel)",
    unbound: "—",
    clear: "Clear",
    hotkey_conflict: "A hotkey matches an in-game key and would block it in the game.",
    hotkey_reserved: "F12 is reserved by Windows and cannot be a hotkey.",
    group_timing: "Timing",
    humanize: "Human-like timing",
    humanize_hint: "Random delay between presses; sometimes a longer one.",
    interval: "Delay, ms",
    breaks: "Occasional breaks",
    breaks_hint: "Every 30 s there is a small chance to pause for a few seconds.",
    break_length: "Break, s",
    group_other: "Other",
    tolerance: "Color tolerance",
    tolerance_hint: "Increase if dialogues are not detected (HDR, filters, color profiles).",
    sound: "Sound on start/pause",
    always_on_top: "Always on top",
    reset: "Reset to defaults",
    open_folder: "Open settings folder",
    log: "Log",
    log_empty: "No events yet",
    already_running: "Genshin Auto Skipper is already running.",
    fatal: "Genshin Auto Skipper failed to start",
};

pub const RU: Texts = Texts {
    status_paused: "ПАУЗА",
    status_running: "РАБОТАЕТ",
    status_break: "ПЕРЕРЫВ",
    game_not_found: "Игра не найдена — запустите Genshin Impact",
    game_minimized: "Игра свёрнута",
    game_focused: "в фокусе",
    game_unfocused: "не в фокусе — ожидание",
    profile_wide: "профиль ultrawide",
    profile_measured: "точный профиль",
    start: "Старт",
    pause: "Пауза",
    stat_lines: "Реплик",
    stat_presses: "Нажатий",
    stat_choices: "Выборов",
    stat_time: "В работе",
    not_admin: "Программа запущена без прав администратора: игра будет игнорировать \
                ввод. Перезапустите от имени администратора.",
    help: "Как пользоваться",
    help_steps: &[
        "В игре откройте Настройки > Прочее и выключите «Автовоспроизведение сюжета».",
        "Запустите игру и нажмите хоткей старта или кнопку выше.",
        "Диалоги пропускаются, только когда окно игры в фокусе.",
        "Реплики листаются клавишей взаимодействия, из вариантов выбирается нижний.",
    ],
    settings: "Настройки",
    group_actions: "Действия",
    skip_mode: "Режим",
    ms: "мс",
    skip_mode_detect: "Распознавать диалоги",
    skip_mode_detect_hint: "Нажимать, только когда на экране диалог.",
    skip_mode_press: "Просто нажимать клавишу",
    skip_mode_press_hint: "Нажимать клавишу взаимодействия со случайным интервалом, не глядя \
                           на экран. Работает, только когда игра в фокусе.",
    mode: "Выбор вариантов",
    mode_keyboard: "Клавиша подтверждения",
    mode_mouse: "Клик мышью",
    interact_key: "Клавиша взаимодействия",
    confirm_key: "Клавиша подтверждения",
    group_hotkeys: "Горячие клавиши",
    hotkey_start: "Старт",
    hotkey_pause: "Пауза",
    hotkey_exit: "Выход",
    press_key: "Нажмите клавишу… (Esc — отмена)",
    unbound: "—",
    clear: "Сбросить",
    hotkey_conflict: "Горячая клавиша совпадает с игровой и будет заблокирована в игре.",
    hotkey_reserved: "F12 зарезервирована Windows и не может быть горячей клавишей.",
    group_timing: "Тайминги",
    humanize: "Имитация человека",
    humanize_hint: "Случайная задержка между нажатиями, иногда длиннее обычной.",
    interval: "Задержка, мс",
    breaks: "Случайные перерывы",
    breaks_hint: "Раз в 30 с — небольшой шанс сделать паузу на несколько секунд.",
    break_length: "Перерыв, с",
    group_other: "Прочее",
    tolerance: "Допуск цвета",
    tolerance_hint: "Увеличьте, если диалоги не распознаются (HDR, фильтры, цветовые профили).",
    sound: "Звук при старте/паузе",
    always_on_top: "Поверх всех окон",
    reset: "Сбросить настройки",
    open_folder: "Открыть папку настроек",
    log: "Журнал",
    log_empty: "Событий пока нет",
    already_running: "Genshin Auto Skipper уже запущен.",
    fatal: "Не удалось запустить Genshin Auto Skipper",
};

pub fn texts(lang: Language) -> &'static Texts {
    match lang {
        Language::En => &EN,
        Language::Ru => &RU,
    }
}

fn action_name(lang: Language, action: HotkeyAction) -> &'static str {
    let t = texts(lang);
    match action {
        HotkeyAction::Start => t.hotkey_start,
        HotkeyAction::Pause => t.hotkey_pause,
        HotkeyAction::Exit => t.hotkey_exit,
    }
}

pub fn log_message(lang: Language, msg: &LogMsg) -> String {
    let ru = lang == Language::Ru;
    match msg {
        LogMsg::Started => (if ru {
            "Пропуск включён"
        } else {
            "Auto-skip started"
        })
        .into(),
        LogMsg::Paused => (if ru { "Пауза" } else { "Paused" }).into(),
        LogMsg::GameFound { width, height } => {
            if ru {
                format!("Найдено окно игры {width}×{height}")
            } else {
                format!("Found the game window {width}×{height}")
            }
        }
        LogMsg::GameLost => (if ru {
            "Окно игры закрыто"
        } else {
            "The game window closed"
        })
        .into(),
        LogMsg::BreakStarted { secs } => {
            if ru {
                format!("Перерыв {secs:.1} с")
            } else {
                format!("Taking a {secs:.1} s break")
            }
        }
        LogMsg::BreakEnded => (if ru {
            "Перерыв окончен"
        } else {
            "Break is over"
        })
        .into(),
        LogMsg::HotkeyFailed { action, key } => {
            let (action, key) = (action_name(lang, *action), keys::name(*key));
            if ru {
                format!(
                    "Не удалось назначить {key} на «{action}»: клавиша занята другой программой"
                )
            } else {
                format!("Cannot bind {key} to “{action}”: the key is used by another program")
            }
        }
        LogMsg::SettingsError(e) => {
            if ru {
                format!("Ошибка настроек: {e}")
            } else {
                format!("Settings error: {e}")
            }
        }
        LogMsg::CaptureFailed(e) => {
            if ru {
                format!("Не удалось прочитать экран: {e}")
            } else {
                format!("Cannot read the screen: {e}")
            }
        }
        LogMsg::InputFailed(e) => {
            if ru {
                format!("Не удалось отправить ввод: {e}")
            } else {
                format!("Cannot send input: {e}")
            }
        }
    }
}
