use std::{
    mem,
    sync::{Arc, Mutex, MutexGuard},
};

use pathfinder_color::ColorU;
use pathfinder_geometry::{
    rect::RectF,
    vector::{vec2f, Vector2F},
};

use crate::{
    event::DispatchedEvent, platform::Cursor, AfterLayoutContext, AppContext, Element,
    EventContext, PaintContext, SizeConstraint,
};

use super::{Fill, Point, ZIndex};

const DRAGBAR_WIDTH: f32 = 5.0;

/// A UI element with internal resizing ability.
///
/// This element takes a ResizableStateHandle to receive a starting size and
/// manage dimensions as the element is resized.
///
/// Supports both horizontal and vertical resizing via the ResizeDirection.
///
/// TODO:
///     - Take a configurable dragbar size instead of always using 5.0
pub struct Resizable {
    child: Box<dyn Element>,
    origin: Option<Vector2F>,
    dragbar: Dragbar,
    state_handle: ResizableStateHandle,
    size: Option<Vector2F>,
    bounds_callback: Option<BoundsCallback>,
    resize_handler: Option<Handler>,
    start_resize_handler: Option<Handler>,
    end_resize_handler: Option<Handler>,
    direction: ResizeDirection,
    origin_delta: Vector2F,
    dragbar_offset: f32,
}

type Handler = Box<dyn FnMut(&mut EventContext, &AppContext)>;
pub type BoundsCallback = Box<dyn FnMut(Vector2F) -> (f32, f32)>;

/// Similar to MouseStateHandle, the view that incorporates the owner can instantiate,
/// read, and set the state.
pub type ResizableStateHandle = Arc<Mutex<ResizableState>>;
pub fn resizable_state_handle(size: f32) -> ResizableStateHandle {
    Arc::new(Mutex::new(ResizableState::new(size)))
}

pub struct ResizableState {
    size: f32,
    hovering_dragbar: bool,
    bounds: Option<(f32, f32)>,
    mode: ResizableMode,
}

#[derive(Default)]
pub enum ResizableMode {
    Dragging {
        last_position: Vector2F,
    },
    #[default]
    Stationary,
}

