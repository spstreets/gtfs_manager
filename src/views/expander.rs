// use crate::theme;
use druid::{
    kurbo::BezPath,
    piet::{LineCap, LineJoin, StrokeStyle},
    theme,
    widget::{prelude::*, Label, LabelText},
    LinearGradient, Point, Rect, UnitPoint,
};

/// A checkbox that toggles a `bool`.
pub struct Expander {
    child_label: Label<bool>,
}

impl Expander {
    /// Create a new `Checkbox` with a text label.
    pub fn new(text: impl Into<LabelText<bool>>) -> Expander {
        Self::from_label(Label::new(text))
    }

    /// Create a new `Checkbox` with the provided [`Label`].
    pub fn from_label(label: Label<bool>) -> Expander {
        Expander { child_label: label }
    }

    /// Update the text label.
    pub fn set_text(&mut self, label: impl Into<LabelText<bool>>) {
        self.child_label.set_text(label);
    }
}

impl Widget<bool> for Expander {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut bool, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                if !ctx.is_disabled() {
                    ctx.set_active(true);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_) => {
                if ctx.is_active() && !ctx.is_disabled() {
                    if ctx.is_hot() {
                        if *data {
                            *data = false;
                        } else {
                            *data = true;
                        }
                    }
                    ctx.request_paint();
                }
                ctx.set_active(false);
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &bool, env: &Env) {
        self.child_label.lifecycle(ctx, event, data, env);
        if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
            ctx.request_paint();
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &bool, data: &bool, env: &Env) {
        self.child_label.update(ctx, old_data, data, env);
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &bool, env: &Env) -> Size {
        bc.debug_check("Checkbox");
        let x_padding = env.get(theme::WIDGET_CONTROL_COMPONENT_PADDING);
        let check_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let label_size = self.child_label.layout(ctx, bc, data, env);

        let desired_size = Size::new(
            check_size + x_padding + label_size.width,
            check_size.max(label_size.height),
        );
        let our_size = bc.constrain(desired_size);
        let baseline = self.child_label.baseline_offset() + (our_size.height - label_size.height);
        ctx.set_baseline_offset(baseline);
        our_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &bool, env: &Env) {
        let wsize = ctx.size();

        // Paint the text label
        self.child_label.draw_at(ctx, (0., 0.));

        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let x_padding = env.get(theme::WIDGET_CONTROL_COMPONENT_PADDING);
        let border_width = 1.;

        let rect = Rect::from_origin_size(
            Point::new(wsize.width - (size + x_padding), 0.),
            Size::new(size, size),
        )
        .inset(-border_width / 2.)
        .to_rounded_rect(2.);

        // //Paint the background
        // let background_gradient = LinearGradient::new(
        //     UnitPoint::TOP,
        //     UnitPoint::BOTTOM,
        //     (
        //         env.get(theme::BACKGROUND_LIGHT),
        //         env.get(theme::BACKGROUND_DARK),
        //     ),
        // );

        // ctx.fill(rect, &background_gradient);

        // let border_color = if ctx.is_hot() && !ctx.is_disabled() {
        //     env.get(theme::BORDER_LIGHT)
        // } else {
        //     env.get(theme::BORDER_DARK)
        // };

        // ctx.stroke(rect, &border_color, border_width);

        if *data {
            // Paint the checkmark
            let x_offset = wsize.width - size;
            let y_offset = (rect.height() - 8.0) / 2.0;
            let mut path = BezPath::new();
            path.move_to((x_offset, y_offset + 5.));
            path.line_to((x_offset + 10., y_offset + 5.));

            let style = StrokeStyle::new()
                .line_cap(LineCap::Round)
                .line_join(LineJoin::Round);

            let brush = if ctx.is_disabled() {
                env.get(theme::DISABLED_TEXT_COLOR)
            } else {
                env.get(theme::TEXT_COLOR)
            };

            ctx.stroke_styled(path, &brush, 2., &style);
        } else {
            // Paint the checkmark
            let x_offset = wsize.width - size;
            let y_offset = (rect.height() - 8.0) / 2.0;
            let mut path = BezPath::new();
            path.move_to((x_offset, y_offset + 5.));
            path.line_to((x_offset + 10., y_offset + 5.));
            path.move_to((x_offset + 5., y_offset));
            path.line_to((x_offset + 5., y_offset + 10.));

            let style = StrokeStyle::new()
                .line_cap(LineCap::Round)
                .line_join(LineJoin::Round);

            let brush = if ctx.is_disabled() {
                env.get(theme::DISABLED_TEXT_COLOR)
            } else {
                env.get(theme::TEXT_COLOR)
            };

            ctx.stroke_styled(path, &brush, 2., &style);
        }
    }
}
