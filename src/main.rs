#![feature(slice_from_ptr_range)]/*shader*/#![allow(non_upper_case_globals,non_camel_case_types)]
use ui::{Result, run, size, xy, int2, vector::vec2, Widget, EventContext, Event, vulkan, shader, image::{rgb, rgbf}};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, from_iter, BufferUsage, buffer};
shader!{square}

struct App {
	state: Vec<Vec<int2>>,
	colors: Vec<rgbf>,
	subfall: f32,
}

const screen: int2 = xy{x: 3840, y: 2160};
const square_side: i32 = 48;
const grid: int2 = xy{x: screen.x/square_side/4+1/*=21*/, y: screen.y/square_side/*=45*/};
impl App {
	fn new() -> Self { let mut s = Self{state: Vec::new(), colors: Vec::new(), subfall: 0.}; s.spawn(); s }
	fn spawn(&mut self) {
		let Self{state, colors, subfall} = self;
		let patterns = [
			vec![xy{x: 0, y: 0}],
			vec![xy{x: 0, y: 0},xy{x: 0, y: 1},xy{x: 0, y: 2},xy{x: 0, y: 3}]
		];
		state.push(patterns[rand::random_range(0..2)].iter().map(|xy{x,y}| xy{x: x+10, y: y+44}).collect());
		let possible_colors = vec![rgb{r:1.,g:0.,b:0.},rgb{r:0.,g:1.,b:0.},rgb{r:0.,g:0.,b:1.}];
		colors.push(possible_colors[(state.len()-1)%possible_colors.len()]);
		*subfall = 0.;
	}
	fn move_current_block(&mut self, d: int2) -> bool {
		let Self{state, ..} = self;
		let current_block = state.last().unwrap();
		for &xy{x,y} in current_block {
			if d.y < 0 && y == 0 { return false; }
			if d.x < 0 && x == 0 { return false; }
			if d.x > 0 && x == grid.x-1 { return false; }
			let next = xy{x: x+d.x, y: y+d.y};
			for block in &state[0..state.len()-1] { for square in block { if &next == square { return false; } } }
		}
		let current_block = state.last_mut().unwrap();
		for square in current_block { *square += d; }
		let any_column_full_up_to = |query| {
			for &query in query {
				let column_full_up_to = |xy{x,y}| {
					for y in 0..y { // any hole
						let filled = |query| {
							for block in &*state { for &square in block { if square == query  { return true; } } }
							false
						};
						if !filled(xy{x,y}) { return false; } // hole
					}
					true // no holes
				};
				if column_full_up_to(query) { return true; } // any full
			}
			false // no full
		};
		if any_column_full_up_to(state.last().unwrap()) { // touch
			self.spawn()
		}
		true
	}
}

impl Widget for App {
fn event(&mut self, _: &Context, _: &mut Commands, _: size, _: &mut EventContext, event: &Event) -> Result<bool> {
	Ok(match event {
		Event::Idle => { true } // Autofall
		Event::Key('←'|'a') => { self.move_current_block(xy{x: -1, y: 0}) }
		Event::Key('→'|'d') => { self.move_current_block(xy{x: 1, y: 0}) }
		Event::Key('↓'|' ') => { self.move_current_block(xy{x: 0, y: -1}) }
		_ => { false }
	})
}
fn paint(&mut self, context: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	let Self{subfall,..} = self;
	let side = square_side as f32;
	*subfall += side/60./2.;
	if *subfall > 1. {
		*subfall -= 1.;
		self.move_current_block(xy{x: 0, y: -1});
	}
	let Self{state,colors,subfall} = self;
	let square_to_quad = |&square, color:&rgbf, subfall| {
		let center = xy{x: screen.x as f32/2.-grid.x as f32*side/2., y: side/2.+subfall} + side*vec2::from(square);
		let xy{x,y} = xy::from(side/2.);
		[center+xy{x: -x, y: -y},center+xy{x: -x, y},center+xy{x, y},center+xy{x, y: -y}].map(|p| (2.*p/vec2::from(screen)-vec2::from(1.), *color))
	};
	let mut quads = Vec::new();
	for (i, (block, color)) in state.iter().zip(colors).enumerate() {
		for square in block { quads.extend(square_to_quad(square, color, if i==state.len()-1 {*subfall} else {0.})); }
	}
	let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, quads.iter().map(|&(xy{x,y}, rgb{r,g,b})| square::Vertex{position: [x,y], color: [r,g,b]}))?;
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

fn main() -> Result { run("tetris", Box::new(|_,_| Ok(Box::new(App::new())))) }