impl ResizableState {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            hovering_dragbar: false,
            bounds: None,
            mode: Default::default(),
        }
    }
    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn clamp_size(&mut self) {
        if let Some((min, max)) = self.bounds {
            self.size = self.size.clamp(min, max);
        }
    }

    fn check_for_resize(
        &mut self,
        position: Vector2F,
        origin: Option<Vector2F>,
        dragbar_side: DragBarSide,
    ) -> Option<Vector2F> {
        if let ResizableMode::Dragging { last_position } = self.mode {
            self.resize(last_position, position, origin, dragbar_side)
        } else {
            None
        }
    }

    fn is_resizing(&self) -> bool {
        matches!(self.mode, ResizableMode::Dragging { .. })
    }

    fn resize(
        &mut self,
        old_position: Vector2F,
        new_position: Vector2F,
        origin: Option<Vector2F>,
        dragbar_side: DragBarSide,
    ) -> Option<Vector2F> {
        let mut resized = false;

        if origin.is_some() {
            let delta = match dragbar_side {
                DragBarSide::Right => new_position.x() - old_position.x(),
                DragBarSide::Left => old_position.x() - new_position.x(),
                DragBarSide::Bottom => new_position.y() - old_position.y(),
                DragBarSide::Top => old_position.y() - new_position.y(),
            };

            let old_size = self.size;
            if delta.abs() >= f32::EPSILON {
                resized = true;
                self.size += delta;
                self.clamp_size();
            }
            let size = self.size;

            // Preserve the initial grab offset, including when size bounds clamp movement.
            let applied_delta = size - old_size;
            let last_position = old_position
                + match dragbar_side {
                    DragBarSide::Right => vec2f(applied_delta, 0.),
                    DragBarSide::Left => vec2f(-applied_delta, 0.),
                    DragBarSide::Bottom => vec2f(0., applied_delta),
                    DragBarSide::Top => vec2f(0., -applied_delta),
                };

            let origin_delta = match dragbar_side {
                DragBarSide::Right => Vector2F::zero(),
                DragBarSide::Left => vec2f(old_size - size, 0.),
                DragBarSide::Bottom => Vector2F::zero(),
                DragBarSide::Top => vec2f(0., old_size - size),
            };

            self.mode = ResizableMode::Dragging { last_position };

            if resized {
                Some(origin_delta)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn begin_resizing(&mut self, position: Vector2F) {
        self.mode = ResizableMode::Dragging {
            last_position: position,
        };
    }

    pub fn end_resizing(&mut self) {
        self.mode = ResizableMode::Stationary;
    }

    pub fn set_size(&mut self, new_size: f32) {
        self.size = new_size;
    }
}

struct Dragbar {
    origin: Option<Point>,
    size: Option<Vector2F>,
    z_index: Option<ZIndex>,
    color: Fill,
    side: DragBarSide,
}

#[derive(Copy, Clone, Default)]
pub enum DragBarSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

#[derive(Copy, Clone, Default)]
pub enum ResizeDirection {
    #[default]
    Horizontal,
    Vertical,
}

impl Dragbar {
    pub fn new() -> Self {
        let color = Fill::Solid(ColorU::transparent_black());
        Self {
            origin: None,
            size: None,
            z_index: None,
            color,
            side: Default::default(),
        }
    }
}

impl Resizable {
    pub fn new(state_handle: ResizableStateHandle, child: Box<dyn Element>) -> Self {
        Self {
            child,
            origin: None,
            state_handle,
            size: None,
            bounds_callback: None,
            resize_handler: None,
            start_resize_handler: None,
            end_resize_handler: None,
            dragbar: Dragbar::new(),
            direction: ResizeDirection::Horizontal,
            origin_delta: Vector2F::zero(),
            dragbar_offset: 0.0,
        }
    }

    /// Adds a callback which will be called on a resize.
    /// Generally, this should trigger a re-render in the parent.
    pub fn on_resize<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&mut EventContext, &AppContext) + 'static,
    {
        self.resize_handler = Some(Box::new(callback));
        self
    }

    pub fn on_start_resizing<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&mut EventContext, &AppContext) + 'static,
    {
        self.start_resize_handler = Some(Box::new(callback));
        self
    }

    pub fn on_end_resizing<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&mut EventContext, &AppContext) + 'static,
    {
        self.end_resize_handler = Some(Box::new(callback));
        self
    }

    /// Sets a function that computes the (min, max) bounds on the width/height
    /// of the resizable. The bounds are updated at paint time.
    pub fn with_bounds_callback(mut self, callback: BoundsCallback) -> Self {
        self.bounds_callback = Some(callback);
        self
    }

    pub fn with_dragbar_color(mut self, color: Fill) -> Self {
        self.dragbar.color = color;
        self
    }

    pub fn with_dragbar_side(mut self, side: DragBarSide) -> Self {
        self.dragbar.side = side;
        // Automatically set direction based on side
        self.direction = match side {
            DragBarSide::Left | DragBarSide::Right => ResizeDirection::Horizontal,
            DragBarSide::Top | DragBarSide::Bottom => ResizeDirection::Vertical,
        };
        self
    }

    /// Sets an offset for the dragbar position.
    /// Positive values move the dragbar outwards (away from the center of the element).
    /// Negative values move the dragbar inwards (towards the center of the element).
    pub fn with_dragbar_offset(mut self, offset: f32) -> Self {
        self.dragbar_offset = offset;
        self
    }

    fn state(&mut self) -> MutexGuard<'_, ResizableState> {
        self.state_handle
            .lock()
            .expect("Resizable state should be accessible")
    }

    /// Determine if the mouse is hovering over the dragbar
    ///
    /// If there is another element above this one at the cursor position, then we treat that as
    /// outside the element for purposes of MouseState
    fn is_mouse_hovering_dragbar(&self, ctx: &EventContext, position: Vector2F) -> bool {
        let Some(dragbar_origin) = self.dragbar.origin else {
            log::warn!("self.origin was None in `Hoverable::is_mouse_in`");
            return false;
        };
        let Some(dragbar_size) = self.dragbar.size else {
            log::warn!("self.size() was None in `Hoverable::is_mouse_in`");
            return false;
        };
        let Some(z_index) = self.dragbar.z_index else {
            log::warn!("self.child_max_z_index was None in `Hoverable::is_mouse_in`");
            return false;
        };

        let is_hovering = ctx
            .visible_rect(dragbar_origin, dragbar_size)
            .is_some_and(|bound| bound.contains_point(position));

        let point = Point::from_vec2f(position, z_index);
        let is_covered = ctx.is_covered(point);

        is_hovering && !is_covered
    }
}

