#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate rand;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use graphics::*;
use std::fmt;
use rand::Rng;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const SCREEN_WIDTH: f64 = 800.0;
const SCREEN_HEIGHT: f64 = 600.0;
const PIXEL_SIZE: f64 = 7.0;
const PIXELS_WIDTH: usize = (SCREEN_WIDTH/PIXEL_SIZE) as usize;
const PIXELS_HEIGHT: usize = (SCREEN_HEIGHT/PIXEL_SIZE) as usize;

fn random(min: i32, max: i32) -> i32{
	let mut rng = rand::thread_rng();
	rng.gen_range(min, max+1) //max should be included
}

struct Coord{
    x: i32,
    y: i32,
}

impl Coord{
    pub fn new(x: i32, y: i32) -> Coord{
        return Coord{
            x: x,
            y: y,
        };
    }
}

struct Pixel{
    x: f64,
    y: f64,
    alive: bool,
}

impl Pixel{
    pub fn new(x: f64, y: f64, alive: bool) -> Pixel{
        let p = Pixel{
            x: x,
            y: y,
            alive: alive,
        };
        return p;
    }

    pub fn draw(&self, gl: &mut GlGraphics, c: Context){
        let mut color: [f32; 4] = BLACK;
        if self.alive {
            color = WHITE;
        }
        rectangle(color, [0.0, 0.0, PIXEL_SIZE, PIXEL_SIZE], c.transform.trans(self.x, self.y), gl);
    }
}

pub struct App {
    gl: GlGraphics,    
    can_click: bool,
    pixels: [[Pixel; PIXELS_WIDTH]; PIXELS_HEIGHT],
}

impl App {
    fn new(opengl: OpenGL) -> App{
        let app: App = App{
            gl: GlGraphics::new(opengl),
            can_click: true,
            pixels: App::init_pixels_organic(),
        };
        return app;
    }

    fn init_pixels() -> [[Pixel; PIXELS_WIDTH]; PIXELS_HEIGHT]{
        let mut pixels:  [[Pixel; PIXELS_WIDTH]; PIXELS_HEIGHT] = unsafe {::std::mem::uninitialized()};
        for x in 0..PIXELS_WIDTH{
            for y in 0..PIXELS_HEIGHT{
                pixels[y][x] = Pixel::new(x as f64 * PIXEL_SIZE, y as f64 * PIXEL_SIZE, false);
            }
        }
        return pixels;
    }

    fn init_pixels_organic() -> [[Pixel; PIXELS_WIDTH]; PIXELS_HEIGHT]{
        println!("unsafe");
        let mut pixels:  [[Pixel; PIXELS_WIDTH]; PIXELS_HEIGHT] = unsafe {::std::mem::uninitialized()};
        println!("after unsafe");
        for x in 0..PIXELS_WIDTH{
            for y in 0..PIXELS_HEIGHT{
                let mut alive: bool = false;
                if random(0, 100) > 66{
                    alive = true;
                }
                pixels[y][x] = Pixel::new(x as f64 * PIXEL_SIZE, y as f64 * PIXEL_SIZE, alive);
            }
        }
        return pixels;
    }

    fn render(&mut self, args: &RenderArgs) {
        let pixels = &self.pixels;
		self.gl.draw(args.viewport(), |c, gl| {
            for x in 0..PIXELS_WIDTH{
                for y in 0..PIXELS_HEIGHT{
                    pixels[y][x].draw(gl, c);
                }
            }
		});
	}

    fn mouse_left_clicked(&mut self, mouse_x: f64, mouse_y: f64){
        if !self.can_click {
            return;
        }
        self.can_click = false;

        for x in 0..PIXELS_WIDTH{
            for y in 0..PIXELS_HEIGHT{
                let mut pixel = &mut self.pixels[y][x];
                if pixel.x < mouse_x && pixel.x + PIXEL_SIZE > mouse_x && pixel.y < mouse_y && pixel.y + PIXEL_SIZE > mouse_y{
                    pixel.alive = !pixel.alive;
                    return;
                }
            }
        }
    }

    fn mouse_left_released(&mut self, x: f64, y: f64){
        self.can_click = true;
    }

