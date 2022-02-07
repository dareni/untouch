mod params;

use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};

use input::{Libinput, LibinputInterface};

extern crate libc;
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32)
      -> Result<RawFd, i32> {
        match path.to_str() {
            Some(x) => println!("PATH:{}", x),
            None => (),
        }

        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: RawFd) {
        println!("close");
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

fn main() {
    let params = params::get_params();

    let keyboards = untouch::get_keyboards();
    if keyboards.len() <= 0 {
        eprintln!("No keyboard device found!");
        std::process::exit(1);
    }

    if params.do_config == true {
        params::do_config_file(&keyboards);
        return ();
    }

    let mut device = &keyboards[0];
    if keyboards.len() > 1 {
        device = params::get_keyboard_selection(&keyboards);
    }
    println!("start");
    //let mut input = Libinput::new_with_udev(Interface);
    //input.udev_assign_seat("seat0").unwrap();
    let mut input = Libinput::new_from_path(Interface);

    let _keyboard_device = input.path_add_device(&device.path);

    println!("assign done");
    loop {
        input.dispatch().unwrap();

        //use input::event::EventTrait;
        //use input::event::keyboard::KeyboardEvent;
        use input::event::keyboard::KeyState;
        use input::event::keyboard::KeyboardEvent::Key;
        use input::event::keyboard::KeyboardEventTrait;
        use input::event::Event::Keyboard;

        static LCTL: u8 = 29;
        static LALT: u8 = 56;
        static RALT: u8 = 100;
        static RCTL: u8 = 97;

        static EXCLUDE_LIST: [u8; 4] = [LCTL, LALT, RALT, RCTL];

        for event in &mut input {
            if let Keyboard(keyboard_event) = &event {
                if let Key(keyboard_key_event) = keyboard_event {
                    let key_event = keyboard_key_event
                      as &dyn KeyboardEventTrait;
                    if key_event.key_state() == KeyState::Pressed {
                        let keycode: u32 = key_event.key();
                        let exclude = EXCLUDE_LIST
                            .iter()
                            .find(|x| keycode == **x as u32)
                            .is_none();
                        println!("key:{}{}", keycode, exclude);
                    }
                }
            }
        }
    }
}
