#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, size, int2, image::{Image, xy, rgb8}, Widget, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, image, WriteDescriptorSet, linear};
shader!{view}

struct App {
	state: Image<Box<[u8]>>,
	ops: usize,
	steps: usize,
	done: bool
}

impl App {
	fn new() -> Self { 
		let size = xy{x: 80, y: 45};
		let mut state = Image::zero(size);
		state[xy{x: size.x/2, y: size.y/2}] = 255;
		Self{state, ops: 0, steps: 0, done: false}
	}
}

impl Widget for App { 
fn event(&mut self, _: &Context, _: &mut Commands, _: size, _: &mut ui::EventContext, event: &ui::Event) -> Result<bool> { Ok(!self.done && matches!(event, ui::Event::Idle)) }
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	use rand::random_range;
	let Self{state, ops, steps, done} = self;
	let mut op = 0;
	for _ in 0..1024 {
		let p0@xy{x,y} = xy{x: random_range(1..state.size.x-1), y: random_range(1..state.size.y-1)};
		if state[p0] == 0 { continue; } // fast-path // TODO: random_range in non-zero
		let p1 = match random_range(0..4) {
			0 => xy{x: x-1, y},
			1 => xy{x: x+1, y},
			2 => xy{x, y: y-1},
			3 => xy{x, y: y+1},
			_ => unreachable!()
		};
		if state[p0] > state[p1]+1 { state[p0] -= 1; state[p1] += 1; op+=1; break; }
	}
	*ops += op; *steps += 1;
	//println!("{ops} {steps}"); 
	if *ops >= 978 || *steps >= 1431 { *done = true; }
	let mut pass = view::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;
	let image = image(context, commands, state.as_ref().map(|&s| 
		//if s < 4 { [rgb8{r:0,g:0,b:0},rgb8{r:0xFF,g:0xFF,b:0},rgb8{r:0,g:0xFF,b:0xFF},rgb8{r:0,g:0xFF,b:0}][s as usize] } else { rgb8{r:s,g:s,b:s} }
		{let g = s.min(15)*16; rgb8{r:g,g:g,b:g}}.into()).as_ref())?;
	pass.begin_rendering(context, commands, target.clone(), None, true, &view::Uniforms::empty(), &[
		WriteDescriptorSet::image_view(0, ImageView::new_default(&image)?),
		WriteDescriptorSet::sampler(1, linear(context)),
	])?;
	unsafe{commands.draw(3, 1, 0, 0)}?;
	commands.end_rendering()?;
	Ok(())
}
}

fn main() -> Result { run("toy", Box::new(|_,_| Ok(Box::new(App::new())))) }
