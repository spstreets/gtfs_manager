use druid::widget::{Controller, CrossAxisAlignment, Flex, Label, LabelText};
use druid::{
    theme, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    LinearGradient, PaintCtx, RenderContext, Size, UnitPoint, UpdateCtx, Widget,
};

pub struct ListItem<T> {
    variant: T,
    child_label: Label<T>,
    label_y: f64,
}
impl<T: Data> ListItem<T> {
    /// Create a single ListItem from label text and an enum variant
    pub fn new(label: impl Into<LabelText<T>>, variant: T) -> ListItem<T> {
        ListItem {
            variant,
            child_label: Label::new(label),
            label_y: 0.0,
        }
    }
}
impl<T: Data> Widget<T> for ListItem<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        *data = self.variant.clone();
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child_label.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.child_label.update(ctx, old_data, data, env);
        if !old_data.same(data) {
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let label_size = self.child_label.layout(ctx, &bc.loosen(), data, env);

        bc.constrain(Size::new(label_size.width, label_size.height))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, _env: &Env) {
        let rect = ctx.size().to_rect();

        if data.same(&self.variant) {
            ctx.fill(rect, &Color::RED);
        }

        self.child_label.draw_at(ctx, (0., 0.));
    }
}
