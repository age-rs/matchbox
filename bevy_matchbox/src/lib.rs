#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use bevy::{
    ecs::system::Command,
    prelude::{Commands, Component, Deref, DerefMut, Resource, World},
    tasks::IoTaskPool,
};
pub use matchbox_socket;
use matchbox_socket::{BuildablePlurality, WebRtcSocket, WebRtcSocketBuilder};
use std::marker::PhantomData;

/// A [`WebRtcSocket`] as a [`Component`] or [`Resource`].
///
/// As a [`Component`], directly
/// ```
/// use bevy_matchbox::prelude::*;
/// use bevy::prelude::*;
///
/// fn open(mut commands: Commands) {
///     let room_url = "ws://matchbox.example.com";
///     let builder = WebRtcSocketBuilder::new(room_url).add_channel(ChannelConfig::reliable());
///     commands.spawn(MatchboxSocket::from(builder));
/// }
///
/// fn close<C: BuildablePlurality + 'static>(
///     mut commands: Commands,
///     socket: Query<Entity, With<MatchboxSocket<C>>>
/// ) {
///     let socket = socket.single();
///     commands.entity(socket).despawn()
/// }
///
/// ```
///
/// As a [`Resource`], with [`Commands`]
/// ```
/// use bevy_matchbox::prelude::*;
/// use bevy::prelude::*;
///
/// fn open(mut commands: Commands) {
///     let room_url = "ws://matchbox.example.com";
///     commands.open_socket(WebRtcSocketBuilder::new(room_url).add_channel(ChannelConfig::reliable()));
/// }
///
/// fn close(mut commands: Commands) {
///     commands.close_socket::<SingleChannel>();
/// }
/// ```
///
/// As a [`Resource`], directly
///
/// ```
/// use bevy_matchbox::prelude::*;
/// use bevy::prelude::*;
///
/// fn open(mut commands: Commands) {
///     let room_url = "ws://matchbox.example.com";
///     let builder = WebRtcSocketBuilder::new(room_url).add_channel(ChannelConfig::reliable());
///     commands.insert_resource(MatchboxSocket::from(builder));
/// }
///
/// fn close(mut commands: Commands) {
///     commands.remove_resource::<MatchboxSocket<SingleChannel>>();
/// }
/// ```
///
/// To create and destroy this resource use the [`OpenSocket`] and [`CloseSocket`] [`Command`]s respectively.
#[derive(Resource, Component, Debug, Deref, DerefMut)]
pub struct MatchboxSocket<C: BuildablePlurality>(WebRtcSocket<C>);

impl<C: BuildablePlurality> From<WebRtcSocketBuilder<C>> for MatchboxSocket<C> {
    fn from(builder: WebRtcSocketBuilder<C>) -> Self {
        let (socket, message_loop) = builder.build();

        let task_pool = IoTaskPool::get();
        task_pool.spawn(message_loop).detach();

        MatchboxSocket(socket)
    }
}

/// A [`Command`] used to open a [`MatchboxSocket`] and allocate it as a resource.
struct OpenSocket<C: BuildablePlurality>(WebRtcSocketBuilder<C>);

impl<C: BuildablePlurality + 'static> Command for OpenSocket<C> {
    fn write(self, world: &mut World) {
        world.insert_resource(MatchboxSocket::from(self.0));
    }
}

/// A [`Commands`] extension used to open a [`MatchboxSocket`] and allocate it as a resource.
pub trait OpenSocketExt<C: BuildablePlurality> {
    /// Opens a [`MatchboxSocket`] and allocates it as a resource.
    fn open_socket(&mut self, socket_builder: WebRtcSocketBuilder<C>);
}

impl<'w, 's, C: BuildablePlurality + 'static> OpenSocketExt<C> for Commands<'w, 's> {
    fn open_socket(&mut self, socket_builder: WebRtcSocketBuilder<C>) {
        self.add(OpenSocket(socket_builder))
    }
}

/// A [`Command`] used to close a [`WebRtcSocket`], deleting the [`MatchboxSocket`] resource.
struct CloseSocket<C: BuildablePlurality>(PhantomData<C>);

impl<C: BuildablePlurality + 'static> Command for CloseSocket<C> {
    fn write(self, world: &mut World) {
        world.remove_resource::<MatchboxSocket<C>>();
    }
}

/// A [`Commands`] extension used to close a [`WebRtcSocket`], deleting the [`MatchboxSocket`] resource.
pub trait CloseSocketExt {
    /// Delete the [`MatchboxSocket`] resource.
    fn close_socket<C: BuildablePlurality + 'static>(&mut self);
}

impl<'w, 's> CloseSocketExt for Commands<'w, 's> {
    fn close_socket<C: BuildablePlurality + 'static>(&mut self) {
        self.add(CloseSocket::<C>(PhantomData::default()))
    }
}

/// use `bevy_matchbox::prelude::*;` to import common resources and commands
pub mod prelude {
    pub use crate::{CloseSocketExt, MatchboxSocket, OpenSocketExt};
    pub use matchbox_socket::{
        BuildablePlurality, ChannelConfig, MultipleChannels, PeerId, PeerState, SingleChannel,
        WebRtcSocketBuilder,
    };
}