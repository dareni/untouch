//Device capability definitions for events:
// /usr/include/linux/input-event-codes.h
// EV_SYN:synchronization_event
// EV_KEY:key_events
// EV_REL:relative_movement_events
// abs:absolute_axes
// ev:event_types
// keys:keys_and_buttons
// rel:relative_axes

// udevadm test-builtin --help
// https://github.com/systemd/systemd/blob/master/src/udev/udev-builtin-kmod.c
// https://github.com/systemd/systemd/blob/main/src/udev/udev-builtin-input_id.c
// From udev/udev-builtin-input_id.c
// mouse
//  :needs  EV_REL REL_X REL_Y BTN_MOUSE
// keyboard
//  :needs EV_SYN 0x00 and EV_KEY 0x01 at a minimum
//   usually also have EV_MSC,EV_LED,EV_REP.
//  :bits 1-31 set in key (escape to D) bit 0 is reserved
//   see /usr/include/linux/input-event-codes.h
// touchpad

//atkeybard
// /sys/devices/platform/i8042/serio0/input/input0/capabilities/
// abs  ev     ff  key           led msc rel snd sw
// 0    120013 0                 7   10  0   0   0
//                 402000000 3803078f800d001 feffffdfffefffff fffffffffffffffe

//mouse
// /sys/devices/pci0000:00/0000:00:1d.0/usb2/2-1/2-1.6/2-1.6:1.0/
//           0003:0000:3825.0001/input/input1/capabilities/
// abs  ev     ff  key           led msc rel snd sw
// 0    17     0   70000 0 0 0 0 0   10  103 0   0

static INPUT_DEVICES: &str = "/sys/class/input";

static DEV_NAME: &str = "/name";
static CAPS: &str = "/capabilities/";
static CAPS_EV: &str = "ev";
static CAPS_KEY: &str = "key";
static UEVENT: &str = "/uevent";
static DEVNAME: &str = "DEVNAME";
static DEV: &str = "/dev/";

static KEYBOARD_KEYS_MASK: u64 = u64::MAX - 1;
static EV_SYNC: u8 = 0;
static EV_KEY: u8 = 1;

use regex::Regex;
use std::fs;

#[derive(Debug)]
pub struct Keyboard {
    pub name: String,
    pub path: String,
}
impl<'a> Keyboard {
    pub fn new(name: String, path: String) -> Keyboard {
        Keyboard { name, path }
    }
}

pub fn get_keyboards() -> Vec<Keyboard> {
    let entries: Vec<Keyboard> = fs::read_dir(&INPUT_DEVICES)
        .expect("Incorrect input device location??")
        .filter_map(|dir_entry| get_keyboard_device(dir_entry))
        .collect();
    return entries;
}

pub fn get_keyboard_device(dir_entry: Result<fs::DirEntry, std::io::Error>) -> Option<Keyboard> {
    let entry: fs::DirEntry = match dir_entry {
        Ok(x) => x,
        Err(_) => {
            eprintln!("read_dir error for:{:?}", dir_entry);
            return None;
        }
    };

    //Get the absolute path from the symbolic link.
    let tmp_path = &entry;
    let entry = match std::fs::canonicalize(entry.path()) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("error obtaining absolute path for link:{:?}", entry);
            return None;
        }
    };

    let device_path = match entry.into_os_string().into_string() {
        Ok(x) => x,
        Err(_) => {
            eprintln!("read_dir error for:{:?}", tmp_path);
            return None;
        }
    };

    //Get the device name
    let mut device_name_path = String::from(&device_path);
    device_name_path.push_str(DEV_NAME);
    let dev_name = fs::read_to_string(&device_name_path);
    let dev_name = match dev_name {
        Ok(name) => name,
        Err(_) => {
            //file does not exist for device
            return None;
        }
    };
    let dev_name = dev_name.trim();

    let mut device_ev_path = String::from(&device_path);
    device_ev_path.push_str(CAPS);
    device_ev_path.push_str(CAPS_EV);
    let ev_cap = fs::read_to_string(&device_ev_path);
    let ev_cap = match ev_cap {
        Ok(ev) => ev,
        Err(_) => {
            //ev cap does not exist for device
            return None;
        }
    };

    if !keyboard_ev_cap_test(&ev_cap) {
        return None;
    }

    let mut device_key_path = String::from(&device_path);
    device_key_path.push_str(CAPS);
    device_key_path.push_str(CAPS_KEY);
    let key_cap = fs::read_to_string(&device_key_path);
    let key_cap = match key_cap {
        Ok(key) => key,
        Err(_) => {
            //ev cap does not exist for device
            return None;
        }
    };

    if !keyboard_key_cap_test(&key_cap) {
        return None;
    }

    let event_regex = Regex::new(r"event\d+$").unwrap();

    let events: Vec<String> = std::fs::read_dir(&device_path)
        .unwrap_or_else(|_|
            panic!("Failed read of device directory:'{}'", &device_path))
        .filter_map(|x| match x {
            Ok(file) => match file.path().into_os_string().into_string() {
                Ok(osf) => {
                    if event_regex.is_match(&osf) {
                        Some(osf)
                    } else {
                        None
                    }
                }
                _ => None,
            },

            _ => None,
        })
        .collect();

    assert!(events.len() == 1, "Error, multiple events.");

    let mut event_uevent = String::from(&events[0]);
    event_uevent.push_str(UEVENT);
    let uevent = fs::read_to_string(&event_uevent);
    let uevent: String = match uevent {
        Ok(x) => x,
        Err(_) => {
            //uevent file does not exist for device
            return None;
        }
    };
    let device = match get_uevent_device(uevent) {
        Some(dev) => dev,
        None => {
            return None;
        }
    };
    let mut device = String::from(device);
    device.insert_str(0, DEV);

    let keyboard = Keyboard::new(String::from(dev_name), device);
    Some(keyboard)
}

