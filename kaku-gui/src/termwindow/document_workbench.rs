use super::box_model::*;
use super::keyevent::Key;
use super::{TermWindow, UIItem, UIItemType};
use crate::document_workbench::{EVENT_NAME, WorkbenchHit, WorkbenchView};
use crate::utilsprites::RenderMetrics;
use config::keyassignment::ClipboardCopyDestination;
use config::{Dimension, DimensionContext};
use mux::pane::{CachePolicy, Pane};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use window::color::LinearRgba;
use window::{KeyEvent, MouseCursor, MouseEvent, MouseEventKind, MousePress, RectF, WindowOps};

const PANEL_MIN_WIDTH: usize = 420;
const PANEL_MAX_RATIO: f32 = 0.58;
const RESIZE_HANDLE_WIDTH: f32 = 5.0;

#[derive(Clone, Copy)]
struct WorkbenchTheme {
    bg: LinearRgba,
    elevated: LinearRgba,
    selected: LinearRgba,
    border: LinearRgba,
    fg: LinearRgba,
    dim: LinearRgba,
    accent: LinearRgba,
    warn: LinearRgba,
}

impl TermWindow {
    pub(crate) fn toggle_document_workbench(&mut self, pane: &Arc<dyn Pane>) {
        if let Some(cwd) = pane_cwd(pane) {
            self.document_workbench.toggle(&cwd);
        } else {
            self.document_workbench
                .show_unavailable("Document Workbench supports local file panes only");
        }
        if let Some(window) = self.window.as_ref() {
            window.invalidate();
        }
    }

    pub(crate) fn document_workbench_blur(&mut self) {
        if self.document_workbench.focused {
            self.document_workbench.focused = false;
            if let Some(window) = self.window.as_ref() {
                window.invalidate();
            }
        }
    }

    pub(crate) fn document_workbench_key_event(
        &mut self,
        window_key: &KeyEvent,
        context: &dyn WindowOps,
    ) -> bool {
        if !self.document_workbench.visible || !self.document_workbench.focused {
            return false;
        }
        if !window_key.key_is_down {
            return true;
        }

        let mods = window_key.modifiers.remove_positional_mods();
        let handled = match self.win_key_code_to_termwiz_key_code(&window_key.key) {
            Key::Code(key) => self.document_workbench.handle_key(key, mods),
            Key::Composed(text) => {
                let mut any = false;
                for ch in text.chars() {
                    any |= self
                        .document_workbench
                        .handle_key(termwiz::input::KeyCode::Char(ch), mods);
                }
                any
            }
            Key::None => true,
        };

        if handled {
            context.invalidate();
        }
        handled
    }

    pub(crate) fn mouse_event_document_workbench(
        &mut self,
        hit: WorkbenchHit,
        item: UIItem,
        pane: Arc<dyn Pane>,
        event: MouseEvent,
        context: &dyn WindowOps,
    ) {
        if matches!(hit, WorkbenchHit::ResizeHandle) {
            context.set_cursor(Some(MouseCursor::SizeLeftRight));
        } else {
            context.set_cursor(Some(MouseCursor::Arrow));
        }

        if event.kind != MouseEventKind::Press(MousePress::Left) {
            return;
        }

        self.document_workbench.focused = true;
        let cwd = pane_cwd(&pane);
        match hit {
            WorkbenchHit::Panel | WorkbenchHit::Editor => {}
            WorkbenchHit::ResizeHandle => {
                self.dragging.replace((item, event));
            }
            WorkbenchHit::Close => {
                self.document_workbench.visible = false;
                self.document_workbench.focused = false;
            }
            WorkbenchHit::CopyPath => {
                if let Some(path) = self.document_workbench.current_path() {
                    self.copy_to_clipboard(
                        ClipboardCopyDestination::Clipboard,
                        path.display().to_string(),
                    );
                }
                self.document_workbench.copy_path();
            }
            WorkbenchHit::OpenExternal => self.document_workbench.open_external(),
            WorkbenchHit::Save => self.document_workbench.save(),
            WorkbenchHit::Discard => self.document_workbench.discard(),
            WorkbenchHit::Reload => {
                if let Some(cwd) = cwd.as_deref() {
                    self.document_workbench.refresh(cwd);
                } else {
                    self.document_workbench
                        .show_unavailable("Document Workbench supports local file panes only");
                }
            }
            WorkbenchHit::View(view) => self.document_workbench.view = view,
            WorkbenchHit::SelectDocument(idx) => {
                if let Some(cwd) = cwd.as_deref() {
                    self.document_workbench.select(idx, cwd);
                } else {
                    self.document_workbench
                        .show_unavailable("Document Workbench supports local file panes only");
                }
            }
        }
        context.invalidate();
    }

