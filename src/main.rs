#![windows_subsystem = "windows"]

use std::mem::{size_of, zeroed};
use std::path::PathBuf;
use std::ptr::{copy_nonoverlapping, null, null_mut};

use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
    DIB_RGB_COLORS, HBITMAP, HDC,
};
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
};
use windows_sys::Win32::UI::HiDpi::{
    GetDpiForSystem, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, ReleaseCapture, SetCapture, VK_END,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, FindWindowW, KillTimer,
    DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, HICON, HMENU,
    ICONINFO, IDC_SIZEALL, LoadCursorW, MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG,
    PostMessageW, PostQuitMessage, RegisterClassExW, RegisterWindowMessageW, SetForegroundWindow,
    SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow, SM_CXSMICON, SM_CYSMICON,
    SM_CXSCREEN, SM_CYSCREEN, SPI_GETWORKAREA, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
    SystemParametersInfoW, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TrackPopupMenu, TranslateMessage, ULW_ALPHA, UpdateLayeredWindow, WM_CONTEXTMENU, WM_DESTROY,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONUP, WM_TIMER,
    WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP, GWL_EXSTYLE, GWLP_USERDATA, HWND_TOPMOST,
};

const KEY_W: i32 = 0x57;
const KEY_A: i32 = 0x41;
const KEY_S: i32 = 0x53;
const KEY_D: i32 = 0x44;
const KEY_SPACE: i32 = 0x20;

const WINDOW_CLASS: &str = "SimpleKeystrokeHud";
const TRAY_MESSAGE: u32 = 0x8001;
const TRAY_SHOW: u32 = 0x8002;
const TIMER_ID: usize = 1;
const TICK_MS: u32 = 8;
const TRAY_ID: u32 = 1;
const EXIT_KEY_DOWN: i16 = 0x8000u16 as i16;
const AUTOSTART_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const AUTOSTART_VALUE: &str = "SimpleKeystroke";

const ANCHOR_LEFT_BOTTOM: usize = 0;
const ANCHOR_RIGHT_BOTTOM: usize = 1;
const ANCHOR_LEFT_TOP: usize = 2;
const ANCHOR_RIGHT_TOP: usize = 3;
const ANCHOR_CENTER: usize = 4;
const ANCHOR_CUSTOM: usize = 5;

const CMD_VISIBLE: usize = 100;
const CMD_LOCK: usize = 101;
const CMD_SIZE_BASE: usize = 110;
const CMD_OPACITY_BASE: usize = 120;
const CMD_THEME_BASE: usize = 130;
const CMD_ANCHOR_BASE: usize = 140;
const CMD_AUTOSTART: usize = 150;
const CMD_RESET: usize = 160;
const CMD_EXIT: usize = 199;

const SCALES: [f32; 3] = [0.8, 1.0, 1.35];
const OPACITIES: [f32; 4] = [1.0, 0.9, 0.75, 0.6];
const OPACITY_NAMES: [&str; 4] = ["100%", "90%", "75%", "60%"];
const THEMES: [(f32, f32, f32); 5] = [
    (0.376, 0.612, 1.000),
    (0.325, 0.855, 0.780),
    (0.706, 0.545, 1.000),
    (0.930, 0.960, 1.000),
    (1.000, 0.667, 0.298),
];

const IDLE_FILL: (f32, f32, f32) = (0.055, 0.071, 0.102);
const IDLE_FILL_ALPHA: f32 = 0.90;
const ACTIVE_FILL_ALPHA: f32 = 0.97;
const IDLE_LABEL: (f32, f32, f32) = (0.698, 0.761, 0.847);
const IDLE_LABEL_ALPHA: f32 = 0.90;
const IDLE_BORDER: (f32, f32, f32) = (1.0, 1.0, 1.0);
const IDLE_BORDER_ALPHA: f32 = 0.14;
const ACTIVE_BORDER_ALPHA: f32 = 0.55;
const GLOW_ALPHA: f32 = 0.55;

const GLYPH_W: &[&[(f32, f32)]] = &[&[
    (0.06, 0.16),
    (0.28, 0.86),
    (0.50, 0.40),
    (0.72, 0.86),
    (0.94, 0.16),
]];

const GLYPH_A: &[&[(f32, f32)]] = &[
    &[(0.08, 0.86), (0.50, 0.14), (0.92, 0.86)],
    &[(0.24, 0.58), (0.76, 0.58)],
];

const GLYPH_S: &[&[(f32, f32)]] = &[&[
    (0.90, 0.28),
    (0.74, 0.14),
    (0.30, 0.14),
    (0.12, 0.30),
    (0.20, 0.47),
    (0.50, 0.52),
    (0.80, 0.58),
    (0.90, 0.72),
    (0.74, 0.86),
    (0.28, 0.86),
    (0.10, 0.73),
]];

const GLYPH_D: &[&[(f32, f32)]] = &[
    &[(0.16, 0.14), (0.16, 0.86)],
    &[
        (0.16, 0.14),
        (0.52, 0.14),
        (0.74, 0.20),
        (0.88, 0.38),
        (0.88, 0.62),
        (0.74, 0.80),
        (0.52, 0.86),
        (0.16, 0.86),
    ],
];