impl Element for Resizable {
    fn layout(
        &mut self,
        constraint: crate::SizeConstraint,
        ctx: &mut crate::LayoutContext,
        app: &AppContext,
    ) -> Vector2F {
        // Use the window size to set bounds on the width/height
        if let Some(bounds_callback) = self.bounds_callback.as_mut() {
            let mut new_bounds = bounds_callback(ctx.window_size);
            if new_bounds.0 > new_bounds.1 {
                log::error!("Resizable: min bound is greater than max bound");
                new_bounds = (new_bounds.0, new_bounds.0);
            }
            self.state().bounds = Some(new_bounds);

            // With new bounds, we should also clamp the current width/height.
            self.state().clamp_size();
        }

        let size = self.state().size;

        // We set the child constraints to never be greater than the current width/height constraint.
        let child_constraint = match self.direction {
            ResizeDirection::Horizontal => SizeConstraint {
                min: (constraint.min)
                    .max(Vector2F::zero())
                    .min(Vector2F::new(size, f32::MAX)),
                max: (constraint.max)
                    .max(Vector2F::zero())
                    .min(Vector2F::new(size, f32::MAX)),
            },
            ResizeDirection::Vertical => SizeConstraint {
                min: (constraint.min)
                    .max(Vector2F::zero())
                    .min(Vector2F::new(f32::MAX, size)),
                max: (constraint.max)
                    .max(Vector2F::zero())
                    .min(Vector2F::new(f32::MAX, size)),
            },
        };
        let child_size = self.child.layout(child_constraint, ctx, app);

        let size = child_size;
        self.size = Some(size);
        size
    }

    fn after_layout(&mut self, ctx: &mut AfterLayoutContext, app: &AppContext) {
        self.child.after_layout(ctx, app)
    }

    fn paint(&mut self, origin: Vector2F, ctx: &mut PaintContext, app: &AppContext) {
        self.child.paint(origin, ctx, app);

        // Draw the dragbar and record its size and position
        let child_size = self.child.size().unwrap();
        let (dragbar_origin, dragbar_size) = match self.dragbar.side {
            DragBarSide::Left => (
                origin - vec2f(self.dragbar_offset, 0.),
                vec2f(DRAGBAR_WIDTH, child_size.y()),
            ),
            DragBarSide::Right => (
                origin + vec2f(child_size.x() - DRAGBAR_WIDTH + self.dragbar_offset, 0.),
                vec2f(DRAGBAR_WIDTH, child_size.y()),
            ),
            DragBarSide::Top => (
                origin - vec2f(0., self.dragbar_offset),
                vec2f(child_size.x(), DRAGBAR_WIDTH),
            ),
            DragBarSide::Bottom => (
                origin + vec2f(0., child_size.y() - DRAGBAR_WIDTH + self.dragbar_offset),
                vec2f(child_size.x(), DRAGBAR_WIDTH),
            ),
        };

        ctx.scene
            .draw_rect_with_hit_recording(RectF::new(dragbar_origin, dragbar_size))
            .with_background(self.dragbar.color);

        self.dragbar.origin = Some(Point::from_vec2f(dragbar_origin, ctx.scene.z_index()));
        self.dragbar.size = Some(dragbar_size);
        self.dragbar.z_index = Some(ctx.scene.max_active_z_index());

        self.origin = Some(origin);
        self.origin_delta = Vector2F::zero();
    }

