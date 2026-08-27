use super::*;

impl Renderer for Renderer2D {
    type Handle = RenderHandle2D;

    fn new(
        window: Arc<Window>,
        clear_color: Option<impl Color>,
        event_sender: flume::Sender<Renderer2DEvent>,
    ) -> Self {
        let asset_provider = comet_assets::AssetProvider::new(comet_assets::AssetManager::new());
        let (font_job_sender, font_result_receiver) = super::text::start_font_variant_worker();
        Self {
            render_state: RenderState::new(window, clear_color),
            #[cfg(feature = "diagnostics")]
            diagnostics: diagnostics::Renderer2DDiagnosticsPublisher::from_env(),
            #[cfg(feature = "diagnostics")]
            frame_diagnostics: diagnostics::Renderer2DDiagnostics::default(),
            #[cfg(feature = "diagnostics")]
            latest_snapshot_produced_at: None,
            #[cfg(feature = "diagnostics")]
            latest_snapshot_sequence: None,
            #[cfg(feature = "diagnostics")]
            last_rendered_snapshot_sequence: None,
            asset_provider,
            graph: RenderGraph::new(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.0,
            event_sender,
            font_cache: std::collections::HashMap::new(),
            glyph_cache: std::collections::HashMap::new(),
            font_job_sender,
            font_result_receiver,
            pending_font_variants: std::collections::HashSet::new(),
            failed_font_variants: std::collections::HashSet::new(),
            sprite_instances: Vec::new(),
            sprite_instance_staging: Vec::new(),
            world_text_vertices: Vec::new(),
            world_text_indices: Vec::new(),
            world_text_staging_vertices: Vec::new(),
            world_text_staging_indices: Vec::new(),
            screen_text_vertices: Vec::new(),
            screen_text_indices: Vec::new(),
            screen_text_staging_vertices: Vec::new(),
            screen_text_staging_indices: Vec::new(),
            gizmo_vertices: Vec::new(),
            gizmo_indices: Vec::new(),
            gizmo_staging_vertices: Vec::new(),
            gizmo_staging_indices: Vec::new(),
        }
    }

    fn init_assets(&mut self, app: &::comet_app::App) {
        if app.has_context::<comet_assets::AssetProvider>() {
            self.asset_provider = app.context::<comet_assets::AssetProvider>().clone();
        }
        self.setup_atlas_pipeline(comet_assets::TextureAtlas::with_capacity(512));
    }

    fn apply_command(&mut self, command: <Self::Handle as RendererHandle>::Command) {
        match command {
            Renderer2DCommand::Clear => {}
            Renderer2DCommand::ResolveAtlasRef(path) => {
                let atlas_ref = self
                    .render_state
                    .resources()
                    .get_asset_atlas_handle("atlas")
                    .and_then(|handle| {
                        self.asset_provider
                            .with(handle, |atlas| {
                                atlas
                                    .textures()
                                    .get(path.as_str())
                                    .copied()
                                    .map(|region| AtlasRef::new(region, handle))
                            })
                            .flatten()
                    });

                let mut dynamic_image_handle: Option<comet_assets::Asset<comet_assets::Image>> =
                    None;
                let atlas_ref = atlas_ref.or_else(|| {
                    if let Some(image_handle) = self
                        .asset_provider
                        .find_by_path::<comet_assets::Image>(path.clone())
                    {
                        dynamic_image_handle = Some(image_handle);
                        return self.ensure_image_in_atlas(image_handle);
                    }

                    let fs_path = comet_assets::resolve_asset_path(path.as_str());
                    let bytes = std::fs::read(&fs_path).ok()?;
                    let image = comet_assets::Image::from_bytes(&bytes, false).ok()?;
                    let image_handle = self.asset_provider.add(image)?;
                    self.asset_provider
                        .track_for_reload::<comet_assets::Image>(image_handle, path.clone());
                    let result = self.ensure_image_in_atlas(image_handle);
                    if result.is_some() {
                        dynamic_image_handle = Some(image_handle);
                    }
                    result
                });

                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::AtlasRef(atlas_ref, dynamic_image_handle));
            }
            Renderer2DCommand::EnsureHandleInAtlas(handle) => {
                let atlas_ref = self.ensure_image_in_atlas(handle);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::AtlasRef(atlas_ref, None));
            }
            Renderer2DCommand::Size => {
                let _ = self.event_sender.send(Renderer2DEvent::Size(self.size()));
            }
            Renderer2DCommand::ScaleFactor => {
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::ScaleFactor(self.scale_factor()));
            }
            Renderer2DCommand::PrecomputedTextBounds {
                text,
                font,
                font_size,
            } => {
                let bounds = self.precompute_text_bounds(&text, font, font_size);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PrecomputedTextBounds {
                        width: bounds.x(),
                        height: bounds.y(),
                    });
            }

            Renderer2DCommand::AddRenderPass(desc) => {
                let pass_output = self.add_pass(desc);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PassAdded(pass_output));
            }
            Renderer2DCommand::RemoveRenderPass(label) => {
                self.remove_pass(&label);
                let _ = self.event_sender.send(Renderer2DEvent::PassRemoved);
            }
            Renderer2DCommand::SetPassOutput(label, output) => {
                let handle = self.set_pass_output(&label, output);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PassOutputSet(handle));
            }
            Renderer2DCommand::SetPassRenderTarget(label, render_target) => {
                self.set_pass_render_target(&label, render_target);
                let _ = self.event_sender.send(Renderer2DEvent::PassRenderTargetSet);
            }
        }
    }

    fn window(&self) -> &Window {
        self.render_state.window()
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.render_state.size()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.render_state.set_size(new_size);
            self.render_state.config_mut().width = new_size.width;
            self.render_state.config_mut().height = new_size.height;
            self.render_state.configure_surface();
            self.graph.on_resize(
                self.render_state.device(),
                self.render_state.queue(),
                new_size.width,
                new_size.height,
            );
        }
    }

    fn scale_factor(&self) -> f64 {
        self.render_state.scale_factor()
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.render_state.set_scale_factor(scale_factor);
    }

    fn update(&mut self) -> f32 {
        let now = std::time::Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        let _ = self
            .event_sender
            .send(Renderer2DEvent::FrameTime(self.delta_time));
        self.delta_time
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        #[cfg(feature = "diagnostics")]
        let frame_started = std::time::Instant::now();
        let output = self.render_state.surface().get_current_texture()?;
        #[cfg(feature = "diagnostics")]
        let surface_wait_time = frame_started.elapsed();
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let clear_color = self.render_state.clear_color();
        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;

        self.graph.execute(
            self.render_state.device(),
            self.render_state.queue(),
            &output_view,
            clear_color,
            format,
            width,
            height,
        );

        output.present();

        #[cfg(feature = "diagnostics")]
        if let Some(diagnostics) = &mut self.diagnostics {
            if let Some(sequence) = self.latest_snapshot_sequence {
                if self.last_rendered_snapshot_sequence == Some(sequence) {
                    self.frame_diagnostics.reused_snapshots += 1;
                }
                self.last_rendered_snapshot_sequence = Some(sequence);
                self.frame_diagnostics.snapshot_sequence = sequence;
            }
            if let Some(produced_at) = self.latest_snapshot_produced_at {
                self.frame_diagnostics.snapshot_age_ms =
                    (produced_at.elapsed().as_secs_f64() * 1_000_000.0).round() / 1000.0;
            }
            self.frame_diagnostics.presentation_interval_ms =
                (self.delta_time as f64 * 1_000_000.0).round() / 1000.0;

            let cpu_frame_time = frame_started.elapsed();
            let cpu_frame_time_ms = cpu_frame_time.as_secs_f64() * 1000.0;
            let surface_wait_time_ms = surface_wait_time.as_secs_f64() * 1000.0;
            let cpu_render_work_time_ms = cpu_frame_time
                .saturating_sub(surface_wait_time)
                .as_secs_f64()
                * 1000.0;
            self.frame_diagnostics.cpu_frame_time_ms =
                (cpu_frame_time_ms * 1000.0).round() / 1000.0;
            self.frame_diagnostics.surface_wait_time_ms =
                (surface_wait_time_ms * 1000.0).round() / 1000.0;
            self.frame_diagnostics.cpu_render_work_time_ms =
                (cpu_render_work_time_ms * 1000.0).round() / 1000.0;
            self.frame_diagnostics.passes = self.graph.node_count() as u32;
            self.frame_diagnostics.draw_calls = self.frame_diagnostics.passes;
            self.frame_diagnostics.pending_font_jobs = self.pending_font_variants.len() as u32;
            diagnostics.publish(&self.frame_diagnostics);
        }

        Ok(())
    }
}

