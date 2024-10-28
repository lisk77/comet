//use comet_resources::Vertex;
use crate::math::{
	Vec2,
	Vec3
};
use component_derive::Component;

pub trait Component: Send + Sync + PartialEq + Default +  'static {
	fn new() -> Self where Self: Sized;

	fn type_id() -> std::any::TypeId {
		std::any::TypeId::of::<Self>()
	}

	fn type_name() -> String {
		std::any::type_name::<Self>().to_string()
	}
}

// ##################################################
// #                    BASIC                       #
// ##################################################

#[derive(Component)]
pub struct Position2D {
	position: Vec2
}

#[derive(Component)]
pub struct Position3D {
	position: Vec3
}

#[derive(Component)]
pub struct Rotation2D {
	theta: f32
}

#[derive(Component)]
pub struct Rotation3D {
	theta_x: f32,
	theta_y: f32,
	theta_z: f32
}

#[derive(Component)]
pub struct Renderer2D {
	is_visible: bool,
	texture: &'static str,
	scale: Vec2
}

impl Position2D {
	pub fn from_vec(vec: Vec2) -> Self {
		Self {
			position: vec
		}
	}

	pub fn as_vec(&self) -> Vec2 {
		self.position
	}

	pub fn x(&self) -> f32 {
		self.position.x()
	}

	pub fn y(&self) -> f32 {
		self.position.y()
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.position.set_x(new_x);
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.position.set_y(new_y);
	}
}

impl Position3D {
	pub fn from_vec(vec: Vec3) -> Self {
		Self {
			position: vec
		}
	}

	pub fn as_vec(&self) -> Vec3 {
		self.position
	}

	pub fn x(&self) -> f32 {
		self.position.x()
	}

	pub fn y(&self) -> f32 {
		self.position.y()
	}

	pub fn z(&self) -> f32 {
		self.position.z()
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.position.set_x(new_x);
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.position.set_y(new_y);
	}

	pub fn set_z(&mut self, new_z: f32) {
		self.position.set_z(new_z);
	}
}

pub trait Render {
	fn is_visible(&self) -> bool;
	fn set_visibility(&mut self, is_visible: bool);
	fn get_texture(&self) -> String;
	fn set_texture(&mut self, texture: &'static str);
	//fn get_vertex_data(&self) -> Vec<Vertex>;
}

impl Render for Renderer2D {
	fn is_visible(&self) -> bool {
		self.is_visible
	}

	fn set_visibility(&mut self, is_visible: bool) {
		self.is_visible = is_visible;
	}

	fn get_texture(&self) -> String {
		self.texture.clone().parse().unwrap()
	}

	/// Use the actual file name of the texture instead of the path
	/// e.g. "comet_icon.png" instead of "resources/textures/comet_icon.png"
	/// The resource manager will already look in the resources/textures folder
	fn set_texture(&mut self, texture: &'static str) {
		self.texture = texture;
	}

	/*fn get_vertex_data(&self) -> Vec<Vertex> {
		vec![
			Vertex::new([0.0, 0.0, 0.0], [0.0, 0.0]),
			Vertex::new([1.0, 0.0, 0.0], [1.0, 0.0]),
			Vertex::new([1.0, 1.0, 0.0], [1.0, 1.0]),
			Vertex::new([0.0, 1.0, 0.0], [0.0, 1.0])
		]
	}*/
}

// ##################################################
// #                   BUNDLES                      #
// ##################################################

#[derive(Component)]
pub struct Transform2D {
	position: Position2D,
	rotation: Rotation2D
}

#[derive(Component)]
pub struct Transform3D {
	position: Position3D,
	rotation: Rotation3D
}

impl Transform2D {
	pub fn position(&self) -> &Position2D {
		&self.position
	}

	pub fn position_mut(&mut self) -> &mut Position2D {
		&mut self.position
	}

	pub fn rotation(&self) -> &Rotation2D {
		&self.rotation
	}

	pub fn rotation_mut(&mut self) -> &mut Rotation2D {
		&mut self.rotation
	}
}

impl Transform3D {
	pub fn position(&self) -> &Position3D {
		&self.position
	}

	pub fn position_mut(&mut self) -> &mut Position3D {
		&mut self.position
	}

	pub fn rotation(&self) -> &Rotation3D {
		&self.rotation
	}

	pub fn rotation_mut(&mut self) -> &mut Rotation3D {
		&mut self.rotation
	}
}


