use super::*;

impl Renderer2D {
    pub fn submit_frame(
        &mut self,
        #[cfg(feature = "diagnostics")] snapshot_sequence: u64,
        #[cfg(feature = "diagnostics")] snapshot_produced_at: Instant,
        #[cfg(feature = "diagnostics")] replaced_snapshots: u64,
        camera: CameraPacket2D,
        mut draws: Vec<Draw2D>,
        texts: Vec<Text2D>,
        screen_texts: Vec<ScreenText2D>,
        referenced_handles: Vec<comet_assets::Asset<comet_assets::Image>>,
        gizmo_shapes: Vec<GizmoShape>,
    ) {
        #[cfg(feature = "diagnostics")]
        {
            self.latest_snapshot_sequence = Some(snapshot_sequence);
            self.latest_snapshot_produced_at = Some(snapshot_produced_at);
            self.frame_diagnostics.replaced_snapshots = replaced_snapshots;
            self.frame_diagnostics.uploaded_bytes = 0;
        }

        if let Some(atlas_handle) = self
            .render_state
            .resources()
            .get_asset_atlas_handle("atlas")
        {
            let any_evicted = self
                .asset_provider
                .with_mut(atlas_handle, |atlas| {
                    let mut evicted = false;
                    for handle in &referenced_handles {
                        if atlas.region_for_handle(*handle).is_some() {
                            atlas.mark_used(*handle);
                        } else {
                            evicted = true;
                        }
                    }
                    atlas.evict_stale(120);
                    evicted
                })
                .unwrap_or(false);
            if any_evicted {
                let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
            }
        }
        let (world_view, screen_view) = self.resolve_camera_views(camera);
        draws.sort_by_key(|draw| draw.draw_index);

        let mut sprite_instances = std::mem::take(&mut self.sprite_instance_staging);
        sprite_instances.clear();
        sprite_instances.reserve(draws.len());
        for draw in draws {
            if !draw.visible {
                continue;
            }

            let region = self.get_texture_region(draw.texture);
            let (width, height) = region.dimensions();
            sprite_instances.push(SpriteInstance::new(
                draw.position,
                [
                    width as f32 * 0.5 * draw.scale[0],
                    height as f32 * 0.5 * draw.scale[1],
                ],
                draw.rotation_deg.to_radians(),
                [region.u0(), region.v0(), region.u1(), region.v1()],
                [1.0; 4],
            ));
        }

        let sprite_instances_changed = sprite_instances != self.sprite_instances;
        let device = self.render_state.device();
        let queue = self.render_state.queue();

        if sprite_instances_changed {
            if let Some(node) = self.graph.pass_mut("Universal") {
                let instance_count = sprite_instances.len() as u32;
                let update_result = node
                    .write_vertex_stream(1, &sprite_instances, device, queue)
                    .and_then(|()| {
                        node.set_draw_command(DrawCommand::Indexed {
                            indices: 0..6,
                            base_vertex: 0,
                            instances: 0..instance_count,
                        })
                    });
                if let Err(error) = update_result {
                    error!("Failed to update sprite draw batch: {}", error);
                } else {
                    #[cfg(feature = "diagnostics")]
                    {
                        self.frame_diagnostics.uploaded_bytes +=
                            std::mem::size_of_val(sprite_instances.as_slice()) as u64;
                    }
                }
            }
        }
        if sprite_instances_changed {
            self.sprite_instance_staging =
                std::mem::replace(&mut self.sprite_instances, sprite_instances);
        } else {
            self.sprite_instance_staging = sprite_instances;
        }

        let mut font_vertex_buffer = std::mem::take(&mut self.world_text_staging_vertices);
        let mut font_index_buffer = std::mem::take(&mut self.world_text_staging_indices);
        font_vertex_buffer.clear();
        font_index_buffer.clear();

        for text in texts {
            if !text.visible {
                continue;
            }

            let position = v2::new(text.position[0], text.position[1]);
            let color = wgpu::Color {
                r: text.color[0] as f64,
                g: text.color[1] as f64,
                b: text.color[2] as f64,
                a: text.color[3] as f64,
            };

            let (raster_size, geometry_scale) = self.resolve_text_size(text.size, world_view);
            let mut bounds = v2::ZERO;
            self.add_text_to_buffers(
                &text.content,
                text.font,
                raster_size,
                geometry_scale,
                position,
                color,
                text.anchor,
                text.justification,
                &mut bounds,
                &mut font_vertex_buffer,
                &mut font_index_buffer,
            );
        }

        let world_text_changed = font_vertex_buffer != self.world_text_vertices
            || font_index_buffer != self.world_text_indices;
        if world_text_changed {
            let device = self.render_state.device();
            let queue = self.render_state.queue();
            if let Some(node) = self.graph.pass_mut("Font") {
                if let Err(error) =
                    node.set_geometry(&font_vertex_buffer, &font_index_buffer, device, queue)
                {
                    error!("Failed to update font draw batch: {}", error);
                } else {
                    #[cfg(feature = "diagnostics")]
                    {
                        self.frame_diagnostics.uploaded_bytes +=
                            (std::mem::size_of_val(font_vertex_buffer.as_slice())
                                + std::mem::size_of_val(font_index_buffer.as_slice()))
                                as u64;
                    }
                }
            }
        }
        if world_text_changed {
            self.world_text_staging_vertices =
                std::mem::replace(&mut self.world_text_vertices, font_vertex_buffer);
            self.world_text_staging_indices =
                std::mem::replace(&mut self.world_text_indices, font_index_buffer);
        } else {
            self.world_text_staging_vertices = font_vertex_buffer;
            self.world_text_staging_indices = font_index_buffer;
        }

        let mut screen_font_vertex_buffer = std::mem::take(&mut self.screen_text_staging_vertices);
        let mut screen_font_index_buffer = std::mem::take(&mut self.screen_text_staging_indices);
        screen_font_vertex_buffer.clear();
        screen_font_index_buffer.clear();
        let screen_size = screen_view.visible_world_size;
        for text in screen_texts {
            if !text.visible {
                continue;
            }

            let half_width = screen_size.x() * 0.5;
            let half_height = screen_size.y() * 0.5;
            let anchor = match text.anchor {
                comet_ecs::Anchor::TopLeft => v2::new(-half_width, half_height),
                comet_ecs::Anchor::TopCenter => v2::new(0.0, half_height),
                comet_ecs::Anchor::TopRight => v2::new(half_width, half_height),
                comet_ecs::Anchor::CenterLeft => v2::new(-half_width, 0.0),
                comet_ecs::Anchor::Center => v2::ZERO,
                comet_ecs::Anchor::CenterRight => v2::new(half_width, 0.0),
                comet_ecs::Anchor::BottomLeft => v2::new(-half_width, -half_height),
                comet_ecs::Anchor::BottomCenter => v2::new(0.0, -half_height),
                comet_ecs::Anchor::BottomRight => v2::new(half_width, -half_height),
            };
            let position = anchor + v2::new(text.offset[0], -text.offset[1]);
            let color = wgpu::Color {
                r: text.color[0] as f64,
                g: text.color[1] as f64,
                b: text.color[2] as f64,
                a: text.color[3] as f64,
            };
            let (raster_size, geometry_scale) = self.resolve_text_size(text.size, screen_view);
            let mut bounds = v2::ZERO;
            self.add_text_to_buffers(
                &text.content,
                text.font,
                raster_size,
                geometry_scale,
                position,
                color,
                text.text_anchor,
                text.justification,
                &mut bounds,
                &mut screen_font_vertex_buffer,
                &mut screen_font_index_buffer,
            );
        }

        let screen_text_changed = screen_font_vertex_buffer != self.screen_text_vertices
            || screen_font_index_buffer != self.screen_text_indices;
        let screen_camera = RenderCamera::new(screen_size, v3::ZERO);
        let mut screen_uniform = CameraUniform::new();
        screen_uniform.update_view_proj(&screen_camera);
        let device = self.render_state.device();
        let queue = self.render_state.queue();
        if let Some(node) = self.graph.pass_mut("ScreenFont") {
            if screen_text_changed {
                if let Err(error) = node.set_geometry(
                    &screen_font_vertex_buffer,
                    &screen_font_index_buffer,
                    device,
                    queue,
                ) {
                    error!("Failed to update screen font draw batch: {}", error);
                } else {
                    #[cfg(feature = "diagnostics")]
                    {
                        self.frame_diagnostics.uploaded_bytes +=
                            (std::mem::size_of_val(screen_font_vertex_buffer.as_slice())
                                + std::mem::size_of_val(screen_font_index_buffer.as_slice()))
                                as u64;
                    }
                }
            }
            node.set_camera(&screen_uniform, queue);
            node.set_viewport(Some(screen_view.viewport));
        }
        if screen_text_changed {
            self.screen_text_staging_vertices =
                std::mem::replace(&mut self.screen_text_vertices, screen_font_vertex_buffer);
            self.screen_text_staging_indices =
                std::mem::replace(&mut self.screen_text_indices, screen_font_index_buffer);
        } else {
            self.screen_text_staging_vertices = screen_font_vertex_buffer;
            self.screen_text_staging_indices = screen_font_index_buffer;
        }

        // Text processing lazily creates the Font pass, so apply camera uniforms afterward.
        self.apply_camera_view(camera, world_view);

        let mut gizmo_verts = std::mem::take(&mut self.gizmo_staging_vertices);
        let mut gizmo_indices = std::mem::take(&mut self.gizmo_staging_indices);
        gizmo_verts.clear();
        gizmo_indices.clear();

        for shape in gizmo_shapes {
            match shape {
                GizmoShape::Line { start, end, color } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let base = gizmo_verts.len() as u16;
                    gizmo_verts.push(Vertex::new(
                        [start.x(), start.y(), start.z()],
                        [0.0, 0.0],
                        c,
                    ));
                    gizmo_verts.push(Vertex::new([end.x(), end.y(), end.z()], [0.0, 0.0], c));
                    gizmo_indices.extend_from_slice(&[base, base + 1]);
                }
                GizmoShape::Rect {
                    position,
                    size,
                    color,
                } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let hx = size.x() * 0.5;
                    let hy = size.y() * 0.5;
                    let base = gizmo_verts.len() as u16;
                    let corners = [
                        [position.x() - hx, position.y() + hy, position.z()],
                        [position.x() + hx, position.y() + hy, position.z()],
                        [position.x() + hx, position.y() - hy, position.z()],
                        [position.x() - hx, position.y() - hy, position.z()],
                    ];
                    for corner in &corners {
                        gizmo_verts.push(Vertex::new(*corner, [0.0, 0.0], c));
                    }
                    gizmo_indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 1,
                        base + 2,
                        base + 2,
                        base + 3,
                        base + 3,
                        base,
                    ]);
                }
                GizmoShape::Circle {
                    position,
                    radius,
                    color,
                } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let segments = 32u32;
                    let base = gizmo_verts.len() as u16;
                    for i in 0..segments {
                        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                        let x = position.x() + radius * angle.cos();
                        let y = position.y() + radius * angle.sin();
                        gizmo_verts.push(Vertex::new([x, y, position.z()], [0.0, 0.0], c));
                        let next = (i + 1) % segments;
                        gizmo_indices.extend_from_slice(&[base + i as u16, base + next as u16]);
                    }
                }
                GizmoShape::NGon {
                    position,
                    radius,
                    vertices,
                    color,
                } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let n = vertices.max(3);
                    let base = gizmo_verts.len() as u16;
                    for i in 0..n {
                        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
                        let x = position.x() + radius * angle.cos();
                        let y = position.y() + radius * angle.sin();
                        gizmo_verts.push(Vertex::new([x, y, position.z()], [0.0, 0.0], c));
                        let next = (i + 1) % n;
                        gizmo_indices.extend_from_slice(&[base + i as u16, base + next as u16]);
                    }
                }
            }
        }

        let gizmos_changed =
            gizmo_verts != self.gizmo_vertices || gizmo_indices != self.gizmo_indices;
        let device = self.render_state.device();
        let queue = self.render_state.queue();

        if gizmos_changed {
            if let Some(node) = self.graph.pass_mut("Gizmo") {
                if let Err(error) = node.set_geometry(&gizmo_verts, &gizmo_indices, device, queue) {
                    error!("Failed to update gizmo draw batch: {}", error);
                } else {
                    #[cfg(feature = "diagnostics")]
                    {
                        self.frame_diagnostics.uploaded_bytes +=
                            (std::mem::size_of_val(gizmo_verts.as_slice())
                                + std::mem::size_of_val(gizmo_indices.as_slice()))
                                as u64;
                    }
                }
            }
        }
        if gizmos_changed {
            self.gizmo_staging_vertices = std::mem::replace(&mut self.gizmo_vertices, gizmo_verts);
            self.gizmo_staging_indices = std::mem::replace(&mut self.gizmo_indices, gizmo_indices);
        } else {
            self.gizmo_staging_vertices = gizmo_verts;
            self.gizmo_staging_indices = gizmo_indices;
        }

        #[cfg(feature = "diagnostics")]
        {
            self.frame_diagnostics.sprite_instances = self.sprite_instances.len() as u32;
            self.frame_diagnostics.glyphs =
                ((self.world_text_indices.len() + self.screen_text_indices.len()) / 6) as u32;
        }
    }

    fn resolve_camera_views(
        &self,
        camera: CameraPacket2D,
    ) -> (
        crate::camera::ResolvedCameraViewport,
        crate::camera::ResolvedCameraViewport,
    ) {
        let output_bounds = if let Some(viewport) = camera.viewport {
            let x = (viewport.x().pixels().floor() as u32)
                .min(self.render_state.config().width.saturating_sub(1));
            let y = (viewport.y().pixels().floor() as u32)
                .min(self.render_state.config().height.saturating_sub(1));
            let width = viewport.width().pixels().round().max(1.0) as u32;
            let height = viewport.height().pixels().round().max(1.0) as u32;
            ResolvedViewport {
                x,
                y,
                width: width.min(self.render_state.config().width - x),
                height: height.min(self.render_state.config().height - y),
            }
        } else {
            ResolvedViewport {
                x: 0,
                y: 0,
                width: self.render_state.config().width,
                height: self.render_state.config().height,
            }
        };
        let scale_factor = self.scale_factor() as f32;
        let virtual_resolution = camera
            .virtual_resolution
            .map(|size| size.resolve(scale_factor))
            .unwrap_or_else(|| {
                v2::new(
                    output_bounds.width as f32 / scale_factor,
                    output_bounds.height as f32 / scale_factor,
                )
            });
        let resolved = resolve_camera_viewport(
            virtual_resolution,
            camera.resolution_scaling,
            camera.magnification,
            output_bounds,
        );
        let screen_view = resolve_camera_viewport(
            virtual_resolution,
            camera.resolution_scaling,
            1.0,
            output_bounds,
        );

        (resolved, screen_view)
    }

    fn apply_camera_view(
        &mut self,
        camera: CameraPacket2D,
        resolved: crate::camera::ResolvedCameraViewport,
    ) {
        let view_proj: [[f32; 4]; 4] = match camera.projection {
            comet_ecs::Projection::Custom(matrix) => matrix.into(),
            _ => RenderCamera::new(
                resolved.visible_world_size,
                v3::new(camera.position[0], camera.position[1], 0.0),
            )
            .build_view_projection_matrix()
            .into(),
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.set_view_proj(view_proj);

        let queue = self.render_state.queue();

        if let Some(node) = self.graph.pass_mut("Universal") {
            node.set_camera(&camera_uniform, queue);
            node.set_viewport(Some(resolved.viewport));
        }
        if let Some(node) = self.graph.pass_mut("Font") {
            node.set_camera(&camera_uniform, queue);
            node.set_viewport(Some(resolved.viewport));
        }
        if let Some(node) = self.graph.pass_mut("Gizmo") {
            node.set_camera(&camera_uniform, queue);
            node.set_viewport(Some(resolved.viewport));
        }
    }
}
