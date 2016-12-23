use std::time::Duration;
use std::thread::sleep;
use std::usize;

use sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Rect, Point};
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;

use yuv::{buf_size, buf_size_pad, Yuv, YuvReader};

const ZOOMED: u32 = 256;

#[derive(Copy, Clone)]
pub enum Channel {
    YUV,
    Y,
    U,
    V,
}

#[derive(Copy, Clone)]
enum ViewFrame {
    FrameA,
    FrameB,
    Diff,
}

enum UserInput {
    Quit,
    Next,
    Prev,
    FirstFrame,
    GridToggle,
    ShowChannel(Channel),
    ShowFrame(ViewFrame),
    Click(i32, i32),
}

struct Button {
    rect: Rect,
}

type ButtonHandler = Fn(&mut UiSettings, i32, i32);

impl Button {
    fn new(r: Rect) -> Button {
        Button { rect: r }
    }

    fn x(&self) -> i32 {
        self.rect.left()
    }

    fn y(&self) -> i32 {
        self.rect.top()
    }

    fn inside(&self, x: i32, y: i32) -> bool {
        let l = self.rect.left();
        let r = self.rect.right();
        let t = self.rect.top();
        let b = self.rect.bottom();

        x < r && x > l && y < b && y > t
    }
}

struct UiSettings {
    show_grid: bool,
    channel: Channel,
    viewed: ViewFrame,
    mblock_size: usize,
    diff_multiplier: u32,
    mblock_x: usize,
    mblock_y: usize,
}

impl UiSettings {
    fn defaults() -> UiSettings {
        UiSettings {
            show_grid: false,
            channel: Channel::Y,
            viewed: ViewFrame::FrameA,
            mblock_size: 16,
            diff_multiplier: 5,
            mblock_x: 0,
            mblock_y: 0,
        }
    }

    fn show_channel(&mut self, c: Channel) {
        self.channel = c;
    }

    fn show_frame(&mut self, f: ViewFrame) {
        self.viewed = f;
    }

    fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    fn mblock_set(&mut self, x: i32, y: i32) {
        self.mblock_x = (x as usize / self.mblock_size) * self.mblock_size;
        self.mblock_y = (y as usize / self.mblock_size) * self.mblock_size;
    }
}

pub struct SdlUi<'a> {
    width: u32,
    height: u32,
    event_pump: sdl2::EventPump,
    renderer: sdl2::render::Renderer<'a>,
    reader_a: YuvReader,
    reader_b: YuvReader,
    diff: Yuv,
    settings: UiSettings,
    buttons: Vec<(Button, Box<ButtonHandler>)>,
    empty_uv: Box<[u8]>,
}

impl<'a> SdlUi<'a> {
    pub fn new(width: u32, height: u32, file_a: &str, file_b: &str) -> Result<SdlUi<'a>, String> {
        let width_s: usize = width as usize;
        let height_s: usize = height as usize;
        let sdl_context = sdl2::init().map_err(|e| format!("Can't init SDL: {}", e))?;
        let video_subsystem = sdl_context.video()
            .map_err(|e| format!("Can't init SDL Video: {}", e))?;

        let w = width + ZOOMED;
        let h = if ZOOMED > height { ZOOMED } else { height };

        let window = video_subsystem.window("yuvdiff", w, h)
            .position_centered()
            .build()
            .map_err(|e| format!("Can't init SDL Window: {}", e))?;

        let renderer =
            window.renderer().build().map_err(|e| format!("Can't init SDL Renderer: {}", e))?;

        let reader_a = YuvReader::new(width_s, height_s, file_a)
            .map_err(|e| format!("Can't open '{}': {}", file_a, e))?;
        let reader_b = YuvReader::new(width_s, height_s, file_b)
            .map_err(|e| format!("Can't open '{}': {}", file_b, e))?;
        let event_pump = sdl_context.event_pump()
            .map_err(|e| format!("Can't init SDL event pump: {}", e))?;
        let uv_size = buf_size_pad(width_s / 2, height_s / 2);

