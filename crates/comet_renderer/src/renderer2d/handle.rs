use super::*;

pub struct RenderHandle2D {
    command_sender: flume::Sender<Renderer2DCommand>,
    event_receiver: flume::Receiver<Renderer2DEvent>,
    frame_mailbox: FrameMailbox2D,
    last_size: Option<PhysicalSize<u32>>,
    pending_atlas_rebuild: bool,
    pending_frame_times: Vec<f32>,
    gizmo_buffer: GizmoBuffer,
    gizmo_registry: GizmoRegistry,
}

#[module]
impl RenderHandle2D {
    fn resolve_atlas_ref(
        &mut self,
        path: AssetPath,
    ) -> Option<(AtlasRef, Option<comet_assets::Asset<comet_assets::Image>>)> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::ResolveAtlasRef(path));
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::AtlasRef(..))
        })
        .and_then(|event| match event {
            Renderer2DEvent::AtlasRef(Some(atlas_ref), image_handle) => {
                Some((atlas_ref, image_handle))
            }
            _ => None,
        })
    }

    fn ensure_handle_in_atlas(
        &mut self,
        handle: comet_assets::Asset<comet_assets::Image>,
    ) -> Option<AtlasRef> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::EnsureHandleInAtlas(handle));
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::AtlasRef(..))
        })
        .and_then(|event| match event {
            Renderer2DEvent::AtlasRef(atlas_ref, _) => atlas_ref,
            _ => None,
        })
    }

    pub fn size(&mut self) -> PhysicalSize<u32> {
        let _ = self.command_sender.send(Renderer2DCommand::Size);
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::Size(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::Size(size) => Some(size),
            _ => None,
        })
        .map(|size| {
            self.last_size = Some(size);
            size
        })
        .unwrap_or_else(|| self.last_size.unwrap_or(PhysicalSize::new(0, 0)))
    }

    pub fn scale_factor(&mut self) -> f64 {
        let _ = self.command_sender.send(Renderer2DCommand::ScaleFactor);
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::ScaleFactor(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::ScaleFactor(factor) => Some(factor),
            _ => None,
        })
        .unwrap_or(1.0)
    }

    pub fn precompute_text_bounds(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        font_size: impl Into<comet_math::ScreenUnit>,
    ) -> v2 {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::PrecomputedTextBounds {
                text: text.to_string(),
                font,
                font_size: font_size.into(),
            });
        self.recv_matching_event(Duration::from_secs(5), |event| {
            matches!(event, Renderer2DEvent::PrecomputedTextBounds { .. })
        })
        .and_then(|e| match e {
            Renderer2DEvent::PrecomputedTextBounds { width, height } => {
                Some(v2::new(width, height))
            }
            _ => None,
        })
        .unwrap_or(v2::ZERO)
    }

    pub fn poll_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                Renderer2DEvent::Size(size) => self.last_size = Some(size),
                Renderer2DEvent::AtlasRebuilt => self.pending_atlas_rebuild = true,
                Renderer2DEvent::FrameTime(frame_time) => self.pending_frame_times.push(frame_time),
                _ => {}
            }
        }
    }

    fn recv_matching_event<F>(&mut self, timeout: Duration, predicate: F) -> Option<Renderer2DEvent>
    where
        F: Fn(&Renderer2DEvent) -> bool,
    {
        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }

            match self.event_receiver.recv_timeout(remaining) {
                Ok(event) => {
                    match &event {
                        Renderer2DEvent::Size(size) => self.last_size = Some(*size),
                        Renderer2DEvent::AtlasRebuilt => self.pending_atlas_rebuild = true,
                        Renderer2DEvent::FrameTime(frame_time) => {
                            self.pending_frame_times.push(*frame_time)
                        }
                        _ => {}
                    }
                    if predicate(&event) {
                        return Some(event);
                    }
                }
                Err(flume::RecvTimeoutError::Timeout) => return None,
                Err(flume::RecvTimeoutError::Disconnected) => return None,
            }
        }
    }

    pub fn add_render_pass(
        &mut self,
        label: String,
        inputs: Vec<&PassOutput>,
        output: Option<String>,
        render_target: Option<&PassOutput>,
        output_format: Option<wgpu::TextureFormat>,
        shader_src: String,
        load: LoadOp,
    ) -> Option<PassOutput> {
        let desc = crate::render_commands::PassDescriptor {
            label,
            inputs: inputs.iter().map(|p| p.0.clone()).collect(),
            output,
            render_target: render_target.map(|p| p.0.clone()),
            output_format,
            shader_src,
            load,
        };
        let _ = self
            .command_sender
            .send(Renderer2DCommand::AddRenderPass(desc));
        self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassAdded(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::PassAdded(handle) => Some(handle),
            _ => None,
        })
    }

    pub fn remove_render_pass(&mut self, output: PassOutput) {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::RemoveRenderPass(output.0));
        let _ = self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassRemoved)
        });
    }

    pub fn set_pass_output(
        &mut self,
        label: &str,
        output: Option<PassOutput>,
    ) -> Option<PassOutput> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::SetPassOutput(label.to_string(), output));
        self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassOutputSet(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::PassOutputSet(handle) => handle,
            _ => None,
        })
    }

    pub fn set_pass_render_target(&mut self, label: &str, render_target: Option<&PassOutput>) {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::SetPassRenderTarget(
                label.to_string(),
                render_target.map(|p| p.0.clone()),
            ));
        let _ = self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassRenderTargetSet)
        });
    }

    pub fn show_gizmo<C: Component + Gizmo + 'static>(&mut self, entity: comet_ecs::Entity) {
        self.gizmo_registry.show::<C>(entity);
    }

    pub fn hide_gizmo<C: Component + Gizmo + 'static>(&mut self, entity: comet_ecs::Entity) {
        self.gizmo_registry.hide::<C>(entity);
    }

    pub fn show_all_gizmos<C: Component + Gizmo + 'static>(&mut self) {
        self.gizmo_registry.show_all::<C>();
    }

    pub fn hide_all_gizmos<C: Component + Gizmo + 'static>(&mut self) {
        self.gizmo_registry.hide_all::<C>();
    }
}

