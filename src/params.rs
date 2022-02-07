use clap::{App, Arg, ArgGroup};
use std::env;
use std::fs;
static CONFIG_FILE: &str = "/.untouch";
use untouch::Keyboard;

pub struct Params {
    pub do_config: bool,
    pub idle: Option<i32>,
    pub enable: Option<String>,
    pub disable: Option<String>,
}

fn is_integer(int: &str) -> Result<(), String> {
    match i32::from_str_radix(int, 10) {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("An integer is required.")),
    }
}

pub fn get_params() -> Params {
    let cfg = Arg::new("config")
        .short('c')
        .long("config")
        .conflicts_with("enable")
        .help("Config the input device.");

    let req1 = ["config", "enable"];
    let requires = ["idle", "enable", "disable"];

    let idle = Arg::new("idle")
        .validator(is_integer)
        .requires_all(&requires)
        .help("keystroke idletime(ms) before touchpad reactivation");

    let enable = Arg::new("enable")
        .requires_all(&requires)
        .help("touchpad activate command");

    let disable = Arg::new("disable")
        .requires_all(&requires)
        .help("touchpad deactivate command");

    let arg_group = ArgGroup::new("req1")
        .multiple(false)
        .required(true)
        .args(&req1);

    let args = [cfg, idle, enable, disable];
    let arg_matches = App::new("untouch")
        .args(&args)
        .group(arg_group)
        .get_matches();

    Params {
        do_config: arg_matches.is_present("config"),
        idle: if arg_matches.is_present("idle") {
            let int = arg_matches.value_of("idle").unwrap();
            Some(i32::from_str_radix(int, 10).unwrap().abs())
        } else {
            None
        },
        enable: if arg_matches.is_present("enable") {
            Some(String::from(arg_matches.value_of("enable").unwrap()))
        } else {
            None
        },
        disable: if arg_matches.is_present("disable") {
            Some(String::from(arg_matches.value_of("disable").unwrap()))
        } else {
            None
        },
    }
}
use std::io::Write;

pub fn do_config_file(keyboards: &Vec<Keyboard>) {
    let mut selection: usize = 0;
    if keyboards.len() > 1 {
        println!("\nKeyboards:");
        let mut count = 0;
        keyboards.iter().for_each(|kb| {
            println!("{}. {}", count, kb.name);
            count += 1;
        });
        print!("\nSelect keyboard for touchpad override> ");
        std::io::stdout().flush().unwrap();
        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_line(&mut buffer) {
            panic!("Key read failed: {}", e);
        }
        let num = i32::from_str_radix(buffer.trim(), 10);
        selection = match num {
            Ok(num) => {
                if !(num >= 0 && num < keyboards.len() as i32) {
                    eprintln!("Invalid keyboard number");
                    std::process::exit(1);
                } else {
                    num as usize
                }
            }
            Err(_) => {
                eprintln!("Invalid keyboard number");
                std::process::exit(1);
            }
        };
    }

    let save_dir = match env::var("HOME") {
        Ok(x) => x,
        Err(_) => {
            panic!("Could not read $HOME env var.");
        }
    };
    let mut config_file = String::from(save_dir);
    config_file.push_str(CONFIG_FILE);
    if let Err(e) = fs::write(&config_file, &keyboards[selection].name) {
        panic!("Failed config file write: {} {}", config_file, e);
    }
    println!("\nSaving keyboard:'{}'", keyboards[selection].name);
    println!(" to config file:{}", config_file);
}

pub fn get_keyboard_selection(keyboards: &Vec<Keyboard>) -> &Keyboard {
    //Get the keyboard name from config.
    let mut save_dir = String::from(env::var("HOME").unwrap());
    save_dir.push_str(CONFIG_FILE);
    let kb_data: Vec<u8> = fs::read(&save_dir)
      .expect("No config data, use --config option");
    let kb_data = String::from_utf8(kb_data)
      .expect("No config data, use --config option");

    //Find the keyboard in the given list with the configured name.
    let kb: &Keyboard = keyboards.iter()
      .find(|kb| kb.name == kb_data).unwrap();
    kb
}
