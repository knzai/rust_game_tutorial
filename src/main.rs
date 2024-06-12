mod components;
mod physics;
mod animator;
mod keyboard;
mod renderer;
mod ai;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::image::{self, LoadTexture, InitFlag};

use specs::prelude::*;

use std::time::Duration;

use crate::components::*;

pub enum MovementCommand {
	Stop,
	Move(Direction),
}

fn direction_spritesheet_row(direction:Direction) -> i32 {
	use self::Direction::*;
	match direction {
		Up => 3,
		Down => 0,
		Left => 1,
		Right =>2,
	}
}

/// Create animation frames for the standard character spritesheet
fn character_animation_frames(spritesheet: usize, top_left_frame: Rect, direction: Direction) -> Vec<Sprite> {
	// All assumptions about the spritesheets are now encapsulated in this function instead of in
	// the design of our entire system. We can always replace this function, but replacing the
	// entire system is harder.

	let (frame_width, frame_height) = top_left_frame.size();
	let y_offset = top_left_frame.y() + frame_height as i32 * direction_spritesheet_row(direction);

	let mut frames = Vec::new();
	for i in 0..3 {
		frames.push(Sprite {
			spritesheet,
			region: Rect::new(
				top_left_frame.x() + frame_width as i32 * i,
				y_offset,
				frame_width,
				frame_height,
			),
		})
	}

	frames
}

fn initialize_player(world: &mut World, player_spritesheet: usize) {
    let player_top_left_frame = Rect::new(0, 0, 26, 36);

    let player_animation = MovementAnimation {
        current_frame: 0,
        up_frames: character_animation_frames(player_spritesheet, player_top_left_frame, Direction::Up),
        down_frames: character_animation_frames(player_spritesheet, player_top_left_frame, Direction::Down),
        left_frames: character_animation_frames(player_spritesheet, player_top_left_frame, Direction::Left),
        right_frames: character_animation_frames(player_spritesheet, player_top_left_frame, Direction::Right),
    };

    world.create_entity()
      .with(KeyboardControlled)
      .with(Position(Point::new(0, 0)))
      .with(Velocity {speed: 0, direction: Direction::Right})
      .with(player_animation.right_frames[0].clone())
      .with(player_animation)
      .build();
}

fn initialize_enemy(world: &mut World, enemy_spritesheet: usize, position: Point) {
	let enemy_top_left_frame = Rect::new(0, 0, 32, 36);

	let enemy_animation = MovementAnimation {
		current_frame: 0,
		up_frames: character_animation_frames(enemy_spritesheet, enemy_top_left_frame, Direction::Up),
		down_frames: character_animation_frames(enemy_spritesheet, enemy_top_left_frame, Direction::Down),
		left_frames: character_animation_frames(enemy_spritesheet, enemy_top_left_frame, Direction::Left),
		right_frames: character_animation_frames(enemy_spritesheet, enemy_top_left_frame, Direction::Right),
	};

	world.create_entity()
	  .with(Enemy)
		.with(Position(position))
		.with(Velocity {speed: 0, direction: Direction::Right})
		.with(enemy_animation.right_frames[0].clone())
		.with(enemy_animation)
		.build();
}


fn main() -> Result<(), String> {
	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;
	let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

	let window = video_subsystem.window("game tutorial", 800, 600)
		.position_centered()
		.build()
		.expect("could not initialize video subsystem");

	let mut canvas = window.into_canvas().build()
		.expect("could not make a canvas");

	let texture_creator = canvas.texture_creator();
	
	let mut dispatcher = DispatcherBuilder::new()
	  .with(keyboard::Keyboard, "Keyboard", &[])
    .with(ai::AI, "AI", &[])
    .with(physics::Physics, "Physics", &["Keyboard", "AI"])
    .with(animator::Animator, "Animator", &["Keyboard", "AI"])
		.build();

	let mut world = World::new();
	dispatcher.setup(&mut world.res);
	renderer::SystemData::setup(&mut world.res);
	
  // Initialize resource
  let movement_command: Option<MovementCommand> = None;
  world.add_resource(movement_command);
	
	let textures = [
		texture_creator.load_texture("assets/bardo.png")?,
		texture_creator.load_texture("assets/reaper.png")?,
	];
	// First texture in textures array
	let player_spritesheet = 0;
	let enemy_spritesheet = 1;
	
	initialize_player(&mut world, player_spritesheet);

  initialize_enemy(&mut world, enemy_spritesheet, Point::new(-150, -150));
  initialize_enemy(&mut world, enemy_spritesheet, Point::new(150, -190));
  initialize_enemy(&mut world, enemy_spritesheet, Point::new(-150, 170));

	let mut event_pump = sdl_context.event_pump()?;
	let mut i = 0;
	'running: loop {
		let mut movement_command = None;
		// Handle events
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running;
				},
				Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } => {
					movement_command = Some(MovementCommand::Move(Direction::Left));
				},
				Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => {
					movement_command = Some(MovementCommand::Move(Direction::Right));
				},
				Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. } => {
					movement_command = Some(MovementCommand::Move(Direction::Up));
				},
				Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => {
					movement_command = Some(MovementCommand::Move(Direction::Down));
				},
				Event::KeyUp { keycode: Some(Keycode::Left), repeat: false, .. } |
				Event::KeyUp { keycode: Some(Keycode::Right), repeat: false, .. } |
				Event::KeyUp { keycode: Some(Keycode::Up), repeat: false, .. } |
				Event::KeyUp { keycode: Some(Keycode::Down), repeat: false, .. } => {
					movement_command = Some(MovementCommand::Stop);
				},
				_ => {}
			}
		}
		
		*world.write_resource() = movement_command;
		
		// Update
		i = (i + 1) % 255;
    dispatcher.dispatch(&mut world.res);
    world.maintain();

		// Render
		renderer::render(&mut canvas, Color::RGB(i, 64, 255 - i), &textures, world.system_data())?;

		// Time management
		::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
	}

	Ok(())
}