struct ErasedRenderer2D {
    renderer: Renderer2D,
    cmd_rx: flume::Receiver<Renderer2DCommand>,
    frame_mailbox: FrameMailbox2D,
}

impl comet_window::ErasedRenderer for ErasedRenderer2D {
    fn init_assets(&mut self, app: &comet_app::App) {
        self.renderer.init_assets(app);
    }
    fn drain_commands(&mut self) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            self.renderer.apply_command(cmd);
        }

        let frame = self.frame_mailbox.lock().unwrap().take();
        if let Some(frame) = frame {
            self.renderer.submit_frame(
                #[cfg(feature = "diagnostics")]
                frame.sequence,
                #[cfg(feature = "diagnostics")]
                frame.produced_at,
                #[cfg(feature = "diagnostics")]
                frame.replaced_frames,
                frame.camera,
                frame.draws,
                frame.texts,
                frame.screen_texts,
                frame.referenced_handles,
                frame.gizmo_shapes,
            );
        }
    }
    fn window(&self) -> &winit::window::Window {
        self.renderer.window()
    }
    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size()
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }
    fn scale_factor(&self) -> f64 {
        self.renderer.scale_factor()
    }
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.renderer.set_scale_factor(scale_factor);
    }
    fn update(&mut self) -> f32 {
        self.renderer.update()
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render()
    }
}

pub struct Renderer2DModule;

impl Renderer2DModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for Renderer2DModule {
    fn dependencies(app: &mut App)
    where
        Self: Sized,
    {
        if !app.has_module::<comet_assets::AssetModule>() {
            app.add_module(comet_assets::AssetModule::new());
        }
    }
    fn build(&mut self, app: &mut App) {
        if !app.has_module::<comet_window::winit_module::WinitModule>() {
            return;
        }
        app.get_module_mut::<comet_window::winit_module::WinitModule>()
            .set_renderer_factory(Box::new(|window, clear_color| {
                let (cmd_tx, cmd_rx) = flume::unbounded::<Renderer2DCommand>();
                let (evt_tx, evt_rx) = flume::unbounded::<Renderer2DEvent>();
                let frame_mailbox = Arc::new(Mutex::new(None));

                let renderer = Renderer2D::new(window, clear_color, evt_tx);
                let handle =
                    RenderHandle2D::with_frame_mailbox(cmd_tx, evt_rx, frame_mailbox.clone());

                let erased: Box<dyn comet_window::ErasedRenderer> = Box::new(ErasedRenderer2D {
                    renderer,
                    cmd_rx,
                    frame_mailbox,
                });
                let add_handle: Box<dyn FnOnce(&mut comet_app::App) + Send> =
                    Box::new(move |app| {
                        app.add_module(handle);
                    });

                (erased, add_handle)
            }));
    }
}