fn keyboard_ev_cap_test(ev_cap: &str) -> bool {
    let ev_num = convert_hex_string(ev_cap);
    let mask = mask_build(&[EV_SYNC, EV_KEY]);
    bits_set(ev_num, mask)
}

fn keyboard_key_cap_test(key_cap: &str) -> bool {
    let key = convert_hex_string(key_cap);
    bits_set(key, KEYBOARD_KEYS_MASK)
}

fn convert_hex_string(hex: &str) -> u64 {
    let mut key_split = hex.trim().split_whitespace();
    let key: Option<&str> = key_split.next_back();
    let key: &str = match key {
        Some(t) => t,
        None => "0",
    };
    let key = u64::from_str_radix(key, 16);
    match key {
        Ok(t) => t,
        Err(_) => 0,
    }
}

fn get_uevent_device(file_data: String) -> Option<String> {
    let dev: String = file_data
        .lines()
        .filter_map(|x| {
            if x.starts_with(DEVNAME) {
                match x.rsplit('=').next() {
                    Some(dv) => Some(dv),
                    None => None,
                }
            } else {
                return None;
            }
        })
        .collect();
    Some(dev)
}

pub fn mask_build(bits: &[u8]) -> u64 {
    bits.iter().fold(0, |acc, bit| acc + (1 << bit))
}
pub fn bits_set(val: u64, mask: u64) -> bool {
    (val & mask) == mask
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bits_set() {
        use crate::bits_set;
        let val = 3;
        let mask = 3;
        assert_eq!(true, bits_set(val, mask));
        let val = 3;
        let mask = 2;
        assert_eq!(true, bits_set(val, mask));
        let val = 3;
        let mask = 4;
        assert_eq!(false, bits_set(val, mask));
        let val = 2;
        let mask = 1;
        assert_eq!(false, bits_set(val, mask));
        let val = 5;
        let mask = 2;
        assert_eq!(false, bits_set(val, mask));
    }
    #[test]
    fn test_ev_cap() {
        let hex = String::from("ff");
        let num = u8::from_str_radix(&hex, 16);
        assert_eq!(255, num.unwrap());
        use crate::keyboard_ev_cap_test;
        assert!(keyboard_ev_cap_test("3"));
        assert!(keyboard_ev_cap_test("3\n"));
        assert!(!keyboard_ev_cap_test("2\n"));
        assert!(!keyboard_ev_cap_test("2\n"));
        assert!(!keyboard_ev_cap_test("1\n"));
        assert!(!keyboard_ev_cap_test("8\n"));
    }
    #[test]
    fn test_key_cap() {
        use crate::keyboard_key_cap_test;
        assert!(keyboard_key_cap_test("ffffffffffffffff"));
        assert!(keyboard_key_cap_test("fffffffffffffffe"));
        assert!(!keyboard_key_cap_test("fffffffffffffffd"));
    }
    #[test]
    fn test_uevent() {
        use crate::get_uevent_device;
        let udev_file_contents = String::from("MAJOR=13\nMINOR=64\nDEVNAME=input/event0");
        assert_eq!(
            Some(String::from("input/event0")),
            get_uevent_device(udev_file_contents)
        );
    }
    #[test]
    fn mask_build_test() {
        use crate::mask_build;
        assert_eq!(1, 1 << 0);
        assert_eq!(2, 1 << 1);
        assert_eq!(4, 1 << 2);
        let mask = [0, 1, 2, 3];
        assert_eq!(15u64, mask_build(&mask));
        let mask = [0, 1];
        assert_eq!(3u64, mask_build(&mask));
        let mask = [1];
        assert_eq!(2u64, mask_build(&mask));
    }
}
