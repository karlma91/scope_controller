use gilrs::{Gilrs, Button, Event};
use std::{thread, time};
use std::str;
use std::time::Duration;
use clap::{Arg, Command};
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Write};
use uuid::Uuid;
use std::io::BufReader;
use std::io::BufRead;
#[derive(Debug, Copy, Clone)]
pub struct StepPos {
    pos: i32,
}
#[derive(Debug)]
enum Steppers{
    XAxis,
    YAxis,
    Focus
}
#[derive(Debug)]
enum StepDir {
    UP,
    DOWN,
    BIG_UP,
    BIG_DOWN
}

fn move_stepper(port: &mut dyn serialport::SerialPort, st: Steppers ,sd: StepDir, sp: &mut StepPos){
    
    let mut byte = String::from("a");
    match st {
        Steppers::YAxis => {
            match sd {
                StepDir::DOWN => byte = "y".to_string(),
                StepDir::UP => byte = "u".to_string(),
                StepDir::BIG_DOWN => byte = "r".to_string(),
                StepDir::BIG_UP=> byte = "t".to_string(),
            }
        },
        Steppers::XAxis => {
            match sd {
                StepDir::DOWN => byte = "b".to_string(),
                StepDir::UP => byte = "a".to_string(),
                StepDir::BIG_DOWN => byte = "r".to_string(),
                StepDir::BIG_UP=> byte = "t".to_string(),
            }
        },
        Steppers::Focus => {
            match sd {
                StepDir::DOWN => byte = "f".to_string(),
                StepDir::UP => byte = "g".to_string(),
                StepDir::BIG_DOWN => byte = "F".to_string(),
                StepDir::BIG_UP=> byte = "G".to_string(),
            }
        }
    }
    println!("Writing: {}", byte);
    let output = byte.as_bytes();
    port.write(output);
    thread::sleep(time::Duration::from_millis(10));

    let mut serial_buf: Vec<u8> = vec![0; 32];
    port.read(serial_buf.as_mut_slice());
    io::stdout().write_all(&serial_buf).unwrap();
    match sd {
        StepDir::DOWN => sp.pos -= 1,
        StepDir::UP => sp.pos += 1,
        StepDir::BIG_DOWN => sp.pos -= 64,
        StepDir::BIG_UP => sp.pos += 64
    }
    
    println!("{:?}:{:?} {}", st, sd, sp.pos);
}

fn main() {
    let mut stepp_ps: [StepPos; 3] = [StepPos{pos: 0},StepPos{pos: 0},StepPos{pos: 0} ];

    let mut gilrs = Gilrs::new().unwrap();
    

    let mut port_path = String::from("COM4");
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
        port_path = p.port_name;
	break;
    }

    let mut port = match serialport::new(port_path, 9600).open() {
        Err(e) => {
            eprintln!("Failed to open. Error: {}", e);
            ::std::process::exit(1);
        }
        Ok(p) => p,
    };

    let output = "a".as_bytes();
    port.write(output).expect("Write failed!");
    thread::sleep(time::Duration::from_millis(100));
    port.flush().unwrap();
   
    println!("Reading from port");
    port.set_timeout(Duration::from_millis(500));
    let mut serial_buf: Vec<u8> = Vec::new();
    match port.read_to_end(&mut serial_buf){
        Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        let guuid = Uuid::from_bytes(gamepad.uuid());
        println!("{} is {:?} {:?} {} ", gamepad.name(), gamepad.power_info(), gamepad.map_name(), guuid.urn());
    }

    let mut active_gamepad = None;

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?}", event);
            active_gamepad = Some(id);
        }
        
        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::DPadLeft) {
                move_stepper(&mut *port, Steppers::YAxis, StepDir::DOWN, &mut stepp_ps[0])
            }
            if gamepad.is_pressed(Button::DPadRight) {
                move_stepper(&mut *port, Steppers::YAxis, StepDir::UP, &mut stepp_ps[0])
            }
            if gamepad.is_pressed(Button::DPadDown) {
                move_stepper(&mut *port, Steppers::XAxis, StepDir::DOWN, &mut stepp_ps[1])
            }
            if gamepad.is_pressed(Button::DPadUp) {
                move_stepper(&mut *port, Steppers::XAxis, StepDir::UP, &mut stepp_ps[1])
            }
            if gamepad.is_pressed(Button::Select) {
                move_stepper(&mut *port, Steppers::Focus, StepDir::DOWN, &mut stepp_ps[2])
            }
            if gamepad.is_pressed(Button::LeftTrigger2) {
                move_stepper(&mut *port, Steppers::Focus, StepDir::UP, &mut stepp_ps[2])
            }
            if gamepad.is_pressed(Button::RightTrigger) {
                move_stepper(&mut *port, Steppers::Focus, StepDir::BIG_DOWN, &mut stepp_ps[2])
            }
            if gamepad.is_pressed(Button::LeftTrigger) {
                move_stepper(&mut *port, Steppers::Focus, StepDir::BIG_UP, &mut stepp_ps[2])
            }
        }
    }
}