    pub(crate) fn drag_document_workbench_resize(
        &mut self,
        item: UIItem,
        start_event: MouseEvent,
        event: MouseEvent,
        context: &dyn WindowOps,
    ) {
        let left_px = event.coords.x.max(0) as usize;
        let desired = self.dimensions.pixel_width.saturating_sub(left_px);
        self.document_workbench.width_px = self.clamp_document_workbench_width(desired);
        if let Err(err) = crate::document_workbench::persist_width(self.document_workbench.width_px)
        {
            log::debug!("failed to persist document workbench width: {err:#}");
        }
        self.dragging.replace((item, start_event));
        context.invalidate();
    }

    pub(crate) fn paint_document_workbench(&mut self) -> anyhow::Result<()> {
        if !self.document_workbench.visible {
            return Ok(());
        }

        self.document_workbench.width_px =
            self.clamp_document_workbench_width(self.document_workbench.width_px);

        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let theme = self.document_workbench_theme();
        let border = self.get_os_border();
        let tab_bar_height = if self.show_tab_bar {
            self.tab_bar_pixel_height().unwrap_or(0.0)
        } else {
            0.0
        };
        let top = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            tab_bar_height
        } else {
            0.0
        } + border.top.get() as f32;
        let bottom = if self.show_tab_bar && self.config.tab_bar_at_bottom {
            tab_bar_height
        } else {
            0.0
        } + border.bottom.get() as f32;
        let height = (self.dimensions.pixel_height as f32 - top - bottom).max(120.0);
        let width = self.document_workbench.width_px as f32;
        let x = (self.dimensions.pixel_width as f32 - width - border.right.get() as f32).max(0.0);

        let mut ui_items = Vec::new();
        let handle = self.compute_workbench_element(
            &font,
            metrics,
            euclid::rect(x - RESIZE_HANDLE_WIDTH, top, RESIZE_HANDLE_WIDTH, height),
            workbench_handle_element(&font, theme),
            80,
        )?;
        ui_items.extend(handle.ui_items());

