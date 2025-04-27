#![allow(unused_parens)]
use raylib::prelude::*;

use squire::instructions::Error;
use squire::error;

pub struct RAYHandler
{

}
impl RaylibDraw for RAYHandler {}

pub struct RAY
{
    is_drawing: bool,
    handler: RAYHandler,
    rl: Option<RaylibHandle>,
    thread: Option<RaylibThread>,
}
impl Default for RAY { fn default() -> Self { Self::new() }}
impl RAY
{

    pub fn new() -> Self
    {
        Self
        {
            handler: RAYHandler {},
            is_drawing: false,
            rl:    None,
            thread: None,
        }
    }

    fn check_active(&self) -> Result<(), Error>
    {
        if(self.rl.is_none() || self.thread.is_none())
        {
            Err(error!("Raylib is not active!"))
        }
        else
        {
            Ok(())
        }
    }
    fn check_drawing(&self) -> Result<(), Error>
    {
        if(self.is_drawing)
        {
            Ok(())
        }
        else
        {
            Err(error!("Attempting to draw without draw handle!"))
        }
    }

    pub fn OpenWindow(&mut self, w: u32, h: u32, title: String, resizable: bool) -> Result<(), Error>
    {
        if(self.rl.is_some() || self.thread.is_some())
        {
            return Err(error!("Cannot initialize raylib while it is active!"));
        }
        if(resizable)
        {
            let (rl, thread) = raylib::init()
                .title(title.as_str())
                .size(w as i32, h as i32)
                .resizable()
                .build();
            self.rl     = Some(  rl  );
            self.thread = Some(thread);
        }
        else
        {
            let (rl, thread) = raylib::init()
                .title(title.as_str())
                .size(w as i32, h as i32)
                .build();
            self.rl     = Some(  rl  );
            self.thread = Some(thread);
        }
        Ok(())
    }
    pub fn CloseWindow(&mut self) -> Result<(), Error>
    {
        self.check_active()?;
        self.rl = None;
        self.thread = None;
        Ok(())
    }

    pub fn BeginDrawing(&mut self) -> Result<(), Error>
    {
        if(self.is_drawing)
        {
            return Err(error!("Attempting to begin drawing while still owning draw handle!"));
        }
        unsafe { raylib::ffi::BeginDrawing(); }
        self.is_drawing = true;
        Ok(())
    }
    pub fn EndDrawing(&mut self) -> Result<(), Error>
    {
        if(!self.is_drawing)
        {
            return Err(error!("Cannot end drawing without draw handle!"));
        }
        unsafe { raylib::ffi::EndDrawing(); }
        self.is_drawing = false;
        Ok(())
    }

    pub fn ClearBackground(&mut self, color: Color) -> Result<(), Error>
    {
        self.check_drawing()?;
        self.handler.clear_background(color);
        Ok(())
    }
    pub fn DrawRectange(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) -> Result<(), Error>
    {
        self.check_drawing()?;
        self.handler.draw_rectangle(x as i32, y as i32, w as i32, h as i32, color);
        Ok(())
    }
    pub fn DrawFPS(&mut self, x: u32, y: u32) -> Result<(), Error>
    {
        self.check_drawing()?;
        self.handler.draw_fps(x as i32, y as i32);
        Ok(())
    }
    pub fn DrawText(&mut self, x: u32, y: u32, font_size: u32, color: Color, text: String) -> Result<(), Error>
    {
        self.check_drawing()?;
        self.handler.draw_text(text.as_str(), x as i32, y as i32, font_size as i32, color);
        Ok(())
    }

    pub fn SetTargetFPS(&mut self, fps: u32) -> Result<(), Error>
    {
        self.check_active()?;
        self.rl.as_mut().unwrap().set_target_fps(fps);
        Ok(())
    }
    pub fn WindowShouldClose(&mut self) -> Result<u8, Error>
    {
        self.check_active()?;
        if(self.rl.as_mut().unwrap().window_should_close())
        {
            Ok(1)
        }
        else
        {
            Ok(0)
        }
    }

    pub fn IsWindowResized(&self) -> Result<u8, Error>
    {
        self.check_active()?;
        if(self.rl.as_ref().unwrap().is_window_resized())
        {
            Ok(1)
        }
        else
        {
            Ok(0)
        }
    }
    pub fn GetWindowWidth(&self) -> Result<u32, Error>
    {
        self.check_active()?;
        Ok(self.rl.as_ref().unwrap().get_render_width() as u32)
    }
    pub fn GetWindowHeight(&self) -> Result<u32, Error>
    {
        self.check_active()?;
        Ok(self.rl.as_ref().unwrap().get_render_height() as u32)
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color { Color { r, g, b, a } }

}
