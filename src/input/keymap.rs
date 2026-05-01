use anyhow::{bail, Result};
use evdev::Key;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

fn build_key_map() -> HashMap<&'static str, Key> {
    let mut m = HashMap::new();

    m.insert("numpad_0", Key::KEY_KP0);
    m.insert("numpad_1", Key::KEY_KP1);
    m.insert("numpad_2", Key::KEY_KP2);
    m.insert("numpad_3", Key::KEY_KP3);
    m.insert("numpad_4", Key::KEY_KP4);
    m.insert("numpad_5", Key::KEY_KP5);
    m.insert("numpad_6", Key::KEY_KP6);
    m.insert("numpad_7", Key::KEY_KP7);
    m.insert("numpad_8", Key::KEY_KP8);
    m.insert("numpad_9", Key::KEY_KP9);
    m.insert("numpad_enter", Key::KEY_KPENTER);
    m.insert("numpad_decimal", Key::KEY_KPDOT);
    m.insert("numpad_dot", Key::KEY_KPDOT);
    m.insert("numpad_plus", Key::KEY_KPPLUS);
    m.insert("numpad_add", Key::KEY_KPPLUS);
    m.insert("numpad_minus", Key::KEY_KPMINUS);
    m.insert("numpad_subtract", Key::KEY_KPMINUS);
    m.insert("numpad_multiply", Key::KEY_KPASTERISK);
    m.insert("numpad_divide", Key::KEY_KPSLASH);
    m.insert("numpad_clear", Key::KEY_NUMLOCK);
    m.insert("numpad_equals", Key::KEY_KPEQUAL);

    m.insert("right_option", Key::KEY_RIGHTALT);
    m.insert("right_alt", Key::KEY_RIGHTALT);
    m.insert("left_option", Key::KEY_LEFTALT);
    m.insert("left_alt", Key::KEY_LEFTALT);
    m.insert("right_command", Key::KEY_RIGHTMETA);
    m.insert("right_cmd", Key::KEY_RIGHTMETA);
    m.insert("left_command", Key::KEY_LEFTMETA);
    m.insert("left_cmd", Key::KEY_LEFTMETA);
    m.insert("right_shift", Key::KEY_RIGHTSHIFT);
    m.insert("left_shift", Key::KEY_LEFTSHIFT);
    m.insert("right_control", Key::KEY_RIGHTCTRL);
    m.insert("right_ctrl", Key::KEY_RIGHTCTRL);
    m.insert("left_control", Key::KEY_LEFTCTRL);
    m.insert("left_ctrl", Key::KEY_LEFTCTRL);
    m.insert("fn_key", Key::KEY_FN);
    m.insert("fn", Key::KEY_FN);
    m.insert("caps_lock", Key::KEY_CAPSLOCK);

    m.insert("f1", Key::KEY_F1);
    m.insert("f2", Key::KEY_F2);
    m.insert("f3", Key::KEY_F3);
    m.insert("f4", Key::KEY_F4);
    m.insert("f5", Key::KEY_F5);
    m.insert("f6", Key::KEY_F6);
    m.insert("f7", Key::KEY_F7);
    m.insert("f8", Key::KEY_F8);
    m.insert("f9", Key::KEY_F9);
    m.insert("f10", Key::KEY_F10);
    m.insert("f11", Key::KEY_F11);
    m.insert("f12", Key::KEY_F12);

    m.insert("f13", Key::KEY_F13);
    m.insert("f14", Key::KEY_F14);
    m.insert("f15", Key::KEY_F15);
    m.insert("f16", Key::KEY_F16);
    m.insert("f17", Key::KEY_F17);
    m.insert("f18", Key::KEY_F18);
    m.insert("f19", Key::KEY_F19);
    m.insert("f20", Key::KEY_F20);

    m.insert("space", Key::KEY_SPACE);
    m.insert("tab", Key::KEY_TAB);
    m.insert("escape", Key::KEY_ESC);
    m.insert("delete", Key::KEY_BACKSPACE);
    m.insert("forward_delete", Key::KEY_DELETE);
    m.insert("return_key", Key::KEY_ENTER);
    m.insert("return", Key::KEY_ENTER);
    m.insert("enter", Key::KEY_ENTER);
    m.insert("home", Key::KEY_HOME);
    m.insert("end", Key::KEY_END);
    m.insert("page_up", Key::KEY_PAGEUP);
    m.insert("page_down", Key::KEY_PAGEDOWN);
    m.insert("up_arrow", Key::KEY_UP);
    m.insert("down_arrow", Key::KEY_DOWN);
    m.insert("left_arrow", Key::KEY_LEFT);
    m.insert("right_arrow", Key::KEY_RIGHT);
    m.insert("insert", Key::KEY_INSERT);
    m.insert("print_screen", Key::KEY_SYSRQ);
    m.insert("scroll_lock", Key::KEY_SCROLLLOCK);
    m.insert("pause", Key::KEY_PAUSE);
    m.insert("num_lock", Key::KEY_NUMLOCK);

    m.insert("section", Key::KEY_102ND);
    m.insert("grave", Key::KEY_GRAVE);
    m.insert("minus", Key::KEY_MINUS);
    m.insert("equal", Key::KEY_EQUAL);
    m.insert("left_bracket", Key::KEY_LEFTBRACE);
    m.insert("right_bracket", Key::KEY_RIGHTBRACE);
    m.insert("backslash", Key::KEY_BACKSLASH);
    m.insert("semicolon", Key::KEY_SEMICOLON);
    m.insert("quote", Key::KEY_APOSTROPHE);
    m.insert("comma", Key::KEY_COMMA);
    m.insert("period", Key::KEY_DOT);
    m.insert("slash", Key::KEY_SLASH);

    m
}

