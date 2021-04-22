use xkbcommon::xkb;
pub use xkb::keysyms;

/**
This structure initialize the xkb components to decode key strokes to an abstract xkb representation,
making possible to handle multiple keyboards layouts.
*/
pub struct KeystrokeDecoder {
    context: xkb::Context,
    keymap: xkb::Keymap,
    pub(crate) state: xkb::State,

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
        self.state.update_key(keycode, direction.into());

        Keystrokes {
            state: &self.state,
            keycode,
            direction
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


#[derive(Clone,Copy,Debug)]
pub enum KeyDirection {
    Up,
    Down
}
impl From<xkb::KeyDirection> for KeyDirection {
    fn from(direction: xkb::KeyDirection)->Self {
        match direction {
            xkb::KeyDirection::Up=>Self::Up,
            xkb::KeyDirection::Down=>Self::Down
        }
    }
}

impl Into<xkb::KeyDirection> for KeyDirection {
    fn into(self)->xkb::KeyDirection {
        match self {
            Self::Up=>xkb::KeyDirection::Up,
            Self::Down=>xkb::KeyDirection::Down
        }
    }
}

pub struct Keystrokes<'a> {
    state: &'a xkb::State,
    keycode: u32,
    direction: KeyDirection
}
impl<'a> Keystrokes<'a> {
    /// Return the keystrokes as keysyms
    pub fn as_keysym(&self) -> Vec<(xkb::Keysym,KeyDirection)> {
        self.state.key_get_syms(self.keycode).into_iter().map(|key|(*key,self.direction.clone())).collect()
    }
    /// Return the keystrokes as chars. Only key down direction will be returned.
    pub fn as_chars(&self) -> Vec<char> {
        match self.direction
        {
            KeyDirection::Up => {return Vec::new();}
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
fn test_automatic_layout_detection() {
    println!("{}", KeystrokeDecoder::detect_keyboard_layout().expect("Failed to autodetect keyboard layout"));
}


#[test]
fn test_gatherer() {
    use input_gatherer::*;

    //Creating the gatherer
    let mut gatherer = InputGatherer::new();
    //Creating the keyboard decoder
    let mut keystroke_decoder = KeystrokeDecoder::new();

    let start = std::time::Instant::now();
    let mut running = true;
    while running {
        //Dispatching events
        gatherer
            .dispatch_new_events(|event, _config| match event {
                InputEvent::NewSeat(seat) => {
                    println!("Seat added: {:#?}", seat);
                }
                InputEvent::SeatChanged(seat) => {
                    println!("Seat changed: {:#?}", seat);
                }
                InputEvent::SeatRemoved(seat) => {
                    println!("Seat removed: {:#?}", seat);
                }
                InputEvent::Keyboard { seat: _, event } => {
                    // Decoding keystrokes
                    let keystrokes = keystroke_decoder.decode(event.key());

                    //Decoding keys into chars
                    for key in keystrokes.as_chars() {
                        println!("{}", key);
                    }
                    //Decoding keys into keysym
                    for key_and_direction in keystrokes.as_keysym() {
                        println!("{:#?}",&key_and_direction);
                        match key_and_direction {
                            (keysyms::KEY_Escape,KeyDirection::Up) => {
                                println!("Esc pressed, early exit");
                                running = false;
                            }
                            _ => {}
                        }
                    }

                }
                InputEvent::PointerMotion { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::PointerMotionAbsolute { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::PointerButton { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::PointerAxis { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::TouchDown { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::TouchMotion { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::TouchUp { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::TouchCancel { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::TouchFrame { seat: _, event } => {
                    println!("{:#?}", event);
                }
                InputEvent::Special(event) => {
                    println!("{:#?}", event);
                }
            })
            .unwrap();

        //After 5 seconds the loop terminate and give the control back to the terminal
        if start.elapsed().as_secs() >= 5 {
            running = false;
        }
    }
}
