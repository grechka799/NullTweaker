
use serde::{Deserialize, Serialize};

use crate::registry::{Action, Addr, Change, Hive, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loc {
    pub ru: String,
    pub en: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Privacy,
    Explorer,
    Performance,
}

impl Group {
    pub fn as_str(self) -> &'static str {
        match self {
            Group::Privacy => "privacy",
            Group::Explorer => "explorer",
            Group::Performance => "performance",
        }
    }
}

pub struct Tweak {
    pub id: &'static str,
    pub group: Group,
    pub title: Loc,
    pub description: Loc,
    pub details: Vec<String>,
    pub requires_admin: bool,
    pub win11_only: bool,
    pub changes: Vec<Change>,
}

fn dword(hive: Hive, key: &str, name: &str, v: u32) -> Change {
    Change {
        addr: Addr { hive, key: key.to_string(), name: name.to_string() },
        action: Action::Set(Value::Dword(v)),
    }
}

fn string(hive: Hive, key: &str, name: &str, v: &str) -> Change {
    Change {
        addr: Addr { hive, key: key.to_string(), name: name.to_string() },
        action: Action::Set(Value::String(v.to_string())),
    }
}

pub fn catalog() -> Vec<Tweak> {
    let hklm = Hive::Hklm;
    let hkcu = Hive::Hkcu;
    let cd = "Software\\Microsoft\\Windows\\CurrentVersion";

    vec![
        Tweak {
            id: "disable-telemetry",
            group: Group::Privacy,
            title: Loc {
                ru: "Телеметрия и сбор диагностики".into(),
                en: "Telemetry and diagnostic data".into(),
            },
            description: Loc {
                ru: "Минимальный уровень отправки диагностических данных.".into(),
                en: "Lowest level of diagnostic data upload.".into(),
            },
            details: vec![
                "HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection : AllowTelemetry = 0".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![
                dword(hklm, "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection", "AllowTelemetry", 0),
                dword(hklm, "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection", "DoNotShowFeedbackNotifications", 1),
            ],
        },
        Tweak {
            id: "disable-advertising-id",
            group: Group::Privacy,
            title: Loc {
                ru: "Идентификатор рекламы".into(),
                en: "Advertising ID".into(),
            },
            description: Loc {
                ru: "Уникальный ID, которым приложения пользуются для показа рекламы.".into(),
                en: "The unique ID that apps use to show you ads.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\AdvertisingInfo : Enabled = 0")],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(hkcu, &format!("{cd}\\AdvertisingInfo"), "Enabled", 0)],
        },
        Tweak {
            id: "disable-tailored-experiences",
            group: Group::Privacy,
            title: Loc {
                ru: "Персонализация по диагностике".into(),
                en: "Tailored experiences".into(),
            },
            description: Loc {
                ru: "Подбор советов и рекламы на основе диагностических данных.".into(),
                en: "Personalizing tips and ads using diagnostic data.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Privacy : TailoredExperiencesWithDiagnosticDataEnabled = 0"),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(
                hkcu,
                &format!("{cd}\\Privacy"),
                "TailoredExperiencesWithDiagnosticDataEnabled",
                0,
            )],
        },
        Tweak {
            id: "disable-online-speech",
            group: Group::Privacy,
            title: Loc {
                ru: "Онлайн-распознавание речи".into(),
                en: "Online speech recognition".into(),
            },
            description: Loc {
                ru: "Облачное распознавание речи для Cortana и диктовки.".into(),
                en: "Cloud speech recognition for Cortana and dictation.".into(),
            },
            details: vec![
                "HKCU\\Software\\Microsoft\\Speech_OneCore\\Settings\\OnlineSpeechPrivacy : HasAccepted = 0".into(),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(
                hkcu,
                "Software\\Microsoft\\Speech_OneCore\\Settings\\OnlineSpeechPrivacy",
                "HasAccepted",
                0,
            )],
        },
        Tweak {
            id: "disable-start-suggestions",
            group: Group::Privacy,
            title: Loc {
                ru: "Реклама в «Пуске»".into(),
                en: "Suggested content in Start".into(),
            },
            description: Loc {
                ru: "Советы, подсказки и автоматическая установка приложений из «Пуска».".into(),
                en: "Start tips, suggestions and silent app installs.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\ContentDeliveryManager : SystemPaneSuggestionsEnabled = 0"),
                format!("HKCU\\{cd}\\ContentDeliveryManager : SilentInstalledAppsEnabled = 0"),
                format!("HKCU\\{cd}\\ContentDeliveryManager : SubscribedContent-338388Enabled = 0"),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![
                dword(hkcu, &format!("{cd}\\ContentDeliveryManager"), "SystemPaneSuggestionsEnabled", 0),
                dword(hkcu, &format!("{cd}\\ContentDeliveryManager"), "SilentInstalledAppsEnabled", 0),
                dword(hkcu, &format!("{cd}\\ContentDeliveryManager"), "SubscribedContent-338388Enabled", 0),
            ],
        },
        Tweak {
            id: "disable-bing-search",
            group: Group::Privacy,
            title: Loc {
                ru: "Веб-поиск Bing в «Пуске»".into(),
                en: "Bing web search in Start".into(),
            },
            description: Loc {
                ru: "Поиск в «Пуске» только по локальным файлам, без сети.".into(),
                en: "Start search shows local results only — no network calls.".into(),
            },
            details: vec![
                "HKCU\\Software\\Policies\\Microsoft\\Windows\\Explorer : DisableSearchBoxSuggestions = 1".into(),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(
                hkcu,
                "Software\\Policies\\Microsoft\\Windows\\Explorer",
                "DisableSearchBoxSuggestions",
                1,
            )],
        },
        Tweak {
            id: "disable-ceip",
            group: Group::Privacy,
            title: Loc {
                ru: "Программа улучшения качества (CEIP)".into(),
                en: "Customer Experience program (CEIP)".into(),
            },
            description: Loc {
                ru: "Отправка данных программы улучшения качества ПО.".into(),
                en: "Customer Experience Improvement Program data upload.".into(),
            },
            details: vec![
                "HKLM\\SOFTWARE\\Policies\\Microsoft\\SQMClient : CEIPEnable = 0".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![dword(hklm, "SOFTWARE\\Policies\\Microsoft\\SQMClient", "CEIPEnable", 0)],
        },
        Tweak {
            id: "disable-app-launch-tracking",
            group: Group::Privacy,
            title: Loc {
                ru: "История активности и запусков".into(),
                en: "Activity history and app launches".into(),
            },
            description: Loc {
                ru: "Ведение журнала запусков приложений и синхронизация истории действий.".into(),
                en: "Logging app launches and syncing activity history.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Privacy\\HistoryEnabled : Start_TrackingProblems = 0"),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![
                dword(hkcu, &format!("{cd}\\Privacy\\HistoryEnabled"), "Start_TrackingProblems", 0),
            ],
        },

        Tweak {
            id: "show-file-extensions",
            group: Group::Explorer,
            title: Loc {
                ru: "Расширения файлов".into(),
                en: "Show file extensions".into(),
            },
            description: Loc {
                ru: "Показывать расширения всех известных типов файлов.".into(),
                en: "Display extensions for all known file types.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\Explorer\\Advanced : HideFileExt = 0")],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "HideFileExt", 0)],
        },
        Tweak {
            id: "show-hidden-files",
            group: Group::Explorer,
            title: Loc {
                ru: "Скрытые файлы и папки".into(),
                en: "Hidden files and folders".into(),
            },
            description: Loc {
                ru: "Отображение скрытых файлов, папок и дисков.".into(),
                en: "Show hidden files, folders and drives.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\Explorer\\Advanced : Hidden = 1")],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "Hidden", 1)],
        },
        Tweak {
            id: "classic-context-menu",
            group: Group::Explorer,
            title: Loc {
                ru: "Полное контекстное меню".into(),
                en: "Classic context menu".into(),
            },
            description: Loc {
                ru: "Полное меню правой кнопки вместо свёрнутого в Windows 11.".into(),
                en: "Full right-click menu instead of the collapsed Windows 11 one.".into(),
            },
            details: vec![
                "HKCU\\Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\\InprocServer32 : (по умолчанию) = \"\"".into(),
            ],
            requires_admin: false,
            win11_only: true,
            changes: vec![string(
                hkcu,
                "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\\InprocServer32",
                "",
                "",
            )],
        },
        Tweak {
            id: "disable-widgets",
            group: Group::Explorer,
            title: Loc {
                ru: "Убрать виджеты и новостную ленту".into(),
                en: "Remove widgets and news feed".into(),
            },
            description: Loc {
                ru: "Панель виджетов (Win11) и лента новостей на панели задач (Win10).".into(),
                en: "Widgets panel (Win11) and the taskbar news feed (Win10).".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Explorer\\Advanced : TaskbarDa = 0"),
                "HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\Dsh : AllowNewsAndInterests = 0".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![
                dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "TaskbarDa", 0),
                dword(hklm, "SOFTWARE\\Policies\\Microsoft\\Windows\\Dsh", "AllowNewsAndInterests", 0),
            ],
        },
        Tweak {
            id: "no-delete-confirm",
            group: Group::Explorer,
            title: Loc {
                ru: "Удаление без подтверждения".into(),
                en: "Delete without confirmation".into(),
            },
            description: Loc {
                ru: "Файлы удаляются сразу, без диалога «Вы действительно хотите…».".into(),
                en: "Files are deleted immediately, no confirmation dialog.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\Explorer\\Advanced : ConfirmFileDelete = 0")],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "ConfirmFileDelete", 0)],
        },
        Tweak {
            id: "taskbar-left-align",
            group: Group::Explorer,
            title: Loc {
                ru: "Панель задач по левому краю".into(),
                en: "Taskbar aligned left".into(),
            },
            description: Loc {
                ru: "Иконки панели задач по левому краю, как в Windows 10.".into(),
                en: "Taskbar icons align left, the Windows 10 way.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\Explorer\\Advanced : TaskbarAl = 0")],
            requires_admin: false,
            win11_only: true,
            changes: vec![dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "TaskbarAl", 0)],
        },
        Tweak {
            id: "hide-taskbar-chat",
            group: Group::Explorer,
            title: Loc {
                ru: "Убрать «Чат» и «Представление задач»".into(),
                en: "Hide Chat and Task View".into(),
            },
            description: Loc {
                ru: "Кнопки «Чат» и «Представление задач» с панели задач.".into(),
                en: "Chat and Task View buttons on the taskbar.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Explorer\\Advanced : TaskbarMn = 0"),
                format!("HKCU\\{cd}\\Explorer\\Advanced : ShowTaskViewButton = 0"),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![
                dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "TaskbarMn", 0),
                dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "ShowTaskViewButton", 0),
            ],
        },
        Tweak {
            id: "clock-seconds",
            group: Group::Explorer,
            title: Loc {
                ru: "Секунды в часах панели задач".into(),
                en: "Seconds in taskbar clock".into(),
            },
            description: Loc {
                ru: "Часы на панели задач показывают и секунды.".into(),
                en: "Taskbar clock shows seconds too.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\Explorer\\Advanced : ShowSecondsInSystemClock = 1")],
            requires_admin: false,
            win11_only: true,
            changes: vec![dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "ShowSecondsInSystemClock", 1)],
        },
        Tweak {
            id: "taskbar-small-icons",
            group: Group::Explorer,
            title: Loc {
                ru: "Мелкие значки на панели задач".into(),
                en: "Small taskbar icons".into(),
            },
            description: Loc {
                ru: "Компактные значки и уменьшенная высота панели задач.".into(),
                en: "Compact icons and a slimmer taskbar.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Explorer\\Advanced\\TaskbarSizeOverride : TaskbarSi = 0"),
            ],
            requires_admin: false,
            win11_only: true,
            changes: vec![dword(
                hkcu,
                &format!("{cd}\\Explorer\\Advanced\\TaskbarSizeOverride"),
                "TaskbarSi",
                0,
            )],
        },

        Tweak {
            id: "fast-menu-show",
            group: Group::Performance,
            title: Loc {
                ru: "Мгновенное раскрытие меню".into(),
                en: "Instant menu display".into(),
            },
            description: Loc {
                ru: "Без задержки перед раскрытием подменю.".into(),
                en: "No delay before submenus open.".into(),
            },
            details: vec!["HKCU\\Control Panel\\Desktop : MenuShowDelay = 0".into()],
            requires_admin: false,
            win11_only: false,
            changes: vec![string(hkcu, "Control Panel\\Desktop", "MenuShowDelay", "0")],
        },
        Tweak {
            id: "disable-animations",
            group: Group::Performance,
            title: Loc {
                ru: "Отключить анимации интерфейса".into(),
                en: "Disable UI animations".into(),
            },
            description: Loc {
                ru: "Сворачивание окон и панель задач без анимаций — быстрее отклик.".into(),
                en: "No minimize and taskbar animations — snappier UI.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Explorer\\Advanced : TaskbarAnimations = 0"),
                "HKCU\\Control Panel\\Desktop\\WindowMetrics : MinAnimate = 0".into(),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![
                dword(hkcu, &format!("{cd}\\Explorer\\Advanced"), "TaskbarAnimations", 0),
                string(hkcu, "Control Panel\\Desktop\\WindowMetrics", "MinAnimate", "0"),
            ],
        },
        Tweak {
            id: "disable-transparency",
            group: Group::Performance,
            title: Loc {
                ru: "Отключить прозрачность".into(),
                en: "Disable transparency".into(),
            },
            description: Loc {
                ru: "Сплошные фоны окон и панелей — меньше работы для GPU.".into(),
                en: "Solid window backgrounds — less GPU work.".into(),
            },
            details: vec![format!("HKCU\\{cd}\\Themes\\Personalize : EnableTransparency = 0")],
            requires_admin: false,
            win11_only: false,
            changes: vec![dword(hkcu, &format!("{cd}\\Themes\\Personalize"), "EnableTransparency", 0)],
        },
        Tweak {
            id: "enable-dark-theme",
            group: Group::Performance,
            title: Loc {
                ru: "Тёмная тема системы".into(),
                en: "System dark theme".into(),
            },
            description: Loc {
                ru: "Тёмное оформление приложений и оболочки Windows.".into(),
                en: "Dark styling for apps and the Windows shell.".into(),
            },
            details: vec![
                format!("HKCU\\{cd}\\Themes\\Personalize : AppsUseLightTheme = 0"),
                format!("HKCU\\{cd}\\Themes\\Personalize : SystemUsesLightTheme = 0"),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![
                dword(hkcu, &format!("{cd}\\Themes\\Personalize"), "AppsUseLightTheme", 0),
                dword(hkcu, &format!("{cd}\\Themes\\Personalize"), "SystemUsesLightTheme", 0),
            ],
        },
        Tweak {
            id: "disable-game-dvr",
            group: Group::Performance,
            title: Loc {
                ru: "Запись игр (Game DVR)".into(),
                en: "Game DVR".into(),
            },
            description: Loc {
                ru: "Фоновая запись игрового процесса — лишняя нагрузка на диск и GPU.".into(),
                en: "Background gameplay recording — extra disk and GPU load.".into(),
            },
            details: vec![
                "HKCU\\System\\GameConfigStore : GameDVR_Enabled = 0".into(),
                "HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\GameDVR : AllowGameDVR = 0".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![
                dword(hkcu, "System\\GameConfigStore", "GameDVR_Enabled", 0),
                dword(hklm, "SOFTWARE\\Policies\\Microsoft\\Windows\\GameDVR", "AllowGameDVR", 0),
            ],
        },
        Tweak {
            id: "disable-app-restart",
            group: Group::Performance,
            title: Loc {
                ru: "Без автоперезапуска приложений".into(),
                en: "No app relaunch after update".into(),
            },
            description: Loc {
                ru: "Windows не перезапускает сохранённые приложения после перезагрузок.".into(),
                en: "Windows won't relaunch saved apps after updates and reboots.".into(),
            },
            details: vec![
                "HKCU\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon : RestartApps = 0".into(),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![
                dword(hkcu, "Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon", "RestartApps", 0),
            ],
        },
        Tweak {
            id: "system-responsiveness",
            group: Group::Performance,
            title: Loc {
                ru: "Приоритет активных приложений".into(),
                en: "Priority for active apps".into(),
            },
            description: Loc {
                ru: "Оптимизирует распределение CPU-времени между фоном и активными окнами.".into(),
                en: "Tunes CPU time split between background services and active windows.".into(),
            },
            details: vec![
                "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile : SystemResponsiveness = 10".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![dword(
                hklm,
                "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                "SystemResponsiveness",
                10,
            )],
        },
        Tweak {
            id: "shutdown-faster",
            group: Group::Performance,
            title: Loc {
                ru: "Быстрое завершение работы".into(),
                en: "Faster shutdown".into(),
            },
            description: Loc {
                ru: "Сокращает таймаут ожидания закрытия зависших приложений.".into(),
                en: "Shortens the timeout for hung apps at shutdown.".into(),
            },
            details: vec![
                "HKCU\\Control Panel\\Desktop : WaitToKillAppTimeout = 2000".into(),
                "HKCU\\Control Panel\\Desktop : HungAppTimeout = 1000".into(),
            ],
            requires_admin: false,
            win11_only: false,
            changes: vec![
                string(hkcu, "Control Panel\\Desktop", "WaitToKillAppTimeout", "2000"),
                string(hkcu, "Control Panel\\Desktop", "HungAppTimeout", "1000"),
            ],
        },
        Tweak {
            id: "svc-mono-process",
            group: Group::Performance,
            title: Loc {
                ru: "Единый процесс для служб".into(),
                en: "Single service process".into(),
            },
            description: Loc {
                ru: "Отключает дробление svchost на отдельные процессы — экономит RAM и ускоряет старт.".into(),
                en: "Stops svchost splitting into per-service processes — saves RAM, boots faster.".into(),
            },
            details: vec![
                "HKLM\\SYSTEM\\CurrentControlSet\\Control : SvcHostSplitThresholdInKB = 0x40000000".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![dword(
                hklm,
                "SYSTEM\\CurrentControlSet\\Control",
                "SvcHostSplitThresholdInKB",
                0x4000_0000,
            )],
        },
        Tweak {
            id: "win32-priority",
            group: Group::Performance,
            title: Loc {
                ru: "Отклик переднего плана".into(),
                en: "Foreground responsiveness".into(),
            },
            description: Loc {
                ru: "Планировщик Windows отдаёт больше квантов активному окну.".into(),
                en: "Scheduler gives longer CPU quanta to the focused window.".into(),
            },
            details: vec![
                "HKLM\\SYSTEM\\CurrentControlSet\\Control\\PriorityControl : Win32PrioritySeparation = 0x26".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![dword(
                hklm,
                "SYSTEM\\CurrentControlSet\\Control\\PriorityControl",
                "Win32PrioritySeparation",
                0x26,
            )],
        },
        Tweak {
            id: "disable-copilot",
            group: Group::Privacy,
            title: Loc {
                ru: "Полное отключение Copilot".into(),
                en: "Fully disable Copilot".into(),
            },
            description: Loc {
                ru: "Политикой Windows Components: Copilot скрывается из системы и панели задач.".into(),
                en: "Group policy turns off Windows Copilot and removes it from the system.".into(),
            },
            details: vec![
                "HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsCopilot : TurnOffWindowsCopilot = 1".into(),
            ],
            requires_admin: true,
            win11_only: false,
            changes: vec![dword(
                hklm,
                "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsCopilot",
                "TurnOffWindowsCopilot",
                1,
            )],
        },
        Tweak {
            id: "disable-recall",
            group: Group::Privacy,
            title: Loc {
                ru: "Отключить Recall и AI-анализ".into(),
                en: "Disable Recall and AI analysis".into(),
            },
            description: Loc {
                ru: "Политикой запрещает фоновые снимки экрана и AI-обработку данных (DisableAIDataAnalysis).".into(),
                en: "Policy blocks background screen snapshots and AI data analysis (DisableAIDataAnalysis).".into(),
            },
            details: vec![
                "HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsAI : DisableAIDataAnalysis = 1".into(),
            ],
            requires_admin: true,
            win11_only: true,
            changes: vec![dword(
                hklm,
                "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsAI",
                "DisableAIDataAnalysis",
                1,
            )],
        },
    ]
}