static KEY_MAP: OnceLock<HashMap<&'static str, Key>> = OnceLock::new();

fn get_key_map() -> &'static HashMap<&'static str, Key> {
    KEY_MAP.get_or_init(build_key_map)
}

fn build_modifier_aliases() -> HashMap<&'static str, Vec<Key>> {
    let mut m = HashMap::new();
    m.insert("ctrl", vec![Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL]);
    m.insert("shift", vec![Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT]);
    m.insert("alt", vec![Key::KEY_LEFTALT, Key::KEY_RIGHTALT]);
    m.insert("super", vec![Key::KEY_LEFTMETA, Key::KEY_RIGHTMETA]);
    m
}

static MODIFIER_ALIASES: OnceLock<HashMap<&'static str, Vec<Key>>> = OnceLock::new();

fn get_modifier_aliases() -> &'static HashMap<&'static str, Vec<Key>> {
    MODIFIER_ALIASES.get_or_init(build_modifier_aliases)
}

static MODIFIER_KEYS: OnceLock<HashSet<Key>> = OnceLock::new();

fn get_modifier_keys() -> &'static HashSet<Key> {
    MODIFIER_KEYS.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert(Key::KEY_LEFTCTRL);
        set.insert(Key::KEY_RIGHTCTRL);
        set.insert(Key::KEY_LEFTSHIFT);
        set.insert(Key::KEY_RIGHTSHIFT);
        set.insert(Key::KEY_LEFTALT);
        set.insert(Key::KEY_RIGHTALT);
        set.insert(Key::KEY_LEFTMETA);
        set.insert(Key::KEY_RIGHTMETA);
        set
    })
}

#[must_use]
pub fn is_modifier_key(key: Key) -> bool {
    get_modifier_keys().contains(&key)
}

#[allow(clippy::missing_errors_doc)]
pub fn parse_key_name(name: &str) -> Result<Key> {
    let key_map = get_key_map();

    if let Some(&key) = key_map.get(name) {
        return Ok(key);
    }

    if let Some(hex) = name.strip_prefix("0x").or_else(|| name.strip_prefix("0X")) {
        if let Ok(n) = u16::from_str_radix(hex, 16) {
            return Ok(Key::new(n));
        }
    }

    if let Ok(n) = name.parse::<u16>() {
        return Ok(Key::new(n));
    }

    bail!("Unknown key name: '{name}'. Use 'sittard list-keys' to see valid names.")
}

#[derive(Clone)]
pub struct HotkeyCombo {
    pub slots: Vec<HashSet<Key>>,
    pub is_modifier_only: bool,
    pub all_keys: HashSet<Key>,
}

impl HotkeyCombo {
    #[must_use]
    pub fn matches(&self, held: &HashSet<Key>) -> bool {
        if held.is_empty() {
            return false;
        }
        self.slots
            .iter()
            .all(|slot| slot.iter().any(|k| held.contains(k)))
            && held.iter().all(|k| self.all_keys.contains(k))
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn parse_hotkey_combo(spec: &str) -> Result<HotkeyCombo> {
    let tokens: Vec<&str> = spec.split('+').map(str::trim).collect();
    if tokens.is_empty() {
        bail!("empty hotkey specification");
    }

    let mut slots: Vec<HashSet<Key>> = Vec::new();
    let mut all_keys = HashSet::new();
    let mut modifier_count = 0usize;

    for token in &tokens {
        if let Some(variants) = get_modifier_aliases().get(token) {
            let slot: HashSet<Key> = variants.iter().copied().collect();
            all_keys.extend(&slot);
            slots.push(slot);
            modifier_count += 1;
        } else {
            let key = parse_key_name(token)?;
            let mut slot = HashSet::new();
            slot.insert(key);
            all_keys.insert(key);
            slots.push(slot);
        }
    }

    if slots.len() > 8 {
        bail!("hotkey combo too complex: max 8 keys");
    }

    let is_modifier_only = modifier_count == slots.len();

    Ok(HotkeyCombo {
        slots,
        is_modifier_only,
        all_keys,
    })
}

#[must_use]
pub fn list_key_names() -> Vec<&'static str> {
    let key_map = get_key_map();
    let mut keys: Vec<_> = key_map.keys().copied().collect();
    keys.sort_unstable();
    keys
}

#[must_use]
pub fn list_modifier_aliases() -> Vec<&'static str> {
    let aliases = get_modifier_aliases();
    let mut names: Vec<_> = aliases.keys().copied().collect();
    names.sort_unstable();
    names
}