    fn dispatch_event(
        &mut self,
        event: &DispatchedEvent,
        ctx: &mut EventContext,
        app: &AppContext,
    ) -> bool {
        if let crate::Event::LeftMouseDown { position, .. } = event.raw_event() {
            if self.is_mouse_hovering_dragbar(ctx, *position) {
                self.state().begin_resizing(*position);
                dispatch_callback(self.resize_handler.as_mut(), ctx, app);
                return true;
            }
        }
        let child_handled = self.child.dispatch_event(event, ctx, app);

        match event.raw_event() {
            crate::Event::LeftMouseUp { .. } => {
                // If a mouse-up occurs, take the view out of resizing mode
                if self.state().is_resizing() {
                    ctx.reset_cursor();
                    self.state().end_resizing();
                    dispatch_callback(self.end_resize_handler.as_mut(), ctx, app);
                    return true;
                }
            }

            crate::Event::LeftMouseDragged { position, .. } => {
                if self.state().is_resizing() {
                    let dragbar_side = self.dragbar.side;
                    let origin = self.origin.map(|origin| origin + self.origin_delta);
                    let resized = self
                        .state()
                        .check_for_resize(*position, origin, dragbar_side);
                    self.origin_delta += resized.unwrap_or_default();
                    if resized.is_some() {
                        dispatch_callback(self.resize_handler.as_mut(), ctx, app)
                    }
                    return true;
                }
            }
            crate::Event::MouseMoved { position, .. } => {
                // A mouse event over the dragbar should set the cursor
                let Some(z_index) = self.dragbar.z_index else {
                    log::warn!("self.z_index() was None in `Resizable`");
                    return false;
                };
                let hovering_dragbar = self.is_mouse_hovering_dragbar(ctx, *position);
                let was_already_hovering =
                    mem::replace(&mut self.state().hovering_dragbar, hovering_dragbar);

                if hovering_dragbar || self.state().is_resizing() {
                    let cursor = match self.direction {
                        ResizeDirection::Horizontal => Cursor::ResizeLeftRight,
                        ResizeDirection::Vertical => Cursor::ResizeUpDown,
                    };
                    ctx.set_cursor(cursor, z_index);
                } else if !hovering_dragbar && was_already_hovering {
                    ctx.reset_cursor();
                }

                return true;
            }
            _ => {}
        }
        child_handled
    }

    fn size(&self) -> Option<Vector2F> {
        self.child.size()
    }

    fn origin(&self) -> Option<Point> {
        self.child.origin()
    }
}

