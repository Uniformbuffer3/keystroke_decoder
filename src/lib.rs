use xkbcommon::xkb;
pub use xkb::keysyms;

/**
This structure initialize the xkb components to decode key strokes to an abstract xkb representation,
making possible to handle multiple keyboards layouts.
*/
pub struct KeystrokeDecoder {
    context: xkb::Context,
    keymap: xkb::Keymap,
    state: xkb::State,
    keystroke_stack: Vec<xkb::Keysym>
}
impl KeystrokeDecoder {
    fn detect_keyboard_layout_from_env() -> Result<String, ()> {
        for (var, value) in std::env::vars() {
            if var == "XKB_DEFAULT_LAYOUT" {
                return Ok(value);
            }
        }
        Err(())
    }

    fn detect_keyboard_layout_from_file() -> Result<String, ()> {
        let regex = regex::Regex::new(r"\s*XKBLAYOUT\s*=(.+)").unwrap();

        let file_data = std::fs::read_to_string("/etc/default/keyboard").unwrap();
        for line in file_data.lines() {
            if let Some(capture) = regex.captures(line) {
                return Ok(capture.get(1).unwrap().as_str().to_string());
            };
        }
        Err(())
    }

    fn detect_keyboard_layout() -> Result<String, ()> {
        //Try to detect from env
        if let Ok(layout) = Self::detect_keyboard_layout_from_env() {
            return Ok(layout);
        }

        //Try to detect from file
        if let Ok(layout) = Self::detect_keyboard_layout_from_file() {
            return Ok(layout);
        }
        Err(())
    }

    pub fn new() -> Self {
        // Initializing the xkb context with no flags
        let context = xkb::Context::new(0);

        // Detecting keyboard layout
        let keyboard_layout = match Self::detect_keyboard_layout() {
            Ok(keyboard_layout) => {
                println!("Detected layout: {}", &keyboard_layout);
                keyboard_layout
            }
            Err(_) => String::from(""),
        };

        // Initializing the keymap using empty values ("").
        // This will make xkb detect automatically the system keymap.
        let keymap = xkb::Keymap::new_from_names(&context, "", "", &keyboard_layout, "", None, 0)
            .expect("Fauled to create keymap");

        // Initializing the xkb state that will be used to decode keystrokes
        let state = xkb::State::new(&keymap);

        let keystroke_stack = Vec::new();

        Self {
            context,
            keymap,
            state,
            keystroke_stack
        }
    }
    /// This function will decode the key into an abstract xkb representation (Keysym).
    /// The keycode will be increased by 8 because the evdev XKB rules reflect X's
    /// broken keycode system, which starts at 8
    pub fn decode_as_keysym(&mut self, keycode: u32) -> &[xkb::Keysym] {
        let direction = match self.keystroke_stack.iter().rposition(|element|element==&keycode) {
            Some(index)=>{
                self.keystroke_stack.remove(index);
                xkb::KeyDirection::Up
            }
            None=>{
                self.keystroke_stack.push(keycode);
                xkb::KeyDirection::Down
            }
        };
        self.state.update_key(keycode + 8, direction);
        self.state.key_get_syms(keycode + 8)
    }
    pub fn decode_as_chars(&mut self, keycode: u32) -> Vec<char> {
        let direction = match self.keystroke_stack.iter().rposition(|element|element==&keycode) {
            Some(index)=>{
                self.keystroke_stack.remove(index);
                xkb::KeyDirection::Up
            }
            None=>{
                self.keystroke_stack.push(keycode);
                xkb::KeyDirection::Down
            }
        };
        self.state.update_key(keycode + 8, direction);
        self.state.key_get_utf8(keycode + 8).chars().collect()
    }

    pub fn is_ctrl_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_alt_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_shift_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_logo_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_caps_lock_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_num_lock_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
}

impl Default for KeystrokeDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_automatic_layout_detection() {
    println!("{}", KeystrokeDecoder::detect_keyboard_layout().expect("Failed to autodetect keyboard layout"));
}
