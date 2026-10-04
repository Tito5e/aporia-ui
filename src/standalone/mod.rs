use std::{error::Error, marker::PhantomData};

use pollster::FutureExt;
use wgpu::{DeviceDescriptor, Instance, InstanceDescriptor, RequestAdapterOptions};
use winit::event_loop::EventLoop;

use crate::standalone::{
	context::Context,
	initializer::{AppInitializer, StateInitializer},
	runner::AppRunner,
};

pub(crate) mod context;
pub(crate) mod initializer;
pub(crate) mod runner;
pub(crate) mod state;
