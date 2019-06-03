#[macro_use]
extern crate log;

use std::sync::Arc;

use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, PrefabLoaderSystem, Processor, RonFormat},
    core::{
        bundle::SystemBundle,
        math::Vector3,
        transform::{Transform, TransformBundle},
        Float,
    },
    ecs::{
        Dispatcher,
        DispatcherBuilder,
        Entity,
        Read,
        ReadExpect,
        Resources,
        System,
        SystemData,
        WriteStorage,
    },
    input::{InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        formats::texture::ImageFormat,
        pass::{DrawDebugLinesDesc, DrawFlat2DDesc},
        rendy::{
            factory::Factory,
            graph::{
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{format::Format, image},
            mesh::{Normal, Position, TexCoord},
        },
        sprite::{SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        types::DefaultBackend,
        GraphCreator,
        RenderingSystem,
        Texture,
    },
    ui::UiBundle,
    utils::{application_root_dir, scene::BasicScenePrefab},
    window::{ScreenDimensions, Window, WindowBundle},
};
use amethyst_physics::PhysicsBundle;
use specs_physics::{
    bodies::BodyStatus,
    colliders::Shape,
    PhysicsBody,
    PhysicsBodyBuilder,
    PhysicsColliderBuilder,
};

pub type GamePrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

/// The Player `Resources` contains player relevant data and holds a reference
/// to the `Entity` that defines the player.
#[derive(Debug)]
pub struct Player {
    /// The player `Entity`.
    pub player: Entity,
}

#[derive(Default)]
struct GameState<'a, 'b> {
    /// `State` specific dispatcher.
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for GameState<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        info!("GameState.on_start");
        let world = data.world;

        // load scene handle
        let scene_handle = world.exec(|loader: PrefabLoader<'_, GamePrefabData>| {
            loader.load("prefab/scene.ron", RonFormat, ())
        });

        // load sprite sheets
        let character_handle =
            self.load_sprite_sheet("texture/character.png", "texture/character.ron", world);
        let objects_handle =
            self.load_sprite_sheet("texture/objects.png", "texture/objects.ron", world);

        // create dispatcher
        self.create_dispatcher(world);

        // initialise scene
        world.create_entity().with(scene_handle.clone()).build();

        // create player Entity
        let player = world
            .create_entity()
            .with(SpriteRender {
                sprite_sheet: character_handle.clone(),
                sprite_number: 0,
            })
            .with(PhysicsBodyBuilder::<Float>::from(BodyStatus::Dynamic).build())
            .with(
                PhysicsColliderBuilder::<Float>::from(Shape::Rectangle(
                    15.0.into(),
                    22.0.into(),
                    1.0.into(),
                ))
                .build(),
            )
            .with(Transform::from(Vector3::new(25.0, 50.0, 0.0)))
            .build();

        // create the player Resource
        world.add_resource(Player { player });

        // create obstacle Entity
        world
            .create_entity()
            .with(SpriteRender {
                sprite_sheet: objects_handle.clone(),
                sprite_number: 0,
            })
            .with(PhysicsBodyBuilder::<Float>::from(BodyStatus::Static).build())
            .with(
                PhysicsColliderBuilder::<Float>::from(Shape::Rectangle(
                    15.0.into(),
                    16.0.into(),
                    1.0.into(),
                ))
                .build(),
            )
            .with(Transform::from(Vector3::new(75.0, 50.0, 0.0)))
            .build();
    }

    fn fixed_update(&mut self, data: StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world.res);
        }

        Trans::None
    }
}