        Ok(SdlUi {
            width: width,
            height: height,
            event_pump: event_pump,
            renderer: renderer,
            reader_a: reader_a,
            reader_b: reader_b,
            diff: Yuv::new(width_s, height_s),
            settings: UiSettings::defaults(),
            buttons: SdlUi::button_cfg(width, height),
            empty_uv: vec![128; uv_size].into_boxed_slice(),
        })
    }

    fn button_cfg(w: u32, h: u32) -> Vec<(Button, Box<ButtonHandler>)> {
        let buttons: Vec<(Button, Box<ButtonHandler>)> = vec![(Button::new(Rect::new(0, 0, w, h)),
                                                               Box::new(UiSettings::mblock_set))];
        buttons
    }

    pub fn set_channel(&mut self, text: &str) -> Result<(), &'static str> {
        match text {
            "C" | "c" => {
                self.settings.show_channel(Channel::YUV);
                Ok(())
            }
            "Y" | "y" => {
                self.settings.show_channel(Channel::Y);
                Ok(())
            }
            "U" | "u" => {
                self.settings.show_channel(Channel::U);
                Ok(())
            }
            "V" | "v" => {
                self.settings.show_channel(Channel::V);
                Ok(())
            }
            _ => Err("Invalid channel argument"),
        }
    }

    pub fn set_view(&mut self, text: &str) -> Result<(), &'static str> {
        match text {
            "A" | "a" => {
                self.settings.show_frame(ViewFrame::FrameA);
                Ok(())
            }
            "B" | "b" => {
                self.settings.show_frame(ViewFrame::FrameB);
                Ok(())
            }
            "D" | "d" => {
                self.settings.show_frame(ViewFrame::Diff);
                Ok(())
            }
            _ => Err("Incorrect view argument"),
        }
    }

    pub fn set_diff_multiplier(&mut self, m: u32) {
        self.settings.diff_multiplier = m;
    }

    pub fn run(&mut self) {
        // read the first frames
        if self.reader_a.has_next() && self.reader_b.has_next() {
            self.reader_a.next_frame().unwrap();
            self.reader_b.next_frame().unwrap();
        }
        let delay = Duration::from_millis(1000 / 60);

        loop {
            let inputs = self.fetch_inputs();
            let should_quit = self.process_inputs(inputs);
            if should_quit {
                break;
            }

            self.display();
            sleep(delay);
        }
    }

    fn uv_len(&self) -> usize {
        buf_size(self.width as usize / 2, self.height as usize / 2)
    }

    fn uv_len_half(&self) -> usize {
        self.uv_len() / 4
    }

    fn draw_grid(&mut self, size: i32) {
        let w: i32 = self.width as i32;
        let h: i32 = self.height as i32;

        self.renderer.set_draw_color(sdl2::pixels::Color::RGBA(128, 128, 128, 128));

        for xx in (1..).map(|x| x * size).take_while(|x| *x < w) {
            let a = Point::new(xx, 0);
            let b = Point::new(xx, h - 1);
            self.renderer.draw_line(a, b).unwrap();
        }

        for yy in (1..).map(|x| x * size).take_while(|x| *x < h) {
            let a = Point::new(0, yy);
            let b = Point::new(w - 1, yy);
            self.renderer.draw_line(a, b).unwrap();
        }
    }

    fn display(&mut self) {
        self.renderer.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.renderer.clear();
        {
            let viewed = match self.settings.viewed {
                ViewFrame::FrameA => self.reader_a.cur_frame(),
                ViewFrame::FrameB => self.reader_b.cur_frame(),
                ViewFrame::Diff => &self.diff,
            };

            // Render frame
            {
                let (text_w, text_h) = match self.settings.channel {
                    Channel::YUV | Channel::Y => (viewed.width(), viewed.height()),
                    Channel::U | Channel::V => (viewed.width_uv(), viewed.height_uv()),
                };

                let uv_len = match self.settings.channel {
                    Channel::YUV | Channel::Y => self.uv_len(),
                    Channel::U | Channel::V => self.uv_len_half(),
                };

                let (y, u, v) = match self.settings.channel {
                    Channel::YUV => (viewed.y_frame(), viewed.u_frame(), viewed.v_frame()),
                    Channel::Y => {
                        (viewed.y_frame(), &self.empty_uv[0..uv_len], &self.empty_uv[0..uv_len])
                    }
                    Channel::U => {
                        (viewed.u_frame(), &self.empty_uv[0..uv_len], &self.empty_uv[0..uv_len])
                    }
                    Channel::V => {
                        (viewed.v_frame(), &self.empty_uv[0..uv_len], &self.empty_uv[0..uv_len])
                    }
                };

                let mut texture = self.renderer
                    .create_texture_streaming(PixelFormatEnum::YV12, text_w as u32, text_h as u32)
                    .unwrap();
                texture.update_yuv(None, y, text_w, u, text_w / 2, v, text_w / 2).unwrap();

                self.renderer
                    .copy(&texture,
                          None,
                          Some(Rect::new(0, 0, self.width, self.height)))
                    .unwrap();
            }

            // Render zoomed mblock
            {
                let (x, y, w, h) = match self.settings.channel {
                    Channel::YUV | Channel::Y => {
                        (self.settings.mblock_x,
                         self.settings.mblock_y,
                         self.width as usize,
                         self.settings.mblock_size)
                    }
                    Channel::U | Channel::V => {
                        (self.settings.mblock_x / 2,
                         self.settings.mblock_y / 2,
                         self.width as usize / 2,
                         self.settings.mblock_size / 2)
                    }
                };

                let y_start = x + (y * w);
                let y_end = y_start + (h * w);

                let uv_start = (x / 2) + ((y / 2) * w / 2);
                let uv_end = uv_start + ((h / 2) * w / 2);

                let (zoom_w, zoom_h) = match self.settings.channel {
                    Channel::YUV | Channel::Y => {
                        (self.settings.mblock_size, self.settings.mblock_size)
                    }
                    Channel::U | Channel::V => {
                        (self.settings.mblock_size / 2, self.settings.mblock_size / 2)
                    }
                };

                let (y_pad, u_pad, v_pad) = match self.settings.channel {
                    Channel::YUV => {
                        (viewed.y_frame_pad(), viewed.u_frame_pad(), viewed.v_frame_pad())
                    }
                    Channel::Y => (viewed.y_frame_pad(), &*self.empty_uv, &*self.empty_uv),
                    Channel::U => (viewed.u_frame_pad(), &*self.empty_uv, &*self.empty_uv),
                    Channel::V => (viewed.v_frame_pad(), &*self.empty_uv, &*self.empty_uv),
                };
                let mut zoomed = self.renderer
                    .create_texture_streaming(PixelFormatEnum::YV12, zoom_w as u32, zoom_h as u32)
                    .unwrap();

                zoomed.update_yuv(None,
                                &y_pad[y_start..y_end],
                                w,
                                &u_pad[uv_start..uv_end],
                                w / 2,
                                &v_pad[uv_start..uv_end],
                                w / 2)
                    .unwrap();

                self.renderer
                    .copy(&zoomed,
                          None,
                          Some(Rect::new(self.width as i32, 0, 256, 256)))
                    .unwrap();
            }
        }

        if self.settings.show_grid {
            let s = self.settings.mblock_size;
            self.draw_grid(s as i32);
        }

        self.renderer.present();
    }

    fn fetch_inputs(&mut self) -> Vec<UserInput> {
        let mut inputs = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    inputs.push(UserInput::Quit);
                }
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    inputs.push(UserInput::Next);
                }
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    inputs.push(UserInput::Prev);
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    inputs.push(UserInput::FirstFrame);
                }
                Event::KeyDown { keycode: Some(Keycode::G), .. } => {
                    inputs.push(UserInput::GridToggle);
                }
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    inputs.push(UserInput::ShowChannel(Channel::YUV));
                }
                Event::KeyDown { keycode: Some(Keycode::Y), .. } => {
                    inputs.push(UserInput::ShowChannel(Channel::Y));
                }
                Event::KeyDown { keycode: Some(Keycode::U), .. } => {
                    inputs.push(UserInput::ShowChannel(Channel::U));
                }
                Event::KeyDown { keycode: Some(Keycode::V), .. } => {
                    inputs.push(UserInput::ShowChannel(Channel::V));
                }
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    inputs.push(UserInput::ShowFrame(ViewFrame::FrameA));
                }
                Event::KeyDown { keycode: Some(Keycode::B), .. } => {
                    inputs.push(UserInput::ShowFrame(ViewFrame::FrameB));
                }
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    inputs.push(UserInput::ShowFrame(ViewFrame::Diff));
                }
                Event::MouseButtonUp { mouse_btn: Some(MouseButton::Left), x, y, .. } => {
                    inputs.push(UserInput::Click(x, y));
                }
                _ => (),
            }
        }
        inputs
    }

    fn check_clicks(&mut self, x: i32, y: i32) {
        for &(ref but, ref act) in &self.buttons {
            if but.inside(x, y) {
                act(&mut self.settings, x - but.x(), y - but.y());
            }
        }
    }

    fn gen_diff(&mut self) {
        let a = self.reader_a.cur_frame();
        let b = self.reader_b.cur_frame();
        let mult = self.settings.diff_multiplier;
        self.diff = Yuv::from_abs_diff(a, b).unwrap().multiplied(mult);
    }

    fn process_inputs(&mut self, inputs: Vec<UserInput>) -> bool {
        let mut should_quit = false;
        for event in inputs {
            match event {
                UserInput::Next => {
                    if self.reader_a.has_next() && self.reader_b.has_next() {
                        self.reader_a.next_frame().unwrap();
                        self.reader_b.next_frame().unwrap();
                        self.gen_diff();
                    }
                }
                UserInput::Prev => {
                    // Assume this only fails at 1st frame, ignore err for now
                    self.reader_a.prev_frame().unwrap_or(());
                    self.reader_b.prev_frame().unwrap_or(());
                    self.gen_diff();
                }
                UserInput::FirstFrame => {
                    self.reader_a.reset().unwrap();
                    self.reader_b.reset().unwrap();
                    self.gen_diff();
                }
                UserInput::Quit => should_quit = true,
                UserInput::ShowChannel(c) => self.settings.show_channel(c),
                UserInput::ShowFrame(f) => self.settings.show_frame(f),
                UserInput::GridToggle => self.settings.toggle_grid(),
                UserInput::Click(x, y) => self.check_clicks(x, y),
            }
        }

        should_quit
    }
}
