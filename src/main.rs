use gilrs::{Gilrs, Button, Event};
use std::{thread, time};
use std::str;
use std::time::Duration;
use clap::{Arg, Command};
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Write};
use uuid::Uuid;
use std::error::Error;
use std::io::BufReader;
use std::io::BufRead;
use std::time::Instant;
use rppal::gpio::{Gpio, OutputPin};

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
    DOWN
}
fn do_steps(port: &mut OutputPin, sp: &mut StepPos, steps:i32){
    for _n in 0..steps {
        port.set_high();
        thread::sleep(time::Duration::from_micros(15));
        port.set_low();
    }
}

fn move_stepper(port: &mut OutputPin, dir_pin: &mut OutputPin, sd: StepDir, sp: &mut StepPos, steps:i32, sleep:u64){
    
    let mut byte = String::from("a");
    let mut dirsteps = steps;
    match sd {
        StepDir::DOWN =>{
            dir_pin.set_low();
            dirsteps = dirsteps * -1;
        },
        
        StepDir::UP => {
            dir_pin.set_high();
        }
    }
    sp.pos += dirsteps;
    do_steps(port, sp, steps);

    if(sleep > 0){
        thread::sleep(time::Duration::from_micros(sleep));
    }
    // let sleep = 100;
    // let mut waited = 0;
    // let max_rpm = 200;
    // let steps_per_rev = 200*16;
    // let max_sps = 200*10*4;
    // let min_sleep = 1000000 / max_sps;
    // let start_sleep = 1000000/(steps_per_rev/2);
    // let accel_steps = 4000;
    // let sleep_steps = (start_sleep-min_sleep)*1000/accel_steps;
    // let mut curr_sleep: i128 = start_sleep*1000;
    // println!("start:{} min:{} step:{}", start_sleep*1000, min_sleep*1000, sleep_steps );

    // for n in 1..640000 {
    //     port.set_high();
    //     curr_sleep -= sleep_steps;
    //     curr_sleep = if curr_sleep < min_sleep*1000 { min_sleep*1000  } else { curr_sleep };
    //     // thread::sleep(time::Duration::from_nanos(sleep));
    //     let now = Instant::now();
    //     while now.elapsed().as_micros() < (curr_sleep/2000) as u128 {
    //     }
    //     port.set_low();
    //     while now.elapsed().as_micros() < (curr_sleep/2000) as u128 {
    //     }
    //     // thread::sleep(time::Duration::from_nanos(sleep));
    // }
    // println!("waited: {}", waited);
    // match sd {
    //     StepDir::DOWN => sp.pos -= 1,
    //     StepDir::UP => sp.pos += 1,
    //     StepDir::BIG_DOWN => sp.pos -= 64,
    //     StepDir::BIG_UP => sp.pos += 64
    // }
    
    // println!("{:?}:{:?} {}", st, sd, sp.pos);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stepp_ps: [StepPos; 3] = [StepPos{pos: 0},StepPos{pos: 0},StepPos{pos: 0} ];

    let mut gilrs = Gilrs::new().unwrap();
    

    let mut port_path = String::from("COM4");
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
        port_path = p.port_name;
	break;
    }

    // define GPIO pins
    const DIRECTION_PIN: u8= 22; // Direction (DIR) GPIO Pin
    const STEP_PIN: u8 = 23; // Step GPIO Pin
    const DIRECTION_PIN_Y: u8= 17; // Direction (DIR) GPIO Pin
    const STEP_PIN_Y: u8 = 27; // Step GPIO Pin
    const ENABLE_PIN: u8 = 24; // enable pin (LOW to enable)
    let gpio = Gpio::new()?;
    let mut dir_pin = gpio.get(DIRECTION_PIN)?.into_output();
    let mut step_pin = gpio.get(STEP_PIN)?.into_output();
    let mut enable_pin = gpio.get(ENABLE_PIN)?.into_output();
    let mut dir_pin_y = gpio.get(DIRECTION_PIN_Y)?.into_output();
    let mut step_pin_y = gpio.get(STEP_PIN_Y)?.into_output();

    enable_pin.set_low();
    dir_pin.set_low();
    dir_pin_y.set_low();
    step_pin_y.set_low();
    step_pin.set_low();

    let output = "a".as_bytes();
    thread::sleep(time::Duration::from_millis(100));

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
            let mut has_changes = false;

            if gamepad.is_pressed(Button::DPadLeft) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::DOWN, &mut stepp_ps[0], 1, 100);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::DPadRight) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::UP, &mut stepp_ps[0],1, 100);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::DPadDown) {
                move_stepper(&mut step_pin_y, &mut dir_pin_y, StepDir::DOWN, &mut stepp_ps[1],1, 100);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::DPadUp) {
                move_stepper(&mut step_pin_y, &mut dir_pin_y, StepDir::UP, &mut stepp_ps[1],1, 100);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::Select) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::DOWN, &mut stepp_ps[2],1, 0);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::LeftTrigger2) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::UP, &mut stepp_ps[2],1, 0);
                has_changes = true;
            }

            if gamepad.is_pressed(Button::West) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::DOWN, &mut stepp_ps[0],16, 0);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::East) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::UP, &mut stepp_ps[0],16, 0);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::South) {
                move_stepper(&mut step_pin_y, &mut dir_pin_y, StepDir::DOWN, &mut stepp_ps[1],16, 0);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::North) {
                move_stepper(&mut step_pin_y, &mut dir_pin_y, StepDir::UP, &mut stepp_ps[1],16, 0);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::RightTrigger) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::DOWN, &mut stepp_ps[2],1, 10);
                has_changes = true;
            }
            if gamepad.is_pressed(Button::LeftTrigger) {
                move_stepper(&mut step_pin, &mut dir_pin, StepDir::UP, &mut stepp_ps[2],1, 10);
                has_changes = true;
            }
            if(has_changes){
                println!("X {}, Y {}, F {}", stepp_ps[0].pos,stepp_ps[1].pos,stepp_ps[2].pos);
            }
        }
    }
}