impl<'a, 'b> GameState<'a, 'b> {
    fn load_sprite_sheet(
        &mut self,
        texture_path: &str,
        ron_path: &str,
        world: &mut World,
    ) -> SpriteSheetHandle {
        // Load the sprite sheet necessary to render the graphics.
        // The texture is the pixel data
        // `sprite_sheet` is the layout of the sprites on the image
        // `texture_handle` is a cloneable reference to the texture
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(texture_path, ImageFormat::default(), (), &texture_storage)
        };

        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load(
            ron_path, // Here we load the associated ron file
            SpriteSheetFormat(texture_handle),
            (),
            &sprite_sheet_store,
        )
    }

    /// Creates the `State` specific `Dispatcher`.
    fn create_dispatcher(&mut self, world: &mut World) {
        if self.dispatcher.is_none() {
            let mut dispatcher_builder = DispatcherBuilder::new();
            PhysicsBundle::default()
                .with_debug_lines()
                .build(&mut dispatcher_builder)
                .expect("Failed to register PhysicsBundle");

            let mut dispatcher = dispatcher_builder.build();
            dispatcher.setup(&mut world.res);
            self.dispatcher = Some(dispatcher);
        }
    }
}

#[derive(Default)]
struct PlayerMovementSystem;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, Player>,
        WriteStorage<'s, PhysicsBody<Float>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, player, mut physics_bodies) = data;
        if let Some(physics_body) = physics_bodies.get_mut(player.player) {
            // handle movement on X axis
            if let Some(movement) = input.axis_value("leftright") {
                physics_body.velocity.x = movement.into();
            }

            // handle movement on Y axis
            if let Some(movement) = input.axis_value("updown") {
                physics_body.velocity.y = movement.into();
            }
        }
    }
}

fn main() -> amethyst::Result<()> {
    //amethyst::start_logger(Default::default());
    amethyst::Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", amethyst::LogLevelFilter::Warn)
        .level_for("rendy_factory::factory", amethyst::LogLevelFilter::Warn)
        .level_for(
            "rendy_memory::allocator::dynamic",
            amethyst::LogLevelFilter::Warn,
        )
        .level_for(
            "rendy_graph::node::render::pass",
            amethyst::LogLevelFilter::Warn,
        )
        .level_for("rendy_graph::node::present", amethyst::LogLevelFilter::Warn)
        .level_for("rendy_graph::graph", amethyst::LogLevelFilter::Warn)
        .level_for(
            "rendy_memory::allocator::linear",
            amethyst::LogLevelFilter::Warn,
        )
        .level_for("rendy_wsi", amethyst::LogLevelFilter::Warn)
        .start();

    let app_root = application_root_dir()?;

    // display configuration
    let display_config_path = app_root.join("examples/resources/display_config.ron");

    // key bindings
    let key_bindings_path = app_root.join("examples/resources/input.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(key_bindings_path)?,
        )?
        .with_bundle(UiBundle::<DefaultBackend, StringBindings>::new())?
        //.with_bundle(PhysicsBundle::default().with_debug_lines())?
        .with(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        )
        .with(PrefabLoaderSystem::<GamePrefabData>::default(), "", &[])
        .with(
            PlayerMovementSystem::default(),
            "player_movement_system",
            &[],
        )
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));

    let mut game = Application::build(app_root.join("examples/assets"), GameState::default())?
        .build(game_data)?;

    game.run();

    Ok(())
}

// This graph structure is used for creating a proper `RenderGraph` for
// rendering. A renderGraph can be thought of as the stages during a render
// pass. In our case, we are only executing one subpass (DrawFlat2D, or the
// sprite pass). This graph also needs to be rebuilt whenever the window is
// resized, so the boilerplate code for that operation is also here.
#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for ExampleGraph {
    // This trait method reports to the renderer if the graph must be rebuilt,
    // usually because the window has been resized. This implementation checks
    // the screen size and returns true if it has changed.
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the
        // same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    // This is the core of a RenderGraph, which is building the actual graph with
    // subpasses and target images.
    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        // Retrieve a reference to the target window, which is created by the
        // WindowBundle
        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);

        // Create a new drawing surface in our window
        let surface = factory.create_surface(&window);
        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = image::Kind::D2(
            dbg!(dimensions.width()) as u32,
            dimensions.height() as u32,
            1,
            1,
        );

        // Begin building our RenderGraph
        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        // Create our single `Subpass`, which is the DrawFlat2D pass.
        // We pass the subpass builder a description of our pass for construction
        let sprite = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawDebugLinesDesc::new().builder())
                .with_group(DrawFlat2DDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        // Finally, add the pass to the graph
        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(sprite));

        graph_builder
    }
}