        let panel = self.compute_workbench_element(
            &font,
            metrics,
            euclid::rect(x, top, width, height),
            self.workbench_panel_element(&font, theme, width as usize, height as usize),
            81,
        )?;
        ui_items.extend(panel.ui_items());

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&handle, gl_state, None)?;
        self.render_element(&panel, gl_state, None)?;
        self.ui_items.append(&mut ui_items);
        Ok(())
    }

    fn clamp_document_workbench_width(&self, desired: usize) -> usize {
        let max = ((self.dimensions.pixel_width as f32) * PANEL_MAX_RATIO)
            .round()
            .max(PANEL_MIN_WIDTH as f32) as usize;
        desired.clamp(PANEL_MIN_WIDTH.min(max), max)
    }

    fn compute_workbench_element(
        &self,
        _font: &Rc<wezterm_font::LoadedFont>,
        metrics: RenderMetrics,
        bounds: RectF,
        element: Element,
        zindex: i8,
    ) -> anyhow::Result<ComputedElement> {
        self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: bounds.height(),
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: bounds.width(),
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds,
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex,
            },
            &element,
        )
    }

    fn workbench_panel_element(
        &self,
        font: &Rc<wezterm_font::LoadedFont>,
        theme: WorkbenchTheme,
        width_px: usize,
        height_px: usize,
    ) -> Element {
        let cell_width = self.render_metrics.cell_size.width.max(1) as usize;
        let cell_height = self.render_metrics.cell_size.height.max(1) as usize;
        let cols = (width_px / cell_width).max(42);
        let rows = (height_px / cell_height).max(14);
        let content_cols = cols.saturating_sub(4);
        let state = &self.document_workbench;
        let mut children = Vec::new();

        children.push(row(
            font,
            vec![
                label(font, "Document Workbench", theme.fg, true),
                action(font, "Close", WorkbenchHit::Close, theme),
            ],
            WorkbenchHit::Panel,
            theme.bg,
        ));
        let dirty = if state.is_dirty() { "dirty" } else { "saved" };
        children.push(row(
            font,
            vec![
                label(
                    font,
                    &format!(
                        "{} docs - {} - {}",
                        state.documents.len(),
                        dirty,
                        if state.focused {
                            "focused"
                        } else {
                            "terminal focus"
                        }
                    ),
                    theme.dim,
                    false,
                ),
                action(font, "Save", WorkbenchHit::Save, theme),
                action(font, "Copy Path", WorkbenchHit::CopyPath, theme),
                action(font, "Open External", WorkbenchHit::OpenExternal, theme),
            ],
            WorkbenchHit::Panel,
            theme.bg,
        ));

        let max_docs = rows.saturating_div(4).clamp(4, 9);
        for (idx, doc) in state.documents.iter().take(max_docs).enumerate() {
            let selected = idx == state.selected;
            let marker = if selected { ">" } else { " " };
            let dirty_marker = if selected && state.is_dirty() {
                "*"
            } else {
                " "
            };
            let text = format!(
                "{}{} [{}] {} ({})",
                marker,
                dirty_marker,
                doc.kind.label(),
                crate::document_workbench::truncate(
                    &doc.display_path,
                    content_cols.saturating_sub(14)
                ),
                doc.source.label()
            );
            children.push(
                Element::new(font, ElementContent::Text(text))
                    .item_type(UIItemType::DocumentWorkbench(WorkbenchHit::SelectDocument(
                        idx,
                    )))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: if selected {
                            theme.selected.into()
                        } else {
                            theme.bg.into()
                        },
                        text: if selected {
                            theme.fg.into()
                        } else {
                            theme.dim.into()
                        },
                    })
                    .padding(cell_pad(0.45, 0.75))
                    .min_width(Some(Dimension::Percent(1.0)))
                    .display(DisplayType::Block),
            );
        }

        children.push(separator(font, theme));
        if let Some(doc) = state.current() {
            children.push(row(
                font,
                vec![
                    label(
                        font,
                        &crate::document_workbench::truncate(
                            &doc.display_path,
                            content_cols.saturating_sub(18),
                        ),
                        theme.fg,
                        true,
                    ),
                    action(font, "Discard", WorkbenchHit::Discard, theme),
                    action(font, "Reload", WorkbenchHit::Reload, theme),
                ],
                WorkbenchHit::Panel,
                theme.elevated,
            ));
        }

        children.push(row(
            font,
            vec![
                tab(font, WorkbenchView::Source, state.view, theme),
                tab(font, WorkbenchView::Preview, state.view, theme),
                tab(font, WorkbenchView::Split, state.view, theme),
            ],
            WorkbenchHit::Panel,
            theme.bg,
        ));

        let used_rows = children.len() + 2;
        let max_content_lines = rows.saturating_sub(used_rows).max(5);
        let content_lines = match state.view {
            WorkbenchView::Source => state.source_lines(content_cols, max_content_lines),
            WorkbenchView::Preview => state.preview_lines(content_cols, max_content_lines),
            WorkbenchView::Split => {
                let left_cols = content_cols / 2;
                let right_cols = content_cols.saturating_sub(left_cols + 3);
                let source = state.source_lines(left_cols, max_content_lines);
                let preview = state.preview_lines(right_cols, max_content_lines);
                let count = source.len().max(preview.len()).min(max_content_lines);
                (0..count)
                    .map(|idx| {
                        format!(
                            "{:<left_cols$} | {}",
                            source.get(idx).cloned().unwrap_or_default(),
                            preview.get(idx).cloned().unwrap_or_default(),
                            left_cols = left_cols
                        )
                    })
                    .collect()
            }
        };

        for line in content_lines {
            children.push(
                Element::new(font, ElementContent::Text(line))
                    .item_type(UIItemType::DocumentWorkbench(WorkbenchHit::Editor))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: theme.elevated.into(),
                        text: theme.fg.into(),
                    })
                    .padding(cell_pad(0.1, 0.75))
                    .min_width(Some(Dimension::Percent(1.0)))
                    .display(DisplayType::Block),
            );
        }

        let status_color = if state.status.contains("failed") || state.status.contains("refused") {
            theme.warn
        } else {
            theme.dim
        };
        children.push(
            Element::new(
                font,
                ElementContent::Text(crate::document_workbench::truncate(
                    &state.status,
                    content_cols,
                )),
            )
            .item_type(UIItemType::DocumentWorkbench(WorkbenchHit::Panel))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: theme.bg.into(),
                text: status_color.into(),
            })
            .padding(cell_pad(0.4, 0.75))
            .min_width(Some(Dimension::Percent(1.0)))
            .display(DisplayType::Block),
        );

        Element::new(font, ElementContent::Children(children))
            .item_type(UIItemType::DocumentWorkbench(WorkbenchHit::Panel))
            .colors(ElementColors {
                border: BorderColor::new(theme.border),
                bg: theme.bg.into(),
                text: theme.fg.into(),
            })
            .border(BoxDimension {
                left: Dimension::Pixels(1.0),
                top: Dimension::Pixels(0.0),
                right: Dimension::Pixels(0.0),
                bottom: Dimension::Pixels(0.0),
            })
            .padding(cell_pad(0.6, 0.6))
            .min_width(Some(Dimension::Pixels(width_px as f32)))
            .min_height(Some(Dimension::Pixels(height_px as f32)))
    }

    fn document_workbench_theme(&mut self) -> WorkbenchTheme {
        let palette = if let Some(pane) = self.get_active_pane_or_overlay() {
            pane.palette()
        } else {
            self.palette().clone()
        };
        WorkbenchTheme {
            bg: palette.background.to_linear(),
            elevated: palette.background.to_linear().mul_alpha(0.96),
            selected: palette.selection_bg.to_linear().mul_alpha(0.82),
            border: palette.colors.0[8].to_linear().mul_alpha(0.7),
            fg: palette.foreground.to_linear(),
            dim: palette.colors.0[8].to_linear(),
            accent: palette.colors.0[14].to_linear(),
            warn: palette.colors.0[11].to_linear(),
        }
    }
}