impl RenderHandle2D {
    pub fn render_scene_2d(&mut self, scene: &mut comet_ecs::Scene) {
        self.poll_events();
        if self.pending_atlas_rebuild {
            self.pending_atlas_rebuild = false;
            for (_, render) in
                scene.query_mut::<(&comet_ecs::Transform, &mut comet_ecs::Sprite), ()>()
            {
                if let ImageRef::ResolvedHandle(h, _) = render.texture() {
                    render.set_image_ref(ImageRef::Handle(h));
                }
            }
        }

        let mut selected_camera: Option<(
            [f32; 2],
            f32,
            comet_ecs::Camera,
            comet_ecs::Projection,
            comet_ecs::Screen,
        )> = None;
        for (transform, camera, projection, screen) in scene.query::<(
            &comet_ecs::Transform,
            &comet_ecs::Camera,
            &comet_ecs::Projection,
            &comet_ecs::Screen,
        ), comet_ecs::With<comet_ecs::Camera2d>>(
        ) {
            if !camera.is_enabled() {
                continue;
            }
            let should_replace = selected_camera
                .as_ref()
                .is_none_or(|(_, _, current, _, _)| camera.priority() > current.priority());
            if should_replace {
                selected_camera = Some((
                    [transform.position().x(), transform.position().y()],
                    transform.rotation().as_degrees().z(),
                    *camera,
                    *projection,
                    screen.clone(),
                ));
            }
        }
        let Some((camera_pos, camera_rot, camera, projection, screen)) = selected_camera else {
            return;
        };

        let mut draws = Vec::new();
        let mut referenced_handles = Vec::new();
        for (transform, render) in
            scene.query_mut::<(&comet_ecs::Transform, &mut comet_ecs::Sprite), ()>()
        {
            if !render.is_visible() {
                continue;
            }

            let atlas_ref = match render.texture() {
                ImageRef::Atlas(atlas_ref) => atlas_ref,
                ImageRef::Unresolved(path) => {
                    let Some((atlas_ref, image_handle)) = self.resolve_atlas_ref(path) else {
                        continue;
                    };
                    if let Some(handle) = image_handle {
                        render.set_image_ref(ImageRef::ResolvedHandle(handle, atlas_ref));
                        referenced_handles.push(handle);
                    } else {
                        render.set_image_ref(ImageRef::Atlas(atlas_ref));
                    }
                    atlas_ref
                }
                ImageRef::Handle(handle) => {
                    let Some(atlas_ref) = self.ensure_handle_in_atlas(handle) else {
                        continue;
                    };
                    render.set_image_ref(ImageRef::ResolvedHandle(handle, atlas_ref));
                    referenced_handles.push(handle);
                    atlas_ref
                }
                ImageRef::ResolvedHandle(handle, atlas_ref) => {
                    referenced_handles.push(handle);
                    atlas_ref
                }
            };

            draws.push(Draw2D {
                position: [transform.position().x(), transform.position().y()],
                rotation_deg: transform.rotation().as_degrees().z(),
                scale: [transform.scale().x(), transform.scale().y()],
                texture: atlas_ref,
                draw_index: render.draw_index(),
                visible: true,
            });
        }

        let mut texts = Vec::new();
        for (transform, text, layout) in scene.query::<(
            &comet_ecs::Transform,
            &comet_ecs::Text,
            Option<&comet_ecs::TextLayout>,
        ), comet_ecs::Without<comet_ecs::ScreenPosition>>(
        ) {
            if !text.is_visible() {
                continue;
            }
            let color = text.color().to_wgpu();
            let anchor = layout.map_or(comet_ecs::Anchor::TopLeft, |layout| layout.anchor());
            let justification = layout.map_or(comet_ecs::TextJustification::Left, |layout| {
                layout.justification()
            });
            texts.push(Text2D {
                position: [transform.position().x(), transform.position().y()],
                anchor,
                justification,
                content: text.content().to_string(),
                font: text.font(),
                size: text.font_size(),
                color: [
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    color.a as f32,
                ],
                visible: true,
            });
        }

        let virtual_resolution = screen.virtual_resolution();
        let mut screen_texts = Vec::new();
        for (position, text, layout) in scene.query::<(
            &comet_ecs::ScreenPosition,
            &comet_ecs::Text,
            Option<&comet_ecs::TextLayout>,
        ), comet_ecs::Without<comet_ecs::Transform>>()
        {
            if !text.is_visible() {
                continue;
            }
            let color = text.color().to_wgpu();
            let text_anchor = layout.map_or(comet_ecs::Anchor::TopLeft, |layout| layout.anchor());
            let justification = layout.map_or(comet_ecs::TextJustification::Left, |layout| {
                layout.justification()
            });
            screen_texts.push(ScreenText2D {
                anchor: position.anchor(),
                offset: [position.offset().x(), position.offset().y()],
                text_anchor,
                justification,
                content: text.content().to_string(),
                font: text.font(),
                size: text.font_size(),
                color: [
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    color.a as f32,
                ],
                visible: true,
            });
        }

        let camera_packet = CameraPacket2D {
            position: camera_pos,
            rotation_deg: camera_rot,
            priority: camera.priority(),
            projection,
            virtual_resolution,
            resolution_scaling: screen.resolution_scaling(),
            magnification: projection.magnification(),
            viewport: screen.viewport(),
        };

        self.gizmo_registry.flush(scene, &mut self.gizmo_buffer);
        let gizmo_shapes = std::mem::take(&mut self.gizmo_buffer.shapes);

        let replaced_frame = self.frame_mailbox.lock().unwrap().replace(FramePacket2D {
            camera: camera_packet,
            draws,
            texts,
            screen_texts,
            referenced_handles,
            gizmo_shapes,
        });
        drop(replaced_frame);
    }
}

