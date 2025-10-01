#![feature(slice_from_ptr_range)]/*shader*/#![allow(non_upper_case_globals,non_camel_case_types)]
use ui::{Result, new_trigger, run_with_trigger, size, xy, int2, vector::vec2, Widget, EventContext, Event, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, from_iter, BufferUsage, buffer};
shader!{square}

struct App {
	state: Vec<Vec<int2>>,
}

const screen: int2 = xy{x: 3840, y: 2160};
const square_side: i32 = 48;
const grid: int2 = xy{x: screen.x/square_side/4+1/*=21*/, y: screen.y/square_side/*=45*/};
impl App { fn new() -> Self { Self{state: vec![vec![xy{x: 10, y: 44}]]} } }

impl App {
fn event(&mut self, /*_: &Context, _: &mut Commands, _: size, _: &mut EventContext,*/ event: &Event) -> Result<bool> {
	let Self{state} = self;
	let d = match event {
		Event::Key('←') => { xy{x: -1, y: 0}}
		Event::Key('→') => { xy{x: 1, y: 0}}
		Event::Key('↓') => { xy{x: 0, y: -1}}
		_ => { return Ok(false); }
	};
	let current_block = state.last().unwrap();
	for &xy{x,y} in current_block {
		if d.y < 0 && y == 0 { return Ok(false); }
		if d.x < 0 && x == 0 { return Ok(false); }
		if d.x > 0 && x == grid.x-1 { return Ok(false); }
		let next = xy{x: x+d.x, y: y+d.y};
		for block in &state[0..state.len()-1] { for square in block { if &next == square { return Ok(false); } } }
	}
	let current_block = state.last_mut().unwrap();
	for square in current_block { *square += d; }
	//println!("{state:?}");
	Ok(true)
}}

impl Widget for App {
fn event(&mut self, _: &Context, _: &mut Commands, _: size, _: &mut EventContext, event: &Event) -> Result<bool> { Ok(matches!(event, Event::Trigger)) }
fn paint(&mut self, context: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	let Self{state} = self;
	let ref quads = state.iter().map(|block| block.iter().map(|&square| {
		let side = square_side as f32;
		let center = xy{x: screen.x as f32/2.-grid.x as f32*side/2., y: side/2.} + side*vec2::from(square);
		let xy{x,y} = xy::from(side/2.);
		[center+xy{x: -x, y: -y},center+xy{x: -x, y},center+xy{x, y},center+xy{x, y: -y}].map(|p| 2.*p/vec2::from(screen)-vec2::from(1.))
	})).flatten().flatten().collect::<Box<_>>();
	let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, quads.into_iter().map(|&xy{x,y}| square::Vertex{position: [x,y]}))?;
	let indices = buffer(context, BufferUsage::INDEX_BUFFER, quads.len()/4*6)?;
	{
		let mut indices = indices.write()?;
		for i in 0..quads.len()/4 {
			indices[i*6+0] = (i*4+0) as u32;
			indices[i*6+1] = (i*4+2) as u32;
			indices[i*6+2] = (i*4+1) as u32;
			indices[i*6+3] = (i*4+0) as u32;
			indices[i*6+4] = (i*4+3) as u32;
			indices[i*6+5] = (i*4+2) as u32;
		}
	}
	let mut pass = square::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;
	pass.begin_rendering(context, commands, target.clone(), None, true, &square::Uniforms::empty(), &[])?;
	commands.bind_index_buffer(indices.clone())?;
	commands.bind_vertex_buffers(0, vertices.clone())?;
	unsafe{commands.draw_indexed(indices.len() as _, 1, 0, 0, 0)}?;
	commands.end_rendering()?;
	Ok(())
}
}

use std::sync::{Mutex, MutexGuard};
#[derive(Default,Clone)] struct Arch<T>(Arc<Mutex<T>>);
impl<T> Arch<T> {
    pub fn new(inner: T) -> Self { Self(std::sync::Arc::new(Mutex::new(inner))) }
	pub fn clone(&self) -> Self { Self(self.0.clone()) }
    pub fn lock(&self) -> MutexGuard<'_, T> { self.0.lock().unwrap() }
}
unsafe impl<T> Send for Arch<T> {}
unsafe impl<T> Sync for Arch<T> {}
impl<T:Widget> Widget for Arch<T> {
	fn paint(&mut self, context: &Context, commands: &mut Commands, target: Arc<ImageView>, size: size, offset: int2) -> Result {
		self.lock().paint(context, commands, target, size, offset) }
	fn event(&mut self, context: &Context, commands: &mut Commands, size: size, event_context: &mut EventContext, event: &Event) -> Result<bool> {
		self.lock().event(context, commands, size, event_context, event) }
}

fn main() -> Result {
	let app : Arch<App> = Arch::new(App::new());
	use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
	let stop = AtomicBool::new(false);
	let stop = &stop;
	let trigger = new_trigger()?;
	let trigger = &trigger;
	std::thread::scope(|s| {
		std::thread::Builder::new().spawn_scoped(s, {let app : Arch<App> = Arch::clone(&app); move || /*Result::<()>::unwrap(try*/ {
			let fd = rustix::fs::open("/dev/input/event21", rustix::fs::OFlags::RDONLY, rustix::fs::Mode::empty()).unwrap();
			#[repr(C)] #[derive(Clone, Copy, Debug)] struct timeval { sec: i64, usec: i64 }
			#[repr(C)] #[derive(Clone, Copy, Debug)] struct input_event { time: timeval, r#type: u16, code: u16, value: i32 }
			unsafe impl bytemuck::Zeroable for input_event {}
			unsafe impl bytemuck::Pod for input_event {}
			while !stop.load(Relaxed) {
				let mut buffer = [0; std::mem::size_of::<input_event>()];
				assert_eq!(rustix::io::read(&fd, &mut buffer).unwrap(), buffer.len());
			 	let input_event{r#type, code, value, ..} = *bytemuck::from_bytes(&buffer);
				const SYN : u16 = 0; const KEY : u16 = 1; const REL : u16 = 2; const ABS : u16 = 3;
				match r#type {
					SYN => {},
					ABS => {
						const X : u16 = 0; const Y : u16 = 1;
						let d = {
							if value < -4096 { match code {X => '←', Y => '↑',_=>{continue;}} }
							else if value > 4096 { match code {X => '→', Y => '↓',_=>{continue;}} }
							else { continue; }
						};
						if app.lock().event(&Event::Key(d)).unwrap() { ui::trigger(trigger).unwrap(); }
					}
					_ => unreachable!("{type}")
				}
			}
		}})?;
		let r = run_with_trigger(trigger, "tetris", Box::new(|_,_| Ok(Box::new(app))));
		stop.store(true, Relaxed);
		r
	})
}