fn dispatch_callback(callback: Option<&mut Handler>, ctx: &mut EventContext, app: &AppContext) {
    if let Some(callback) = callback {
        callback(ctx, app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{
        ConstrainedBox, Container, DispatchEventResult, EventHandler, ParentElement, Rect,
    };
    use crate::{
        platform::WindowStyle, App, Entity, Event, Presenter, TypedActionView, WindowInvalidation,
    };
    use std::{cell::RefCell, rc::Rc};

    struct ResizeView {
        state: ResizableStateHandle,
        side: DragBarSide,
        offset: f32,
    }

    impl Entity for ResizeView {
        type Event = ();
    }

    impl crate::core::View for ResizeView {
        fn ui_name() -> &'static str {
            "resize_test"
        }

        fn render(&self, _: &AppContext) -> Box<dyn Element> {
            let child = EventHandler::new(
                ConstrainedBox::new(Rect::new().finish())
                    .with_height(100.)
                    .finish(),
            )
            .on_mouse_in(
                |ctx, _, _| {
                    ctx.reset_cursor();
                    DispatchEventResult::PropagateToParent
                },
                None,
            )
            .finish();
            Container::new(
                Resizable::new(self.state.clone(), child)
                    .with_dragbar_side(self.side)
                    .with_dragbar_offset(self.offset)
                    .finish(),
            )
            .with_horizontal_padding(self.offset)
            .finish()
        }
    }

    impl TypedActionView for ResizeView {
        type Action = ();
    }

    #[test]
    fn padded_edges_keep_resize_cursor_and_accept_repeated_drags() {
        App::test((), |mut app| async move {
            for offset in [0., 6., 12.] {
                for side in [DragBarSide::Left, DragBarSide::Right] {
                    let state = resizable_state_handle(100.);
                    let handle = state.clone();
                    let (window, _) =
                        app.add_window(WindowStyle::NotStealFocus, move |_| ResizeView {
                            state: handle,
                            side,
                            offset,
                        });
                    let mut presenter = Presenter::new(window);
                    let root = app.root_view_id(window).unwrap();
                    let invalidation = WindowInvalidation {
                        updated: [root].into_iter().collect(),
                        ..Default::default()
                    };
                    app.update(|ctx| {
                        presenter.invalidate(invalidation, ctx);
                        presenter.build_scene(vec2f(200., 100.), 1., None, ctx);
                        let presenter = Rc::new(RefCell::new(presenter));
                        let (position, outward) = match side {
                            DragBarSide::Left => (vec2f(1., 50.), vec2f(-1., 0.)),
                            _ => (vec2f(99. + 2. * offset, 50.), vec2f(1., 0.)),
                        };
                        for _ in 0..3 {
                            ctx.simulate_window_event(
                                Event::MouseMoved {
                                    position,
                                    cmd: false,
                                    shift: false,
                                    is_synthetic: false,
                                },
                                window,
                                presenter.clone(),
                            );
                            assert_eq!(ctx.get_cursor_shape(), Cursor::ResizeLeftRight);
                        }
                        presenter.borrow_mut().invalidate(
                            WindowInvalidation {
                                updated: [root].into_iter().collect(),
                                ..Default::default()
                            },
                            ctx,
                        );
                        presenter
                            .borrow_mut()
                            .build_scene(vec2f(200., 100.), 1., None, ctx);
                        for (position, expected) in [
                            (vec2f(180., 50.), Cursor::Arrow),
                            (position, Cursor::ResizeLeftRight),
                        ] {
                            ctx.simulate_window_event(
                                Event::MouseMoved {
                                    position,
                                    cmd: false,
                                    shift: false,
                                    is_synthetic: false,
                                },
                                window,
                                presenter.clone(),
                            );
                            assert_eq!(ctx.get_cursor_shape(), expected);
                        }
                        ctx.simulate_window_event(
                            Event::LeftMouseDown {
                                position,
                                modifiers: Default::default(),
                                click_count: 1,
                                is_first_mouse: false,
                            },
                            window,
                            presenter.clone(),
                        );
                        assert!(state.lock().unwrap().is_resizing());
                        for distance in [10., 20., 30.] {
                            ctx.simulate_window_event(
                                Event::LeftMouseDragged {
                                    position: position + outward * distance,
                                    modifiers: Default::default(),
                                },
                                window,
                                presenter.clone(),
                            );
                            assert_eq!(state.lock().unwrap().size(), 100. + distance);
                        }
                        ctx.simulate_window_event(
                            Event::LeftMouseUp {
                                position: position + outward * 30.,
                                modifiers: Default::default(),
                            },
                            window,
                            presenter,
                        );
                        assert!(!state.lock().unwrap().is_resizing());
                    });
                }
            }
        });
    }

    #[test]
    fn repeated_drags_preserve_grab_offset_on_every_side() {
        for (side, outward, grab) in [
            (DragBarSide::Right, vec2f(1., 0.), vec2f(404., 200.)),
            (DragBarSide::Left, vec2f(-1., 0.), vec2f(90., 200.)),
            (DragBarSide::Bottom, vec2f(0., 1.), vec2f(200., 404.)),
            (DragBarSide::Top, vec2f(0., -1.), vec2f(200., 90.)),
        ] {
            let mut state = ResizableState::new(300.);
            let mut origin = vec2f(100., 100.);
            state.begin_resizing(grab);
            for distance in [10., 20., 35., 15.] {
                let pointer = grab + outward * distance;
                origin += state.check_for_resize(pointer, Some(origin), side).unwrap();
                assert_eq!(state.size(), 300. + distance);
                assert!(state
                    .check_for_resize(pointer, Some(origin), side)
                    .is_none());
                assert_eq!(state.size(), 300. + distance);
            }
            state.end_resizing();
            assert!(state.check_for_resize(grab, Some(origin), side).is_none());
        }
    }

    #[test]
    fn clamped_drags_keep_the_grab_offset_when_reversing() {
        for (side, outward) in [
            (DragBarSide::Right, vec2f(1., 0.)),
            (DragBarSide::Left, vec2f(-1., 0.)),
            (DragBarSide::Bottom, vec2f(0., 1.)),
            (DragBarSide::Top, vec2f(0., -1.)),
        ] {
            let mut state = ResizableState::new(300.);
            state.bounds = Some((150., 320.));
            let grab = vec2f(404., 404.);
            let mut origin = vec2f(100., 100.);
            state.begin_resizing(grab);
            for (distance, expected) in [
                (60., 320.),
                (80., 320.),
                (10., 310.),
                (-200., 150.),
                (-140., 160.),
            ] {
                if let Some(delta) =
                    state.check_for_resize(grab + outward * distance, Some(origin), side)
                {
                    origin += delta;
                }
                assert_eq!(state.size(), expected);
            }
        }
    }
}