impl RenderHandle2D {
    pub(super) fn with_frame_mailbox(
        command_sender: flume::Sender<Renderer2DCommand>,
        event_receiver: flume::Receiver<Renderer2DEvent>,
        frame_mailbox: FrameMailbox2D,
    ) -> Self {
        Self {
            command_sender,
            event_receiver,
            frame_mailbox,
            last_size: None,
            pending_atlas_rebuild: false,
            pending_frame_times: Vec::new(),
            gizmo_buffer: GizmoBuffer::new(),
            gizmo_registry: GizmoRegistry::new(),
        }
    }
}

impl RendererHandle for RenderHandle2D {
    type Command = Renderer2DCommand;
    type Event = Renderer2DEvent;

    fn new(sender: flume::Sender<Self::Command>, receiver: flume::Receiver<Self::Event>) -> Self {
        Self::with_frame_mailbox(sender, receiver, Arc::new(Mutex::new(None)))
    }

    fn poll_event(&self) -> Option<Renderer2DEvent> {
        self.event_receiver.try_recv().ok()
    }
}

impl comet_app::Module for RenderHandle2D {
    fn dependencies(app: &mut comet_app::App)
    where
        Self: Sized,
    {
        if !app.has_module::<comet_assets::AssetModule>() {
            app.add_module(comet_assets::AssetModule::new());
        }
        if !app.has_module::<comet_ecs::EcsModule>() {
            app.add_module(comet_ecs::EcsModule::new());
        }
    }
    fn build(&mut self, app: &mut comet_app::App) {
        app.add_post_tick_hook(|app| {
            let mut renderer = app.take_module::<RenderHandle2D>().unwrap();
            renderer.render_scene_2d(app.scene_mut());
            for frame_time in renderer.pending_frame_times.drain(..) {
                app.record_render_frame_time(frame_time);
            }
            app.reinsert_module(renderer);
        });
    }
}
