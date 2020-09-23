mod cpu;

// use std::fs::File;
// use std::io::BufReader;

use std::env;
use std::fs;
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::Color,
    log::Level,
    input::Key,
    run, Graphics, Input, Result, Settings, Timer, Window,
};
use rodio::Sink;

const PIXEL_SIZE: f32 = 10.;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const CPU_TICKS_PER_FRAME: i32 = 10;
const TPS: f32 = 60.;


fn main() {
    let log_level = match env::var("RUST_LOG") {
        Ok(var) => match var.as_str() {
            "debug" => Level::Debug,
            "info" => Level::Info,
            "warn" => Level::Warn,
            _ => Level::Error,
        },
        Err(_) => Level::Error,
    };

    run(
        Settings {
            title: "Chip8",
            size: Vector::new(PIXEL_SIZE * WIDTH as f32, PIXEL_SIZE * HEIGHT as f32),
            log_level: log_level,
            use_static_dir: false,
            ..Settings::default()
        },
        app,
    );
}

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    let dark = Color::from_rgba(40, 40, 40, 1.);
    gfx.clear(dark);

    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);
    let source = rodio::source::SineWave::new(440);
    // let file = File::open("test.ogg").unwrap();
    // let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    sink.append(source);
    sink.pause();

    let mut update_timer = Timer::time_per_second(TPS);

    let mut rects = Vec::new();
    for i in 0..WIDTH {
        let mut col = Vec::new();
        for j in 0..HEIGHT {
            col.push(Rectangle::new(
                Vector::new(PIXEL_SIZE * i as f32, PIXEL_SIZE * j as f32),
                Vector::new(PIXEL_SIZE * (i+1) as f32, PIXEL_SIZE * (j+1) as f32)));
        }
        rects.push(col);
    }
    let mut cpu = cpu::CPU::new();

    let args: Vec<String> = env::args().collect();
    let program = fs::read(&args[1]);
    for (index, byte) in program.unwrap().iter().enumerate() {
        cpu.memory[index + 0x200] = *byte;
    }
    cpu.pc = 0x200;

    loop {
        while let Some(_) = input.next_event().await {}

        cpu.keyboard[0x1] = input.key_down(Key::A);
        cpu.keyboard[0x2] = input.key_down(Key::Z);
        cpu.keyboard[0x3] = input.key_down(Key::E);
        cpu.keyboard[0xC] = input.key_down(Key::R);

        cpu.keyboard[0x4] = input.key_down(Key::Q);
        cpu.keyboard[0x5] = input.key_down(Key::S);
        cpu.keyboard[0x6] = input.key_down(Key::D);
        cpu.keyboard[0xD] = input.key_down(Key::F);

        cpu.keyboard[0x7] = input.key_down(Key::U);
        cpu.keyboard[0x8] = input.key_down(Key::I);
        cpu.keyboard[0x9] = input.key_down(Key::O);
        cpu.keyboard[0xE] = input.key_down(Key::P);

        cpu.keyboard[0xA] = input.key_down(Key::J);
        cpu.keyboard[0x0] = input.key_down(Key::K);
        cpu.keyboard[0xB] = input.key_down(Key::L);
        cpu.keyboard[0xF] = input.key_down(Key::M);

        if input.key_down(Key::Escape) {
            return Ok(());
        }

        while update_timer.tick() {
            for _ in 0..CPU_TICKS_PER_FRAME {
                cpu.run();
            }
            rects = display(&mut gfx, rects, &cpu.screen);

            cpu.update_timers();

            if cpu.st != 0 {
                sink.play();
            } else {
                sink.pause();
            }
        }

        gfx.present(&window)?;
    }
}

fn display(gfx: &mut Graphics,
           rects: Vec<Vec<Rectangle>>,
           pixels: &[bool; 2048]) -> Vec<Vec<Rectangle>> {
    let dark = Color::from_rgba(40, 40, 40, 1.);
    let clear = Color::from_rgba(200, 200, 200, 1.);
    let mut color: Color;

    for (index, pixel) in pixels.iter().enumerate() {
        let row = index / 64;
        let col = index % 64;

        color = if *pixel { clear } else { dark };
        gfx.fill_rect(&rects[col][row], color);
    }
    rects
}