fn pane_cwd(pane: &Arc<dyn Pane>) -> Option<PathBuf> {
    pane.get_current_working_dir(CachePolicy::AllowStale)
        .and_then(|url| url.to_file_path().ok())
}

fn workbench_handle_element(font: &Rc<wezterm_font::LoadedFont>, theme: WorkbenchTheme) -> Element {
    Element::new(font, ElementContent::Text(String::new()))
        .item_type(UIItemType::DocumentWorkbench(WorkbenchHit::ResizeHandle))
        .colors(ElementColors {
            border: BorderColor::default(),
            bg: theme.border.into(),
            text: theme.border.into(),
        })
        .min_width(Some(Dimension::Percent(1.0)))
        .min_height(Some(Dimension::Percent(1.0)))
}

fn row(
    font: &Rc<wezterm_font::LoadedFont>,
    children: Vec<Element>,
    hit: WorkbenchHit,
    bg: LinearRgba,
) -> Element {
    Element::new(font, ElementContent::Children(children))
        .item_type(UIItemType::DocumentWorkbench(hit))
        .colors(ElementColors {
            border: BorderColor::default(),
            bg: bg.into(),
            text: bg.into(),
        })
        .padding(cell_pad(0.25, 0.6))
        .min_width(Some(Dimension::Percent(1.0)))
        .display(DisplayType::Block)
}

fn label(
    font: &Rc<wezterm_font::LoadedFont>,
    text: &str,
    color: LinearRgba,
    strong: bool,
) -> Element {
    Element::new(font, ElementContent::Text(text.to_string())).colors(ElementColors {
        border: BorderColor::default(),
        bg: LinearRgba::TRANSPARENT.into(),
        text: if strong {
            color.into()
        } else {
            color.mul_alpha(0.92).into()
        },
    })
}

fn action(
    font: &Rc<wezterm_font::LoadedFont>,
    text: &str,
    hit: WorkbenchHit,
    theme: WorkbenchTheme,
) -> Element {
    Element::new(font, ElementContent::Text(format!(" {} ", text)))
        .item_type(UIItemType::DocumentWorkbench(hit))
        .float(Float::Right)
        .colors(ElementColors {
            border: BorderColor::new(theme.border),
            bg: theme.elevated.into(),
            text: theme.fg.into(),
        })
        .border(BoxDimension::new(Dimension::Pixels(1.0)))
        .padding(cell_pad(0.15, 0.35))
}

fn tab(
    font: &Rc<wezterm_font::LoadedFont>,
    view: WorkbenchView,
    active: WorkbenchView,
    theme: WorkbenchTheme,
) -> Element {
    let is_active = view == active;
    Element::new(font, ElementContent::Text(format!(" {} ", view.label())))
        .item_type(UIItemType::DocumentWorkbench(WorkbenchHit::View(view)))
        .colors(ElementColors {
            border: BorderColor::new(if is_active {
                theme.accent
            } else {
                theme.border
            }),
            bg: if is_active {
                theme.selected.into()
            } else {
                theme.bg.into()
            },
            text: if is_active {
                theme.fg.into()
            } else {
                theme.dim.into()
            },
        })
        .border(BoxDimension {
            left: Dimension::Pixels(0.0),
            top: Dimension::Pixels(0.0),
            right: Dimension::Pixels(0.0),
            bottom: Dimension::Pixels(2.0),
        })
        .padding(cell_pad(0.2, 0.75))
}

fn separator(font: &Rc<wezterm_font::LoadedFont>, theme: WorkbenchTheme) -> Element {
    Element::new(font, ElementContent::Text(String::new()))
        .colors(ElementColors {
            border: BorderColor::new(theme.border),
            bg: theme.border.into(),
            text: theme.border.into(),
        })
        .min_height(Some(Dimension::Pixels(1.0)))
        .min_width(Some(Dimension::Percent(1.0)))
        .display(DisplayType::Block)
}

fn cell_pad(v: f32, h: f32) -> BoxDimension {
    BoxDimension {
        left: Dimension::Cells(h),
        right: Dimension::Cells(h),
        top: Dimension::Cells(v),
        bottom: Dimension::Cells(v),
    }
}

pub(crate) fn is_document_workbench_event(name: &str) -> bool {
    name == EVENT_NAME
}
