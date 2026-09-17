use winit::window::WindowAttributes;

pub struct WindowConfig {
	inner_size: Option<WindowSize>,
	min_inner_size: Option<WindowSize>,
	max_inner_size: Option<WindowSize>,
	//position: Option,
	resizable: bool,
	// enabled_buttons,
	title: String,
	maximized: bool,
	visible: bool,
	transparent: bool,
	blur: bool,
	decorations: bool,
	// window_icon,
	// preferred_theme,
	resize_increments: Option<WindowSize>,
	content_protected: bool,
	// window_level,
	active: bool,
	// cursor,
	// fullscreen,
}

impl WindowConfig {
	#[inline(always)]
	pub fn with_inner_size(mut self, inner_size: impl Into<WindowSize>) -> Self {
		self.inner_size = Some(inner_size.into());
		self
	}

	#[inline(always)]
	pub fn with_min_inner_size(mut self, min_inner_size: impl Into<WindowSize>) -> Self {
		self.min_inner_size = Some(min_inner_size.into());
		self
	}

	#[inline(always)]
	pub fn with_max_inner_size(mut self, max_inner_size: impl Into<WindowSize>) -> Self {
		self.max_inner_size = Some(max_inner_size.into());
		self
	}

	#[inline(always)]
	pub fn with_resizable(mut self, resizable: bool) -> Self {
		self.resizable = resizable;
		self
	}

	#[inline(always)]
	pub fn with_title(mut self, title: impl Into<String>) -> Self {
		self.title = title.into();
		self
	}

	#[inline(always)]
	pub fn with_maximized(mut self, maximized: bool) -> Self {
		self.maximized = maximized;
		self
	}

	#[inline(always)]
	pub fn with_visible(mut self, visible: bool) -> Self {
		self.visible = visible;
		self
	}

	#[inline(always)]
	pub fn with_transparent(mut self, transparent: bool) -> Self {
		self.transparent = transparent;
		self
	}

	#[inline(always)]
	pub fn with_blur(mut self, blur: bool) -> Self {
		self.blur = blur;
		self
	}

	#[inline(always)]
	pub fn with_decorations(mut self, decorations: bool) -> Self {
		self.decorations = decorations;
		self
	}

	#[inline(always)]
	pub fn with_resize_increments(mut self, resize_increments: impl Into<WindowSize>) -> Self {
		self.resize_increments = Some(resize_increments.into());
		self
	}

	#[inline(always)]
	pub fn with_content_protected(mut self, content_protected: bool) -> Self {
		self.content_protected = content_protected;
		self
	}

	#[inline(always)]
	pub fn with_active(mut self, active: bool) -> Self {
		self.active = active;
		self
	}
}

impl Default for WindowConfig {
	fn default() -> Self {
		Self {
			inner_size: Default::default(),
			min_inner_size: Default::default(),
			max_inner_size: Default::default(),
			resizable: true,
			title: Default::default(),
			maximized: false,
			visible: true,
			transparent: false,
			blur: false,
			decorations: true,
			resize_increments: Default::default(),
			content_protected: false,
			active: true,
		}
	}
}

impl Into<WindowAttributes> for WindowConfig {
	fn into(self) -> WindowAttributes {
		let mut attributes = WindowAttributes::default()
			//	.with_position(self.position.map(|size| size.into()))
			.with_resizable(self.resizable)
			.with_title(self.title)
			.with_maximized(self.maximized)
			.with_visible(self.visible)
			.with_transparent(self.transparent)
			.with_blur(self.blur)
			.with_decorations(self.decorations)
			.with_content_protected(self.content_protected)
			.with_active(self.active);

		attributes.inner_size = self.inner_size.map(|size| size.into());
		attributes.min_inner_size = self.min_inner_size.map(|size| size.into());
		attributes.max_inner_size = self.max_inner_size.map(|size| size.into());
		attributes.resize_increments = self.resize_increments.map(|size| size.into());

		attributes
	}
}

pub struct PhysicalSize {
	width: u32,
	height: u32,
}

pub struct LogicalSize {
	width: f64,
	height: f64,
}

pub enum WindowSize {
	Logical(LogicalSize),
	Physical(PhysicalSize),
}

impl From<PhysicalSize> for WindowSize {
	fn from(value: PhysicalSize) -> Self {
		Self::Physical(value)
	}
}

impl From<LogicalSize> for WindowSize {
	fn from(value: LogicalSize) -> Self {
		Self::Logical(value)
	}
}

impl Into<winit::dpi::Size> for WindowSize {
	fn into(self) -> winit::dpi::Size {
		match self {
			WindowSize::Logical(logical_size) => winit::dpi::Size::Logical(
				winit::dpi::LogicalSize::new(logical_size.width, logical_size.height),
			),
			WindowSize::Physical(physical_size) => winit::dpi::Size::Physical(
				winit::dpi::PhysicalSize::new(physical_size.width, physical_size.height),
			),
		}
	}
}
