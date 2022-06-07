use druid::widget::{Controller, CrossAxisAlignment, Flex, Label, LabelText};
use druid::{
    theme, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    LinearGradient, PaintCtx, RenderContext, Size, UnitPoint, UpdateCtx, Widget,
};
// use core::fmt::Debug;
use std::fmt::Debug;

// each ListItem has a fixed String which is used for its label: variant
// the data parameter that is provided as a function argument is the variant that has most recently been assigned to it
pub struct ListItem<T: PartialEq + Debug> {
    variant: T,
    child_label: Label<T>,
}
impl<T: Data + PartialEq + Debug> ListItem<T> {
    /// Create a single ListItem from label text and an enum variant
    pub fn new(label: impl Into<LabelText<T>>, variant: T) -> ListItem<T> {
        ListItem {
            variant,
            child_label: Label::new(label),
        }
    }
}
impl<T: Data + PartialEq + Debug> Widget<T> for ListItem<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseUp(_) => {
                // it seems like this is how we pass out data for the Lens to use
                *data = self.variant.clone();
                // ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child_label.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        // self.child_label.update(ctx, old_data, data, env);
        // if !old_data.same(data) {
        //     ctx.request_paint();
        // }
        // if *data == self.variant {
        //     ctx.request_paint();
        // }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let label_size = self.child_label.layout(ctx, &bc.loosen(), data, env);
        bc.constrain(Size::new(label_size.width, label_size.height))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, _env: &Env) {
        // everything (in view of the scroll) (every item in all three lists) is being repainted for every selection. I know this because &data logs the variant of each list * the length of each list
        dbg!(&data);
        if data.same(&self.variant) {
            let rect = ctx.size().to_rect();
            ctx.fill(rect, &Color::RED);
        }

        self.child_label.draw_at(ctx, (0., 0.));
    }
}
