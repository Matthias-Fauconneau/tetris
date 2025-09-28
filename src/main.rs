#![feature(slice_from_ptr_range)] // shader
use ui::{Result, run, new_trigger, size, int2, image::{Image, xy, rgb8}, Widget, vulkan, shader};
use vulkan::{Context, Commands, Arc, ImageView, PrimitiveTopology, image, WriteDescriptorSet, linear};
shader!{view}

struct App {
	state: Image<Box<[u8]>>
}

impl App {
	fn new() -> Self { 
		let size = xy{x: 1280, y: 720};
		let mut state = Image::zero(size);
		state[xy{x: 1*size.x/3, y: size.y/2}] = 1;
		state[xy{x: 2*size.x/3, y: size.y/2}] = 2;
		Self{state}
	}
}

impl Widget for App { 
fn event(&mut self, _: &Context, _: &mut Commands, _: size, _: &mut ui::EventContext, event: &ui::Event) -> Result<bool> { Ok(matches!(event, ui::Event::Idle)) }
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result {
	use rand::random_range;
	let Self{state} = self;
	for _ in 0..1024 {
		loop {
			let p0@xy{x,y} = xy{x: random_range(1..state.size.x-1), y: random_range(1..state.size.y-1)};
			let p1 = match random_range(0..4) {
				0 => xy{x: x-1, y},
				1 => xy{x: x+1, y},
				2 => xy{x, y: y-1},
				3 => xy{x, y: y+1},
				_ => unreachable!()
			};
			if state[p0] == 1 && state[p1] == 0 { state[p1] = 1; break; }
			if state[p0] == 2 && state[p1] == 0 { state[p1] = 2; break; }
			if state[p0] == 1 && state[p1] == 2 { state[p0] = 3; state[p1] = 3; break; }
		}
	}
	let mut pass = view::Pass::new(context, false, PrimitiveTopology::TriangleList, false)?;
	let image = image(context, commands, state.as_ref().map(|&s| [rgb8{r:0,g:0,b:0},rgb8{r:0xFF,g:0xFF,b:0},rgb8{r:0,g:0xFF,b:0xFF},rgb8{r:0,g:0xFF,b:0}][s as usize].into()).as_ref())?;
	pass.begin_rendering(context, commands, target.clone(), None, true, &view::Uniforms::empty(), &[
		WriteDescriptorSet::image_view(0, ImageView::new_default(&image)?),
		WriteDescriptorSet::sampler(1, linear(context)),
	])?;
	unsafe{commands.draw(3, 1, 0, 0)}?;
	commands.end_rendering()?;
	Ok(())
}
}

fn main() -> Result { run(new_trigger().unwrap(), "view", Box::new(|_,_| Ok(Box::new(App::new())))) }
