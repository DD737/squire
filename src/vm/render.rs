#![allow(clippy::await_holding_lock)]

use std::{collections::VecDeque, f32};

use macroquad::{color::{Color, GREEN}, conf::Conf, shapes::draw_rectangle, text::draw_text, time::get_fps, window::{clear_background, next_frame}};
use squire::instructions::Error;

enum RenderDispatch
{
    AwaitFrame,
    ClearBackground(Color),
    DrawRectangle(u32, u32 ,u32, u32, Color), // x y w h col
    DrawText(String, u32, u32, u32, Color), // text x y size col
    DrawFPS(u32, u32), // x y
    CloseWindow,
}

struct RenderDispatcher
{
    queue: VecDeque<RenderDispatch>,
    errors: Vec<Error>,
}
impl RenderDispatcher
{
    pub fn new() -> Self
    {
        RenderDispatcher
        {
            queue: VecDeque::new(),
            errors: Vec::new(),
        }
    }
    pub fn get(&mut self) -> Option<RenderDispatch>
    {
        todo!()
    }
    pub fn dispatch(&mut self, cmd: RenderDispatch)
    {
        todo!()
    }
    pub fn errors(&mut self) -> Vec<Error>
    {
        std::mem::take(&mut self.errors)
    }
    pub fn push_error(&mut self, e: Error)
    {
        self.errors.push(e);
    }
} 

use std::sync::{LazyLock, Mutex};
static DISPATCHER: LazyLock<Mutex<RenderDispatcher>> = LazyLock::new(|| Mutex::new(RenderDispatcher::new()));

pub struct Render
{
    active: bool,
}
impl Render
{

    async fn RenderLoop() -> Result<(), Error>
    {
        loop
        {
            while let Some(d) = DISPATCHER.lock().unwrap().get()
            {
                match d
                {
                    RenderDispatch::CloseWindow => return Ok(()),
                    RenderDispatch::AwaitFrame => next_frame().await,
                    RenderDispatch::ClearBackground(col) => clear_background(col),
                    RenderDispatch::DrawRectangle(x, y, w, h, col) => draw_rectangle(x as f32, y as f32, w as f32, h as f32, col),
                    RenderDispatch::DrawText(msg, x, y, size, col) => { draw_text(&msg, x as f32, y as f32, size as f32, col); },
                    RenderDispatch::DrawFPS(x, y) => { draw_text(format!("{}", get_fps()).as_str(), x as f32, y as f32, 20.0, GREEN); }
                }
            }
        }
    }

    pub fn OpenWindow(&mut self)
    {
        macroquad::Window::from_config(Conf { ..Default::default() }, async 
            {
                if let Err(e) = Render::RenderLoop().await 
                {
                    DISPATCHER.lock().unwrap().push_error(e);
                }
            });
        self.active = true;
    }

    pub fn IsActive(&mut self)
    {
    }

}
