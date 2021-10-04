pub use xkbcommon::xkb;
pub use xkb::keysyms;
pub use xkb::KeyDirection;

#[derive(Debug,Clone)]
pub struct Rmlvo {
    rules: String,
    model: String,
    layout: String,
    variant: String,
    options: Option<String>,
}
impl Rmlvo {
    pub fn new()->Self {
        let mut rmlvo = Self::default();
        rmlvo.detect_from_etc_default_keyboard();
        rmlvo.detect_from_env();
        rmlvo
    }


    fn detect_from_etc_default_keyboard(&mut self){
        let file_data = match std::fs::read_to_string("/etc/default/keyboard"){
            Ok(file_data)=>file_data,
            Err(_)=>return
        };

        let rules_regex = regex::Regex::new(r#"\s*XKBRULES\s*="(.+)""#).unwrap();
        let model_regex = regex::Regex::new(r#"\s*XKBMODEL\s*="(.+)""#).unwrap();
        let layout_regex = regex::Regex::new(r#"\s*XKBLAYOUT\s*="(.+)""#).unwrap();
        let variant_regex = regex::Regex::new(r#"\s*XKBVARIANT\s*="(.+)""#).unwrap();
        let options_regex = regex::Regex::new(r#"\s*XKBOPTIONS\s*="(.+)""#).unwrap();

        for line in file_data.lines() {
            if let Some(capture) = rules_regex.captures(line) {
                self.rules = capture.get(1).unwrap().as_str().to_string();
            };
            if let Some(capture) = model_regex.captures(line) {
                self.model = capture.get(1).unwrap().as_str().to_string();
            };
            if let Some(capture) = layout_regex.captures(line) {
                self.layout = capture.get(1).unwrap().as_str().split(',').next().unwrap().to_string();
            };
            if let Some(capture) = variant_regex.captures(line) {
                self.variant = capture.get(1).unwrap().as_str().to_string();
            };
            if let Some(capture) = options_regex.captures(line) {
                self.options = Some(capture.get(1).unwrap().as_str().to_string());
            };
        }
    }

    fn detect_from_env(&mut self){
        for (var, value) in std::env::vars() {
            match var.as_str() {
                "XKB_DEFAULT_RULES"=>self.rules = value,
                "XKB_DEFAULT_MODEL"=>self.model = value,
                "XKB_DEFAULT_LAYOUT"=>self.layout = value,
                "XKB_DEFAULT_VARIANT"=>self.variant = value,
                "XKB_DEFAULT_OPTIONS"=>self.options = Some(value),
                _=>()
            }
        }
    }

    pub fn rules(&self)->&String {&self.rules}
    pub fn model(&self)->&String {&self.model}
    pub fn layout(&self)->&String {&self.layout}
    pub fn variant(&self)->&String {&self.variant}
    pub fn options(&self)->&Option<String> {&self.options}
}

impl Default for Rmlvo {
    fn default()->Self {
        Self {
            rules: String::new(),
            model: String::new(),
            layout: String::new(),
            variant: String::new(),
            options: None
        }
    }
}

/**
This structure initialize the xkb components to decode key strokes to an abstract xkb representation,
making possible to handle multiple keyboards layouts.
*/
pub struct KeystrokeDecoder {
    context: xkb::Context,
    rmlvo: Rmlvo,
    keymap: xkb::Keymap,
    state: xkb::State,
    keystroke_stack: Vec<xkb::Keysym>
}
impl KeystrokeDecoder {
    pub fn new() -> Self {
        // Initializing the xkb context with no flags
        let context = xkb::Context::new(0);

        let rmlvo = Rmlvo::new();
        let keymap = xkb::Keymap::new_from_names(&context, rmlvo.rules(), rmlvo.model(), rmlvo.layout(), rmlvo.variant(), rmlvo.options().clone(), 0)
            .expect("Failed to create keymap");

        // Initializing the xkb state that will be used to decode keystrokes
        let state = xkb::State::new(&keymap);

        let keystroke_stack = Vec::new();

        Self {
            context,
            rmlvo,
            keymap,
            state,
            keystroke_stack
        }
    }

    /// This function will decode the key into an abstract xkb representation (Keysym).
    /// The keycode will be increased by 8 because the evdev XKB rules reflect X's
    /// broken keycode system, which starts at 8
    pub fn decode(&mut self, keycode: u32)->Keystrokes {
        let direction = match self.keystroke_stack.iter().rposition(|element|element==&keycode) {
            Some(index)=>{
                self.keystroke_stack.remove(index);
                KeyDirection::Up
            }
            None=>{
                self.keystroke_stack.push(keycode);
                KeyDirection::Down
            }
        };
        let keycode = keycode + 8;
        self.state.update_key(keycode,match direction{
            KeyDirection::Up=>KeyDirection::Up,
            KeyDirection::Down=>KeyDirection::Down,
        });

        Keystrokes {
            state: &self.state,
            keycode,
            direction
        }
    }

    pub fn layout(&self)->&String {self.rmlvo.layout()}
    pub fn set_layout(&mut self, layout: String)->bool {
        let mut rmlvo = self.rmlvo.clone();
        rmlvo.layout = layout;
        match xkb::Keymap::new_from_names(&self.context, rmlvo.rules(), rmlvo.model(), rmlvo.layout(), rmlvo.variant(), rmlvo.options().clone(), 0)
        {
            Some(keymap)=>{self.keymap = keymap;true}
            None=>false
        }
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

pub struct Keystrokes<'a> {
    state: &'a xkb::State,
    keycode: u32,
    direction: KeyDirection
}
impl<'a> Keystrokes<'a> {
    /// Return the keystrokes as keysyms
    pub fn as_keysyms(&self) -> Vec<(xkb::Keysym,KeyDirection)> {
        self.state.key_get_syms(self.keycode).iter().map(|key|{
            let direction = match self.direction{
                KeyDirection::Up=>KeyDirection::Up,
                KeyDirection::Down=>KeyDirection::Down,
            };
            (*key,direction)
        }).collect()
    }
    /// Return the keystrokes as chars. Only key down direction will be returned.
    pub fn as_chars(&self) -> Vec<char> {
        match self.direction
        {
            KeyDirection::Up => Vec::new(),
            KeyDirection::Down => {
                let chars: Vec<char> = self.state.key_get_utf8(self.keycode).chars().collect();
                if self.is_shift_pressed() {chars.into_iter().map(|character|character.to_uppercase().into_iter().collect::<Vec<char>>()).flatten().collect()}
                else {chars}
            }
        }
    }

    pub fn is_ctrl_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_alt_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_shift_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_logo_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_caps_lock_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
    pub fn is_num_lock_pressed(&self)->bool {self.state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)}
}


#[test]
fn test_create(){
    let decoder = KeystrokeDecoder::new();

    println!("{:#?}",decoder.layout());
    //keymap.leds().for_each(|item|println!("{:#?}",item));
    //keymap.layouts().for_each(|item|println!("{:#?}",item));

}