    fn are_valid_pixel_coords(&self, x: i32, y: i32) -> bool{
        return x > 0 && x < PIXELS_WIDTH as i32 && y > 0 && y < PIXELS_HEIGHT as i32;
    }

    fn pixel_alive(&self, x: usize, y: usize) -> bool{
        return self.pixels[y][x].alive;
    }

    fn count_alive_surrounding_pixels(&self, x: usize, y: usize) -> i32{
        let mut count: i32 = 0;

        //Get all possible coords
        let mut coords: Vec<Coord> = Vec::new();
        let x_i32: i32 = x as i32;
        let y_i32: i32 = y as i32;
        coords.push(Coord::new(x_i32-1  , y_i32     ));
        coords.push(Coord::new(x_i32+1  , y_i32     ));
        coords.push(Coord::new(x_i32-1  , y_i32+1   ));
        coords.push(Coord::new(x_i32+1  , y_i32+1   ));
        coords.push(Coord::new(x_i32-1  , y_i32-1   ));
        coords.push(Coord::new(x_i32+1  , y_i32-1   ));
        coords.push(Coord::new(x_i32    , y_i32+1   ));
        coords.push(Coord::new(x_i32    , y_i32-1   ));
        
        for coord in &coords{
            if self.are_valid_pixel_coords(coord.x, coord.y){
                if self.pixel_alive(coord.x as usize, coord.y as usize){
                    count += 1;
                }
                //else{
                //    count -= 1;
                //}
            }
        }
        return count;
    }

    fn update_pixels(&mut self){
        let mut new_pixels: [[Pixel; PIXELS_WIDTH]; PIXELS_HEIGHT] = App::init_pixels();

        for x in 0..PIXELS_WIDTH{
            for y in 0..PIXELS_HEIGHT{
                let pixel = &self.pixels[y][x];
                
                if pixel.alive {
                    //1. Any live cell with fewer than two live neighbors dies, as if by underpopulation.        
                    if self.count_alive_surrounding_pixels(x, y) < 2{
                        new_pixels[y][x].alive = false;
                        continue;
                    }

                    //2. Any live cell with two or three live neighbors lives on to the next generation.
                    if self.count_alive_surrounding_pixels(x, y) == 2 || self.count_alive_surrounding_pixels(x, y) == 3{
                        new_pixels[y][x].alive = true;
                        continue;
                    }

                    //3. Any live cell with more than three live neighbors dies, as if by overpopulation.
                    if self.count_alive_surrounding_pixels(x, y) > 3{
                        new_pixels[y][x].alive = false;
                        continue;
                    }
                }
                
                //4 .Any dead cell with exactly three live neighbors becomes a live cell, as if by reproduction.
                else{
                    if self.count_alive_surrounding_pixels(x, y) == 3{
                        new_pixels[y][x].alive = true;
                    }
                }
            }
        }
        self.pixels = new_pixels;
    }
}

fn main() {
    
    let opengl = OpenGL::V4_5;

    let mut window: Window = WindowSettings::new(
            "rusty game of life",
            [SCREEN_WIDTH, SCREEN_HEIGHT]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.\
    let mut app = App::new(opengl);
    let mut events = Events::new(EventSettings::new());
    
	let mut mouse_x: f64 = 0.0;
	let mut mouse_y: f64 = 0.0;
    let mut running: bool = false;
    let mut can_right_click: bool = true;

	while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            if running{
                app.update_pixels();
            }
        }
		
		
		e.mouse_cursor(|x, y| {
			mouse_x = x;
			mouse_y = y;
			//app.board.mouse_move(mouse_x, mouse_y);
		});

		if let Some(Button::Mouse(button)) = e.release_args() {
			if button == MouseButton::Left {
                app.mouse_left_released(mouse_x, mouse_y);
			}
			if button == MouseButton::Right {
                can_right_click = true;
            }
		}
		
		if let Some(Button::Mouse(button)) = e.press_args() {
            if button == MouseButton::Left {
				app.mouse_left_clicked(mouse_x, mouse_y);
			}
			if button == MouseButton::Right {
                if can_right_click{
                    can_right_click = false;
                    running = !running;
                    println!("running {}",running)
                }
			}
		}
		//End mouse input
    }
}