struct Texts {
    show_hud: &'static str,
    hide_hud: &'static str,
    lock_position: &'static str,
    unlock_drag: &'static str,
    size: &'static str,
    scale_names: [&'static str; 3],
    opacity: &'static str,
    theme: &'static str,
    theme_names: [&'static str; 5],
    position: &'static str,
    anchor_names: [&'static str; 5],
    custom_drag: &'static str,
    autostart: &'static str,
    reset: &'static str,
    quit: &'static str,
    tooltip: &'static str,
}

const TEXTS_EN: Texts = Texts {
    show_hud: "Show HUD",
    hide_hud: "Hide HUD",
    lock_position: "Lock position",
    unlock_drag: "Unlock and drag",
    size: "Size",
    scale_names: ["Small", "Medium", "Large"],
    opacity: "Opacity",
    theme: "Accent color",
    theme_names: ["Ice blue", "Teal", "Lilac", "White", "Amber"],
    position: "Position",
    anchor_names: [
        "Bottom left",
        "Bottom right",
        "Top left",
        "Top right",
        "Screen center",
    ],
    custom_drag: "Custom (drag to place)",
    autostart: "Start with Windows",
    reset: "Reset to defaults",
    quit: "Quit",
    tooltip: "SimpleKeystroke - right-click for settings - double-click to toggle",
};

const TEXTS_ZH_HANS: Texts = Texts {
    show_hud: "显示 HUD",
    hide_hud: "隐藏 HUD",
    lock_position: "锁定位置",
    unlock_drag: "解锁并拖动",
    size: "尺寸",
    scale_names: ["小", "中", "大"],
    opacity: "不透明度",
    theme: "主题色",
    theme_names: ["冰蓝", "青绿", "淡紫", "纯白", "琥珀"],
    position: "位置",
    anchor_names: ["左下角", "右下角", "左上角", "右上角", "屏幕居中"],
    custom_drag: "自定义（拖动定位）",
    autostart: "开机自动启动",
    reset: "恢复默认设置",
    quit: "退出",
    tooltip: "按键 HUD · 右键设置 · 双击显示或隐藏",
};

const TEXTS_ZH_HANT: Texts = Texts {
    show_hud: "顯示 HUD",
    hide_hud: "隱藏 HUD",
    lock_position: "鎖定位置",
    unlock_drag: "解鎖並拖曳",
    size: "尺寸",
    scale_names: ["小", "中", "大"],
    opacity: "不透明度",
    theme: "主題色",
    theme_names: ["冰藍", "青綠", "淡紫", "純白", "琥珀"],
    position: "位置",
    anchor_names: ["左下角", "右下角", "左上角", "右上角", "畫面置中"],
    custom_drag: "自訂（拖曳定位）",
    autostart: "開機自動啟動",
    reset: "還原預設值",
    quit: "結束",
    tooltip: "按鍵 HUD · 右鍵設定 · 雙擊顯示或隱藏",
};

const TEXTS_JA: Texts = Texts {
    show_hud: "HUD を表示",
    hide_hud: "HUD を隠す",
    lock_position: "位置を固定",
    unlock_drag: "ロックを解除して移動",
    size: "サイズ",
    scale_names: ["小", "中", "大"],
    opacity: "不透明度",
    theme: "アクセントカラー",
    theme_names: ["アイスブルー", "ティール", "ライラック", "ホワイト", "アンバー"],
    position: "位置",
    anchor_names: ["左下", "右下", "左上", "右上", "画面中央"],
    custom_drag: "カスタム（ドラッグで配置）",
    autostart: "Windows と同時に起動",
    reset: "既定値に戻す",
    quit: "終了",
    tooltip: "キーストローク HUD ・右クリックで設定・ダブルクリックで表示切替",
};

const TEXTS_KO: Texts = Texts {
    show_hud: "HUD 표시",
    hide_hud: "HUD 숨기기",
    lock_position: "위치 고정",
    unlock_drag: "잠금 해제 후 이동",
    size: "크기",
    scale_names: ["작게", "보통", "크게"],
    opacity: "불투명도",
    theme: "강조 색상",
    theme_names: ["아이스 블루", "틸", "라일락", "화이트", "앰버"],
    position: "위치",
    anchor_names: ["왼쪽 아래", "오른쪽 아래", "왼쪽 위", "오른쪽 위", "화면 중앙"],
    custom_drag: "사용자 지정(드래그로 배치)",
    autostart: "Windows 시작 시 실행",
    reset: "기본값으로 재설정",
    quit: "종료",
    tooltip: "키스트로크 HUD · 우클릭 설정 · 더블클릭 표시 전환",
};

const TEXTS_RU: Texts = Texts {
    show_hud: "Показать HUD",
    hide_hud: "Скрыть HUD",
    lock_position: "Зафиксировать положение",
    unlock_drag: "Разблокировать и перетащить",
    size: "Размер",
    scale_names: ["Малый", "Средний", "Большой"],
    opacity: "Непрозрачность",
    theme: "Акцентный цвет",
    theme_names: ["Ледяной синий", "Бирюзовый", "Лавандовый", "Белый", "Янтарный"],
    position: "Положение",
    anchor_names: ["Слева внизу", "Справа внизу", "Слева вверху", "Справа вверху", "По центру"],
    custom_drag: "Своё (перетащите)",
    autostart: "Запускать с Windows",
    reset: "Сбросить настройки",
    quit: "Выход",
    tooltip: "HUD нажатий · ПКМ — настройки · Двойной щелчок — скрыть или показать",
};

const TEXTS_ES: Texts = Texts {
    show_hud: "Mostrar HUD",
    hide_hud: "Ocultar HUD",
    lock_position: "Bloquear posición",
    unlock_drag: "Desbloquear y arrastrar",
    size: "Tamaño",
    scale_names: ["Pequeño", "Medio", "Grande"],
    opacity: "Opacidad",
    theme: "Color de acento",
    theme_names: ["Azul hielo", "Turquesa", "Lila", "Blanco", "Ámbar"],
    position: "Posición",
    anchor_names: [
        "Abajo izquierda",
        "Abajo derecha",
        "Arriba izquierda",
        "Arriba derecha",
        "Centro",
    ],
    custom_drag: "Personalizado (arrastra)",
    autostart: "Iniciar con Windows",
    reset: "Restablecer valores",
    quit: "Salir",
    tooltip: "HUD de teclas · Clic derecho: ajustes · Doble clic: mostrar u ocultar",
};

const TEXTS_FR: Texts = Texts {
    show_hud: "Afficher le HUD",
    hide_hud: "Masquer le HUD",
    lock_position: "Verrouiller la position",
    unlock_drag: "Déverrouiller et déplacer",
    size: "Taille",
    scale_names: ["Petit", "Moyen", "Grand"],
    opacity: "Opacité",
    theme: "Couleur d'accent",
    theme_names: ["Bleu glacé", "Turquoise", "Lilas", "Blanc", "Ambre"],
    position: "Position",
    anchor_names: [
        "En bas à gauche",
        "En bas à droite",
        "En haut à gauche",
        "En haut à droite",
        "Centré",
    ],
    custom_drag: "Personnalisée (glisser)",
    autostart: "Lancer avec Windows",
    reset: "Réinitialiser",
    quit: "Quitter",
    tooltip: "HUD de frappes · Clic droit : réglages · Double-clic : afficher ou masquer",
};

const TEXTS_DE: Texts = Texts {
    show_hud: "HUD anzeigen",
    hide_hud: "HUD ausblenden",
    lock_position: "Position sperren",
    unlock_drag: "Entsperren und verschieben",
    size: "Größe",
    scale_names: ["Klein", "Mittel", "Groß"],
    opacity: "Deckkraft",
    theme: "Akzentfarbe",
    theme_names: ["Eisblau", "Türkis", "Flieder", "Weiß", "Bernstein"],
    position: "Position",
    anchor_names: ["Unten links", "Unten rechts", "Oben links", "Oben rechts", "Bildmitte"],
    custom_drag: "Benutzerdefiniert (ziehen)",
    autostart: "Mit Windows starten",
    reset: "Zurücksetzen",
    quit: "Beenden",
    tooltip: "Tasten-HUD · Rechtsklick: Einstellungen · Doppelklick: ein- oder ausblenden",
};

const TEXTS_PT: Texts = Texts {
    show_hud: "Mostrar HUD",
    hide_hud: "Ocultar HUD",
    lock_position: "Bloquear posição",
    unlock_drag: "Desbloquear e arrastar",
    size: "Tamanho",
    scale_names: ["Pequeno", "Médio", "Grande"],
    opacity: "Opacidade",
    theme: "Cor de destaque",
    theme_names: ["Azul gelo", "Turquesa", "Lilás", "Branco", "Âmbar"],
    position: "Posição",
    anchor_names: [
        "Inferior esquerda",
        "Inferior direita",
        "Superior esquerda",
        "Superior direita",
        "Centro",
    ],
    custom_drag: "Personalizado (arrastar)",
    autostart: "Iniciar com o Windows",
    reset: "Restaurar padrões",
    quit: "Sair",
    tooltip: "HUD de teclas · Clique direito: ajustes · Clique duplo: mostrar ou ocultar",
};

fn system_texts() -> &'static Texts {
    let language = unsafe { GetUserDefaultUILanguage() };
    let primary = language & 0x3FF;
    let sublanguage = (language >> 10) & 0x3F;
    match primary {
        0x04 if sublanguage == 1 || sublanguage == 3 || sublanguage == 5 => &TEXTS_ZH_HANT,
        0x04 => &TEXTS_ZH_HANS,
        0x07 => &TEXTS_DE,
        0x0A => &TEXTS_ES,
        0x0C => &TEXTS_FR,
        0x11 => &TEXTS_JA,
        0x12 => &TEXTS_KO,
        0x16 => &TEXTS_PT,
        0x19 => &TEXTS_RU,
        _ => &TEXTS_EN,
    }
}

#[derive(Clone, Copy)]
struct Region {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Region {
    fn at(left: f32, top: f32, width: f32, height: f32) -> Region {
        Region {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.bottom - self.top
    }

    fn center_x(&self) -> f32 {
        (self.left + self.right) * 0.5
    }

    fn center_y(&self) -> f32 {
        (self.top + self.bottom) * 0.5
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn mix(from: (f32, f32, f32), to: (f32, f32, f32), amount: f32) -> (f32, f32, f32) {
    let amount = amount.clamp(0.0, 1.0);
    (
        from.0 + (to.0 - from.0) * amount,
        from.1 + (to.1 - from.1) * amount,
        from.2 + (to.2 - from.2) * amount,
    )
}

fn mix_scalar(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

fn rounded_distance(x: f32, y: f32, area: Region, radius: f32) -> f32 {
    let half_width = area.width() * 0.5;
    let half_height = area.height() * 0.5;
    let radius = radius.min(half_width).min(half_height);
    let offset_x = (x - area.center_x()).abs() - half_width + radius;
    let offset_y = (y - area.center_y()).abs() - half_height + radius;
    let outside_x = offset_x.max(0.0);
    let outside_y = offset_y.max(0.0);
    let outside = (outside_x * outside_x + outside_y * outside_y).sqrt();
    let inside = offset_x.max(offset_y).min(0.0);
    outside + inside - radius
}

fn segment_distance(x: f32, y: f32, start: (f32, f32), end: (f32, f32)) -> f32 {
    let direction_x = end.0 - start.0;
    let direction_y = end.1 - start.1;
    let length_squared = direction_x * direction_x + direction_y * direction_y;
    let mut amount = 0.0;
    if length_squared > 0.0 {
        amount = (((x - start.0) * direction_x + (y - start.1) * direction_y) / length_squared)
            .clamp(0.0, 1.0);
    }
    let offset_x = start.0 + direction_x * amount - x;
    let offset_y = start.1 + direction_y * amount - y;
    (offset_x * offset_x + offset_y * offset_y).sqrt()
}

struct Painter {
    width: i32,
    height: i32,
    pixels: Vec<u32>,
}

impl Painter {
    fn new(width: i32, height: i32) -> Painter {
        Painter {
            width,
            height,
            pixels: vec![0u32; (width.max(0) * height.max(0)) as usize],
        }
    }

    fn clear(&mut self) {
        self.pixels.fill(0);
    }

    fn blend(&mut self, x: i32, y: i32, color: (f32, f32, f32), coverage: f32) {
        if coverage <= 0.0 || x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let coverage = coverage.min(1.0);
        let index = (y * self.width + x) as usize;
        let destination = self.pixels[index];
        let keep = 1.0 - coverage;
        let destination_alpha = ((destination >> 24) & 0xFF) as f32 / 255.0;
        let destination_red = ((destination >> 16) & 0xFF) as f32 / 255.0;
        let destination_green = ((destination >> 8) & 0xFF) as f32 / 255.0;
        let destination_blue = (destination & 0xFF) as f32 / 255.0;
        let alpha = coverage + destination_alpha * keep;
        let red = color.0 * coverage + destination_red * keep;
        let green = color.1 * coverage + destination_green * keep;
        let blue = color.2 * coverage + destination_blue * keep;
        let alpha_byte = (alpha * 255.0 + 0.5).min(255.0) as u32;
        let red_byte = (red * 255.0 + 0.5).min(255.0) as u32;
        let green_byte = (green * 255.0 + 0.5).min(255.0) as u32;
        let blue_byte = (blue * 255.0 + 0.5).min(255.0) as u32;
        self.pixels[index] = (alpha_byte << 24) | (red_byte << 16) | (green_byte << 8) | blue_byte;
    }

    fn fill_rounded(&mut self, area: Region, radius: f32, color: (f32, f32, f32), alpha: f32) {
        let left = area.left.floor() as i32 - 1;
        let right = area.right.ceil() as i32 + 1;
        let top = area.top.floor() as i32 - 1;
        let bottom = area.bottom.ceil() as i32 + 1;
        for y in top..=bottom {
            for x in left..=right {
                let distance = rounded_distance(x as f32 + 0.5, y as f32 + 0.5, area, radius);
                let coverage = (0.5 - distance).clamp(0.0, 1.0) * alpha;
                if coverage > 0.0 {
                    self.blend(x, y, color, coverage);
                }
            }
        }
    }

    fn stroke_rounded(
        &mut self,
        area: Region,
        radius: f32,
        thickness: f32,
        color: (f32, f32, f32),
        alpha: f32,
    ) {
        let left = area.left.floor() as i32 - 2;
        let right = area.right.ceil() as i32 + 2;
        let top = area.top.floor() as i32 - 2;
        let bottom = area.bottom.ceil() as i32 + 2;
        let half = thickness * 0.5;
        for y in top..=bottom {
            for x in left..=right {
                let distance = rounded_distance(x as f32 + 0.5, y as f32 + 0.5, area, radius);
                let coverage = (half + 0.5 - distance.abs()).clamp(0.0, 1.0) * alpha;
                if coverage > 0.0 {
                    self.blend(x, y, color, coverage);
                }
            }
        }
    }

    fn glow_rounded(
        &mut self,
        area: Region,
        radius: f32,
        spread: f32,
        color: (f32, f32, f32),
        alpha: f32,
    ) {
        let left = area.left.floor() as i32 - spread as i32 - 1;
        let right = area.right.ceil() as i32 + spread as i32 + 1;
        let top = area.top.floor() as i32 - spread as i32 - 1;
        let bottom = area.bottom.ceil() as i32 + spread as i32 + 1;
        for y in top..=bottom {
            for x in left..=right {
                let distance = rounded_distance(x as f32 + 0.5, y as f32 + 0.5, area, radius);
                let falloff = (1.0 - distance / spread).clamp(0.0, 1.0);
                let coverage = falloff * falloff * alpha;
                if coverage > 0.0 {
                    self.blend(x, y, color, coverage);
                }
            }
        }
    }

    fn stroke_polyline(
        &mut self,
        points: &[(f32, f32)],
        thickness: f32,
        color: (f32, f32, f32),
        alpha: f32,
    ) {
        if points.len() < 2 {
            return;
        }
        let half = thickness * 0.5;
        let mut left = f32::MAX;
        let mut right = f32::MIN;
        let mut top = f32::MAX;
        let mut bottom = f32::MIN;
        for point in points {
            left = left.min(point.0);
            right = right.max(point.0);
            top = top.min(point.1);
            bottom = bottom.max(point.1);
        }
        let left = (left - half - 1.0).floor() as i32;
        let right = (right + half + 1.0).ceil() as i32;
        let top = (top - half - 1.0).floor() as i32;
        let bottom = (bottom + half + 1.0).ceil() as i32;
        for y in top..=bottom {
            for x in left..=right {
                let sample_x = x as f32 + 0.5;
                let sample_y = y as f32 + 0.5;
                let mut nearest = f32::MAX;
                for index in 0..points.len() - 1 {
                    let distance =
                        segment_distance(sample_x, sample_y, points[index], points[index + 1]);
                    nearest = nearest.min(distance);
                }
                let coverage = (half + 0.5 - nearest).clamp(0.0, 1.0) * alpha;
                if coverage > 0.0 {
                    self.blend(x, y, color, coverage);
                }
            }
        }
    }
}

fn draw_glyph(
    painter: &mut Painter,
    glyph: &[&[(f32, f32)]],
    area: Region,
    thickness: f32,
    color: (f32, f32, f32),
    alpha: f32,
) {
    for path in glyph {
        let mut points = Vec::with_capacity(path.len());
        for point in path.iter() {
            points.push((
                area.left + point.0 * area.width(),
                area.top + point.1 * area.height(),
            ));
        }
        painter.stroke_polyline(&points, thickness, color, alpha);
    }
}

#[derive(Clone, Copy)]
struct Settings {
    scale_index: usize,
    opacity_index: usize,
    theme_index: usize,
    anchor: usize,
    custom_x: i32,
    custom_y: i32,
    locked: bool,
    visible: bool,
    autostart: bool,
}

impl Settings {
    fn fresh() -> Settings {
        Settings {
            scale_index: 1,
            opacity_index: 0,
            theme_index: 0,
            anchor: ANCHOR_LEFT_BOTTOM,
            custom_x: 0,
            custom_y: 0,
            locked: true,
            visible: true,
            autostart: false,
        }
    }
}

fn portable_path() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    Some(executable.parent()?.join("simplekeystroke.ini"))
}

fn fallback_path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")?;
    let directory = PathBuf::from(base).join("KeystrokeHud");
    std::fs::create_dir_all(&directory).ok()?;
    Some(directory.join("settings.ini"))
}

fn load_settings() -> Settings {
    let mut settings = Settings::fresh();
    let source = portable_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .or_else(|| fallback_path().and_then(|path| std::fs::read_to_string(path).ok()));
    if let Some(text) = source {
        for line in text.lines() {
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or_default().trim();
            let value = parts.next().unwrap_or_default().trim();
            match key {
                "scale" => settings.scale_index = value.parse::<usize>().unwrap_or(1).min(2),
                "opacity" => settings.opacity_index = value.parse::<usize>().unwrap_or(0).min(3),
                "theme" => settings.theme_index = value.parse::<usize>().unwrap_or(0).min(4),
                "anchor" => settings.anchor = value.parse::<usize>().unwrap_or(0).min(5),
                "x" => settings.custom_x = value.parse::<i32>().unwrap_or(0),
                "y" => settings.custom_y = value.parse::<i32>().unwrap_or(0),
                "locked" => settings.locked = value == "1",
                "visible" => settings.visible = value == "1",
                _ => {}
            }
        }
    }
    settings.autostart = unsafe { autostart_enabled() };
    settings
}

fn save_settings(settings: &Settings) {
    let text = format!(
        "scale={}\nopacity={}\ntheme={}\nanchor={}\nx={}\ny={}\nlocked={}\nvisible={}\n",
        settings.scale_index,
        settings.opacity_index,
        settings.theme_index,
        settings.anchor,
        settings.custom_x,
        settings.custom_y,
        if settings.locked { 1 } else { 0 },
        if settings.visible { 1 } else { 0 }
    );
    if let Some(path) = portable_path() {
        if std::fs::write(&path, &text).is_ok() {
            return;
        }
    }
    if let Some(path) = fallback_path() {
        let _ = std::fs::write(path, text);
    }
}

unsafe fn autostart_enabled() -> bool {
    let mut key: HKEY = null_mut();
    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        wide(AUTOSTART_KEY).as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut key,
    ) != 0
    {
        return false;
    }
    let mut buffer = [0u16; 1024];
    let mut size = (buffer.len() * 2) as u32;
    let mut kind = 0u32;
    let status = RegQueryValueExW(
        key,
        wide(AUTOSTART_VALUE).as_ptr(),
        null(),
        &mut kind,
        buffer.as_mut_ptr() as *mut u8,
        &mut size,
    );
    RegCloseKey(key);
    status == 0 && size > 2
}

unsafe fn set_autostart(enabled: bool) {
    let mut key: HKEY = null_mut();
    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        wide(AUTOSTART_KEY).as_ptr(),
        0,
        KEY_SET_VALUE,
        &mut key,
    ) != 0
    {
        return;
    }
    if enabled {
        let mut path = [0u16; 1024];
        let length = GetModuleFileNameW(null_mut(), path.as_mut_ptr(), path.len() as u32) as usize;
        let mut command: Vec<u16> = Vec::with_capacity(length + 3);
        command.push('"' as u16);
        command.extend_from_slice(&path[..length]);
        command.push('"' as u16);
        command.push(0);
        RegSetValueExW(
            key,
            wide(AUTOSTART_VALUE).as_ptr(),
            0,
            REG_SZ,
            command.as_ptr() as *const u8,
            (command.len() * 2) as u32,
        );
    } else {
        RegDeleteValueW(key, wide(AUTOSTART_VALUE).as_ptr());
    }
    RegCloseKey(key);
}

struct Key {
    virtual_key: i32,
    glyph: &'static [&'static [(f32, f32)]],
    area: Region,
    level: f32,
}

fn build_keys(unit: f32) -> Vec<Key> {
    let key = 46.0 * unit;
    let gap = 8.0 * unit;
    let padding = 12.0 * unit;
    let space_height = 24.0 * unit;
    let row_width = key * 3.0 + gap * 2.0;
    let top = padding;
    let middle = padding + key + gap;
    let bottom = middle + key + gap;
    let center = padding + (row_width - key) * 0.5;
    vec![
        Key {
            virtual_key: KEY_W,
            glyph: GLYPH_W,
            area: Region::at(center, top, key, key),
            level: 0.0,
        },
        Key {
            virtual_key: KEY_A,
            glyph: GLYPH_A,
            area: Region::at(padding, middle, key, key),
            level: 0.0,
        },
        Key {
            virtual_key: KEY_S,
            glyph: GLYPH_S,
            area: Region::at(padding + key + gap, middle, key, key),
            level: 0.0,
        },
        Key {
            virtual_key: KEY_D,
            glyph: GLYPH_D,
            area: Region::at(padding + (key + gap) * 2.0, middle, key, key),
            level: 0.0,
        },
        Key {
            virtual_key: KEY_SPACE,
            glyph: &[],
            area: Region::at(padding, bottom, row_width, space_height),
            level: 0.0,
        },
    ]
}

fn canvas_size(keys: &[Key], unit: f32) -> (i32, i32) {
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    for key in keys {
        width = width.max(key.area.right);
        height = height.max(key.area.bottom);
    }
    (
        (width + 12.0 * unit).round() as i32,
        (height + 12.0 * unit).round() as i32,
    )
}

unsafe fn create_surface(
    device: HDC,
    width: i32,
    height: i32,
) -> (HBITMAP, *mut core::ffi::c_void) {
    let mut info: BITMAPINFO = zeroed();
    info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = width;
    info.bmiHeader.biHeight = -height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    let mut bits: *mut core::ffi::c_void = null_mut();
    let bitmap = CreateDIBSection(device, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if !bitmap.is_null() {
        SelectObject(device, bitmap as _);
    }
    (bitmap, bits)
}

unsafe fn build_icon(size: i32, accent: (f32, f32, f32)) -> HICON {
    let mut painter = Painter::new(size, size);
    let inset = size as f32 * 0.05;
    let area = Region::at(
        inset,
        inset,
        size as f32 - inset * 2.0,
        size as f32 - inset * 2.0,
    );
    painter.fill_rounded(area, size as f32 * 0.24, accent, 1.0);
    let glyph_area = Region::at(
        size as f32 * 0.28,
        size as f32 * 0.28,
        size as f32 * 0.44,
        size as f32 * 0.44,
    );
    draw_glyph(
        &mut painter,
        GLYPH_W,
        glyph_area,
        (size as f32 * 0.085).max(1.0),
        (1.0, 1.0, 1.0),
        1.0,
    );
    let mut straight = vec![0u8; (size * size * 4) as usize];
    for index in 0..(size * size) as usize {
        let pixel = painter.pixels[index];
        let alpha = (pixel >> 24) & 0xFF;
        let factor = if alpha == 0 {
            0.0
        } else {
            255.0 / alpha as f32
        };
        let red = (((pixel >> 16) & 0xFF) as f32 * factor).min(255.0) as u8;
        let green = (((pixel >> 8) & 0xFF) as f32 * factor).min(255.0) as u8;
        let blue = ((pixel & 0xFF) as f32 * factor).min(255.0) as u8;
        straight[index * 4] = blue;
        straight[index * 4 + 1] = green;
        straight[index * 4 + 2] = red;
        straight[index * 4 + 3] = alpha as u8;
    }
    let screen = GetDC(null_mut());
    let device = CreateCompatibleDC(screen);
    let mut info: BITMAPINFO = zeroed();
    info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = size;
    info.bmiHeader.biHeight = -size;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    let mut bits: *mut core::ffi::c_void = null_mut();
    let color = CreateDIBSection(device, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if !color.is_null() && !bits.is_null() {
        copy_nonoverlapping(straight.as_ptr(), bits as *mut u8, straight.len());
    }
    let mask_stride = (((size + 15) / 16) * 2) as usize;
    let mask_bits = vec![0u8; mask_stride * size as usize];
    let mask = CreateBitmap(size, size, 1, 1, mask_bits.as_ptr() as *const _);
    let description = ICONINFO {
        fIcon: 1,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: mask,
        hbmColor: color,
    };
    let icon = CreateIconIndirect(&description);
    if !mask.is_null() {
        DeleteObject(mask as _);
    }
    if !color.is_null() {
        DeleteObject(color as _);
    }
    DeleteDC(device);
    ReleaseDC(null_mut(), screen);
    icon
}

struct Application {
    window: HWND,
    painter: Painter,
    keys: Vec<Key>,
    settings: Settings,
    texts: &'static Texts,
    dpi_scale: f32,
    unit: f32,
    device: HDC,
    bitmap: HBITMAP,
    bits: *mut core::ffi::c_void,
    icon: HICON,
    width: i32,
    height: i32,
    offset: POINT,
    dirty: bool,
    hover: bool,
    dragging: bool,
    drag_pointer: POINT,
    drag_window: POINT,
    taskbar_created: u32,
    tray_registered: bool,
}

impl Application {
    fn render(&mut self) {
        let unit = self.unit;
        let opacity = OPACITIES[self.settings.opacity_index];
        let accent = THEMES[self.settings.theme_index];
        self.painter.clear();
        for key in &self.keys {
            let level = key.level;
            let radius = key.area.height() * 0.22;
            if level > 0.01 {
                self.painter.glow_rounded(
                    key.area,
                    radius,
                    16.0 * unit,
                    accent,
                    GLOW_ALPHA * level * opacity,
                );
            }
            let fill = mix(IDLE_FILL, accent, level);
            let fill_alpha = mix_scalar(IDLE_FILL_ALPHA, ACTIVE_FILL_ALPHA, level) * opacity;
            self.painter.fill_rounded(key.area, radius, fill, fill_alpha);
            let border = mix(IDLE_BORDER, mix(accent, (1.0, 1.0, 1.0), 0.35), level);
            let border_alpha = mix_scalar(IDLE_BORDER_ALPHA, ACTIVE_BORDER_ALPHA, level) * opacity;
            self.painter
                .stroke_rounded(key.area, radius, unit.max(1.0), border, border_alpha);
            let label = mix(IDLE_LABEL, (1.0, 1.0, 1.0), level);
            let label_alpha = mix_scalar(IDLE_LABEL_ALPHA, 1.0, level) * opacity;
            if key.glyph.is_empty() {
                let bar_width = key.area.width() * 0.30;
                let bar_height = (3.0 * unit).max(2.0);
                let bar = Region::at(
                    key.area.center_x() - bar_width * 0.5,
                    key.area.center_y() - bar_height * 0.5,
                    bar_width,
                    bar_height,
                );
                self.painter
                    .fill_rounded(bar, bar_height * 0.5, label, label_alpha);
            } else {
                let side = key.area.width().min(key.area.height()) * 0.44;
                let glyph_area = Region::at(
                    key.area.center_x() - side * 0.5,
                    key.area.center_y() - side * 0.5,
                    side,
                    side,
                );
                let thickness = (side * 0.17).max(2.0 * unit);
                draw_glyph(
                    &mut self.painter,
                    key.glyph,
                    glyph_area,
                    thickness,
                    label,
                    label_alpha,
                );
            }
        }
        if self.hover && !self.settings.locked {
            let frame = Region::at(
                2.0 * unit,
                2.0 * unit,
                self.width as f32 - 4.0 * unit,
                self.height as f32 - 4.0 * unit,
            );
            self.painter
                .stroke_rounded(frame, 10.0 * unit, unit.max(1.0), accent, 0.32 * opacity);
        }
        self.dirty = false;
    }

    fn present(&self) {
        unsafe {
            if self.bits.is_null() {
                return;
            }
            copy_nonoverlapping(
                self.painter.pixels.as_ptr(),
                self.bits as *mut u32,
                self.painter.pixels.len(),
            );
            let size = SIZE {
                cx: self.width,
                cy: self.height,
            };
            let source = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            UpdateLayeredWindow(
                self.window,
                null_mut(),
                &self.offset,
                &size,
                self.device,
                &source,
                0,
                &blend,
                ULW_ALPHA,
            );
        }
    }

    fn flush(&mut self) {
        if !self.settings.visible || !self.dirty {
            return;
        }
        self.render();
        self.present();
    }

    fn tick(&mut self) {
        if !self.settings.visible {
            return;
        }
        let mut changed = false;
        for key in &mut self.keys {
            let pressed = unsafe { GetAsyncKeyState(key.virtual_key) & EXIT_KEY_DOWN != 0 };
            let target = if pressed { 1.0 } else { 0.0 };
            let rate = if pressed { 0.55 } else { 0.18 };
            if (target - key.level).abs() > 0.002 {
                key.level += (target - key.level) * rate;
                changed = true;
            } else if key.level != target {
                key.level = target;
                changed = true;
            }
        }
        let hovering = if self.settings.locked {
            false
        } else {
            unsafe {
                let mut cursor = POINT { x: 0, y: 0 };
                GetCursorPos(&mut cursor);
                cursor.x >= self.offset.x
                    && cursor.x < self.offset.x + self.width
                    && cursor.y >= self.offset.y
                    && cursor.y < self.offset.y + self.height
            }
        };
        if hovering != self.hover {
            self.hover = hovering;
            changed = true;
        }
        if changed {
            self.dirty = true;
        }
    }

    fn rebuild(&mut self) {
        unsafe {
            self.unit = self.dpi_scale * SCALES[self.settings.scale_index];
            self.keys = build_keys(self.unit);
            let (width, height) = canvas_size(&self.keys, self.unit);
            if width != self.width || height != self.height {
                let (bitmap, bits) = create_surface(self.device, width, height);
                if !self.bitmap.is_null() {
                    DeleteObject(self.bitmap as _);
                }
                self.width = width;
                self.height = height;
                self.bitmap = bitmap;
                self.bits = bits;
                self.painter = Painter::new(width, height);
            }
        }
        self.reposition();
        self.dirty = true;
    }

    fn work_area(&self) -> RECT {
        let mut area: RECT = unsafe { zeroed() };
        unsafe {
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut area as *mut RECT as *mut core::ffi::c_void,
                0,
            );
        }
        if area.right <= area.left || area.bottom <= area.top {
            area = RECT {
                left: 0,
                top: 0,
                right: unsafe { GetSystemMetrics(SM_CXSCREEN) },
                bottom: unsafe { GetSystemMetrics(SM_CYSCREEN) },
            };
        }
        area
    }

    fn reposition(&mut self) {
        let area = self.work_area();
        let margin = (24.0 * self.unit) as i32;
        let (mut x, mut y) = match self.settings.anchor {
            ANCHOR_RIGHT_BOTTOM => (
                area.right - self.width - margin,
                area.bottom - self.height - margin,
            ),
            ANCHOR_LEFT_TOP => (area.left + margin, area.top + margin),
            ANCHOR_RIGHT_TOP => (area.right - self.width - margin, area.top + margin),
            ANCHOR_CENTER => (
                area.left + (area.right - area.left - self.width) / 2,
                area.top + (area.bottom - area.top - self.height) / 2,
            ),
            ANCHOR_CUSTOM => (self.settings.custom_x, self.settings.custom_y),
            _ => (area.left + margin, area.bottom - self.height - margin),
        };
        x = x.clamp(area.left, (area.right - self.width).max(area.left));
        y = y.clamp(area.top, (area.bottom - self.height).max(area.top));
        self.settings.custom_x = x;
        self.settings.custom_y = y;
        self.offset = POINT { x, y };
        unsafe {
            let mut flags = SWP_NOACTIVATE;
            if self.settings.visible {
                flags |= SWP_SHOWWINDOW;
            }
            SetWindowPos(self.window, HWND_TOPMOST, x, y, self.width, self.height, flags);
        }
    }

    fn apply_click_through(&self) {
        unsafe {
            let current = GetWindowLongPtrW(self.window, GWL_EXSTYLE) as u32;
            let updated = if self.settings.locked {
                current | WS_EX_TRANSPARENT
            } else {
                current & !WS_EX_TRANSPARENT
            };
            if updated != current {
                SetWindowLongPtrW(self.window, GWL_EXSTYLE, updated as isize);
            }
        }
    }

    fn set_visible(&mut self, visible: bool) {
        self.settings.visible = visible;
        if visible {
            self.hover = false;
            self.dirty = true;
            self.flush();
            unsafe {
                ShowWindow(self.window, SW_SHOW);
                SetTimer(self.window, TIMER_ID, TICK_MS, None);
            }
        } else {
            unsafe {
                ShowWindow(self.window, SW_HIDE);
                KillTimer(self.window, TIMER_ID);
            }
        }
        save_settings(&self.settings);
    }

    fn set_locked(&mut self, locked: bool) {
        self.settings.locked = locked;
        self.apply_click_through();
        self.hover = false;
        self.dirty = true;
        self.flush();
        save_settings(&self.settings);
    }

    fn begin_drag(&mut self) {
        if self.settings.locked {
            return;
        }
        unsafe {
            let mut cursor = POINT { x: 0, y: 0 };
            GetCursorPos(&mut cursor);
            self.drag_pointer = cursor;
            self.drag_window = self.offset;
            self.dragging = true;
            SetCapture(self.window);
        }
    }

    fn update_drag(&mut self) {
        if !self.dragging {
            return;
        }
        unsafe {
            let mut cursor = POINT { x: 0, y: 0 };
            GetCursorPos(&mut cursor);
            let area = self.work_area();
            let x = (self.drag_window.x + cursor.x - self.drag_pointer.x)
                .clamp(area.left, (area.right - self.width).max(area.left));
            let y = (self.drag_window.y + cursor.y - self.drag_pointer.y)
                .clamp(area.top, (area.bottom - self.height).max(area.top));
            self.offset = POINT { x, y };
            SetWindowPos(
                self.window,
                HWND_TOPMOST,
                x,
                y,
                self.width,
                self.height,
                SWP_NOACTIVATE,
            );
        }
    }

    fn end_drag(&mut self) {
        if !self.dragging {
            return;
        }
        self.dragging = false;
        unsafe {
            ReleaseCapture();
        }
        self.settings.anchor = ANCHOR_CUSTOM;
        self.settings.custom_x = self.offset.x;
        self.settings.custom_y = self.offset.y;
        save_settings(&self.settings);
    }

    fn refresh_icon(&mut self) {
        unsafe {
            let size = GetSystemMetrics(SM_CXSMICON)
                .max(16)
                .min(GetSystemMetrics(SM_CYSMICON).max(16));
            let icon = build_icon(size, THEMES[self.settings.theme_index]);
            if !icon.is_null() {
                let previous = self.icon;
                self.icon = icon;
                self.tray_update();
                if !previous.is_null() {
                    DestroyIcon(previous);
                }
            }
        }
    }

    fn tray_add(&mut self) {
        unsafe {
            let mut data: NOTIFYICONDATAW = zeroed();
            data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = self.window;
            data.uID = TRAY_ID;
            data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            data.uCallbackMessage = TRAY_MESSAGE;
            data.hIcon = self.icon;
            let tip = self.texts.tooltip;
            for (index, code) in tip.encode_utf16().enumerate() {
                if index < data.szTip.len() - 1 {
                    data.szTip[index] = code;
                }
            }
            Shell_NotifyIconW(NIM_ADD, &data);
            self.tray_registered = true;
        }
    }

    fn tray_update(&mut self) {
        if !self.tray_registered {
            return;
        }
        unsafe {
            let mut data: NOTIFYICONDATAW = zeroed();
            data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = self.window;
            data.uID = TRAY_ID;
            data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            data.uCallbackMessage = TRAY_MESSAGE;
            data.hIcon = self.icon;
            Shell_NotifyIconW(NIM_MODIFY, &data);
        }
    }

    fn tray_remove(&mut self) {
        unsafe {
            let mut data: NOTIFYICONDATAW = zeroed();
            data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = self.window;
            data.uID = TRAY_ID;
            Shell_NotifyIconW(NIM_DELETE, &data);
            self.tray_registered = false;
        }
    }

    fn reset(&mut self) {
        unsafe {
            let autostart = autostart_enabled();
            self.settings = Settings::fresh();
            self.settings.autostart = autostart;
        }
        self.rebuild();
        self.apply_click_through();
        self.refresh_icon();
        self.set_visible(true);
    }
}

unsafe fn append_item(menu: HMENU, command: usize, text: &str, checked: bool) {
    let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
    AppendMenuW(menu, flags, command, wide(text).as_ptr());
}

unsafe fn append_submenu(menu: HMENU, label: &str, submenu: HMENU) {
    AppendMenuW(menu, MF_POPUP, submenu as usize, wide(label).as_ptr());
}

unsafe fn show_menu(application: &mut Application) {
    let texts = application.texts;
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return;
    }
    append_item(
        menu,
        CMD_VISIBLE,
        if application.settings.visible {
            texts.hide_hud
        } else {
            texts.show_hud
        },
        application.settings.visible,
    );
    append_item(
        menu,
        CMD_LOCK,
        if application.settings.locked {
            texts.unlock_drag
        } else {
            texts.lock_position
        },
        application.settings.locked,
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, null());

    let size_menu = CreatePopupMenu();
    for (index, name) in texts.scale_names.iter().enumerate() {
        append_item(
            size_menu,
            CMD_SIZE_BASE + index,
            name,
            application.settings.scale_index == index,
        );
    }
    append_submenu(menu, texts.size, size_menu);

    let opacity_menu = CreatePopupMenu();
    for (index, name) in OPACITY_NAMES.iter().enumerate() {
        append_item(
            opacity_menu,
            CMD_OPACITY_BASE + index,
            name,
            application.settings.opacity_index == index,
        );
    }
    append_submenu(menu, texts.opacity, opacity_menu);

    let theme_menu = CreatePopupMenu();
    for (index, name) in texts.theme_names.iter().enumerate() {
        append_item(
            theme_menu,
            CMD_THEME_BASE + index,
            name,
            application.settings.theme_index == index,
        );
    }
    append_submenu(menu, texts.theme, theme_menu);

    let anchor_menu = CreatePopupMenu();
    for (index, name) in texts.anchor_names.iter().enumerate() {
        append_item(
            anchor_menu,
            CMD_ANCHOR_BASE + index,
            name,
            application.settings.anchor == index,
        );
    }
    AppendMenuW(anchor_menu, MF_SEPARATOR, 0, null());
    append_item(
        anchor_menu,
        CMD_ANCHOR_BASE + ANCHOR_CUSTOM,
        texts.custom_drag,
        application.settings.anchor == ANCHOR_CUSTOM,
    );
    append_submenu(menu, texts.position, anchor_menu);

    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_item(menu, CMD_AUTOSTART, texts.autostart, application.settings.autostart);
    append_item(menu, CMD_RESET, texts.reset, false);
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_item(menu, CMD_EXIT, texts.quit, false);

    let mut cursor = POINT { x: 0, y: 0 };
    GetCursorPos(&mut cursor);
    SetForegroundWindow(application.window);
    let command = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
        cursor.x,
        cursor.y,
        0,
        application.window,
        null(),
    ) as usize;
    PostMessageW(application.window, 0, 0, 0);
    DestroyMenu(menu);

    if command != 0 {
        apply_command(application, command);
    }
}

unsafe fn apply_command(application: &mut Application, command: usize) {
    if command >= CMD_SIZE_BASE && command < CMD_SIZE_BASE + SCALES.len() {
        application.settings.scale_index = command - CMD_SIZE_BASE;
        application.rebuild();
        save_settings(&application.settings);
        application.flush();
        return;
    }
    if command >= CMD_OPACITY_BASE && command < CMD_OPACITY_BASE + OPACITIES.len() {
        application.settings.opacity_index = command - CMD_OPACITY_BASE;
        application.dirty = true;
        save_settings(&application.settings);
        application.flush();
        return;
    }
    if command >= CMD_THEME_BASE && command < CMD_THEME_BASE + THEMES.len() {
        application.settings.theme_index = command - CMD_THEME_BASE;
        application.refresh_icon();
        application.dirty = true;
        save_settings(&application.settings);
        application.flush();
        return;
    }
    if command >= CMD_ANCHOR_BASE && command <= CMD_ANCHOR_BASE + ANCHOR_CUSTOM {
        let anchor = command - CMD_ANCHOR_BASE;
        if anchor != ANCHOR_CUSTOM {
            application.settings.anchor = anchor;
            application.reposition();
            application.dirty = true;
            save_settings(&application.settings);
            application.flush();
        }
        return;
    }
    match command {
        CMD_VISIBLE => {
            let visible = !application.settings.visible;
            application.set_visible(visible);
        }
        CMD_LOCK => {
            let locked = !application.settings.locked;
            application.set_locked(locked);
        }
        CMD_AUTOSTART => {
            let enabled = !application.settings.autostart;
            set_autostart(enabled);
            application.settings.autostart = autostart_enabled();
        }
        CMD_RESET => {
            application.reset();
        }
        CMD_EXIT => {
            PostQuitMessage(0);
        }
        _ => {}
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let pointer = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Application;
    if pointer.is_null() {
        return DefWindowProcW(window, message, wparam, lparam);
    }
    let application = &mut *pointer;
    if message == application.taskbar_created {
        application.tray_registered = false;
        application.tray_add();
        return 0;
    }
    match message {
        WM_TIMER => {
            if GetAsyncKeyState(VK_END as i32) & EXIT_KEY_DOWN != 0 {
                PostQuitMessage(0);
                return 0;
            }
            application.tick();
            application.flush();
            0
        }
        TRAY_SHOW => {
            if !application.settings.visible {
                application.set_visible(true);
            }
            0
        }
        TRAY_MESSAGE => {
            match lparam as u32 {
                WM_LBUTTONDBLCLK => {
                    let visible = !application.settings.visible;
                    application.set_visible(visible);
                }
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    show_menu(application);
                }
                _ => {}
            }
            0
        }
        WM_LBUTTONDOWN => {
            application.begin_drag();
            0
        }
        WM_MOUSEMOVE => {
            application.update_drag();
            0
        }
        WM_LBUTTONUP => {
            application.end_drag();
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let singleton = CreateMutexW(null(), 1, wide("Local\\SimpleKeystrokeSingleton").as_ptr());
        if singleton.is_null() {
            return;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let existing = FindWindowW(wide(WINDOW_CLASS).as_ptr(), null());
            if !existing.is_null() {
                PostMessageW(existing, TRAY_SHOW, 0, 0);
            }
            return;
        }
        let texts = system_texts();
        let dpi_scale = GetDpiForSystem() as f32 / 96.0;
        let settings = load_settings();
        let unit = dpi_scale * SCALES[settings.scale_index];
        let keys = build_keys(unit);
        let (width, height) = canvas_size(&keys, unit);

        let instance = GetModuleHandleW(null());
        let class_name = wide(WINDOW_CLASS);
        let mut window_class: WNDCLASSEXW = zeroed();
        window_class.cbSize = size_of::<WNDCLASSEXW>() as u32;
        window_class.lpfnWndProc = Some(window_proc);
        window_class.hInstance = instance;
        window_class.hCursor = LoadCursorW(null_mut(), IDC_SIZEALL);
        window_class.lpszClassName = class_name.as_ptr();
        if RegisterClassExW(&window_class) == 0 {
            return;
        }
        let window = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            null(),
            WS_POPUP,
            0,
            0,
            width,
            height,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if window.is_null() {
            return;
        }

        let screen = GetDC(null_mut());
        let device = CreateCompatibleDC(screen);
        ReleaseDC(null_mut(), screen);
        let (bitmap, bits) = create_surface(device, width, height);
        if bitmap.is_null() || bits.is_null() {
            return;
        }

        let icon_size = GetSystemMetrics(SM_CXSMICON)
            .max(16)
            .min(GetSystemMetrics(SM_CYSMICON).max(16));
        let icon = build_icon(icon_size, THEMES[settings.theme_index]);
        let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());

        let application = Box::into_raw(Box::new(Application {
            window,
            painter: Painter::new(width, height),
            keys,
            settings,
            texts,
            dpi_scale,
            unit,
            device,
            bitmap,
            bits,
            icon,
            width,
            height,
            offset: POINT { x: 0, y: 0 },
            dirty: true,
            hover: false,
            dragging: false,
            drag_pointer: POINT { x: 0, y: 0 },
            drag_window: POINT { x: 0, y: 0 },
            taskbar_created,
            tray_registered: false,
        }));
        SetWindowLongPtrW(window, GWLP_USERDATA, application as isize);

        let state = &mut *application;
        state.reposition();
        state.apply_click_through();
        state.tray_add();
        state.render();
        state.present();
        if state.settings.visible {
            ShowWindow(window, SW_SHOW);
            SetTimer(window, TIMER_ID, TICK_MS, None);
        } else {
            ShowWindow(window, SW_HIDE);
        }

        let mut message: MSG = zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        let mut application = Box::from_raw(application);
        application.tray_remove();
        DestroyWindow(window);
        if !application.bitmap.is_null() {
            DeleteObject(application.bitmap as _);
        }
        DeleteDC(application.device);
        if !application.icon.is_null() {
            DestroyIcon(application.icon);
        }
        save_settings(&application.settings);
    }
}
