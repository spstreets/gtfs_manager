use druid::im::Vector;
use druid::widget::{
    Button, Checkbox, CrossAxisAlignment, Flex, Label, List, ListIter, Radio, TextBox,
};
use druid::{
    AppLauncher, ArcStr, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, PaintCtx, Size, UnitPoint, UpdateCtx, Widget, WidgetExt, WidgetPod, WindowDesc,
};
use std::error::Error;
use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;

// Data
// FilterIter which impls druid::ListIter
//
#[derive(Data, Clone)]
pub struct FilteredListIter<I> {
    data: I,
    indices: Vector<usize>,
}
impl<I> FilteredListIter<I> {
    pub fn new(data: I, indices: Vector<usize>) -> Self {
        FilteredListIter { data, indices }
    }
}
impl<T: Data, I: ListIter<T>> ListIter<T> for FilteredListIter<I> {
    // for each each index in indices, run the callback for the corresponding item in data (which must impl ListIter<>)
    fn for_each(&self, mut cb: impl FnMut(&T, usize)) {
        let mut indices = self.indices.iter();
        let mut next = indices.next();
        let mut counter = 0;
        self.data.for_each(|element, index| {
            // if we haven't reached the end of indices
            if let Some(next_index) = next {
                // and the current data index is the current indices index
                if index == *next_index {
                    // then run the callback (which seemingly requires a non-sparese index ie counter)
                    cb(element, counter);
                    // and increment the indices index and counter
                    next = indices.next();
                    counter += 1;
                }
            }
            // else move to the next data element
        });
    }

    // same as above but mut
    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut T, usize)) {
        let mut indices = self.indices.iter();
        let mut next = indices.next();
        let mut counter = 0;
        self.data.for_each_mut(|element, index| {
            if let Some(next_index) = next {
                if index == *next_index {
                    cb(element, counter);
                    next = indices.next();
                    counter += 1;
                }
            }
        });
    }

    fn data_len(&self) -> usize {
        self.indices.len()
    }
}

//
// Widget
// FilteredList
//
type FilterUpdate<I, D> = dyn Fn(&mut Vector<usize>, usize, &I, Range<usize>, &D);
// "A widget which filters a list for its inner widget."
// this works too
// pub struct ListFilter<D: Data, T: Data, I: ListIter<T>> {
pub struct FilteredList<D, T, I> {
    // used for FilterIter<I>::indices
    accepted: Vector<usize>,
    filter_update: Box<FilterUpdate<I, D>>,
    // a thing which impls Widget with associated data FilterIter
    child_list: Box<dyn Widget<FilteredListIter<I>>>,
    phantom: PhantomData<T>,
}
impl<D: Data, T: Data, I: ListIter<T>> FilteredList<D, T, I> {
    pub fn new(
        child_list: impl Widget<FilteredListIter<I>> + 'static,
        filter_function: impl Fn(&T, &D) -> bool + 'static,
    ) -> Self {
        Self {
            accepted: Vector::new(),
            filter_update: Box::new(
                // dyn Fn(&mut Vector<usize>, usize, &I, Range<usize>, &D)
                move |indices, mut insert_index, elements, update_range, filter_option| {
                    elements.for_each(|element, index| {
                        if index >= update_range.start
                            && index < update_range.end
                            && filter_function(element, filter_option)
                        {
                            indices.insert(insert_index, index);
                            insert_index += 1;
                        }
                    })
                },
            ),
            child_list: Box::new(child_list),
            phantom: PhantomData,
        }
    }
}
// note the associated data's type is a tuple: (I, D)
impl<D: Data, T: Data, I: ListIter<T>> Widget<(I, D)> for FilteredList<D, T, I> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (I, D), env: &Env) {
        let mut inner_data = FilteredListIter::new(data.0.clone(), self.accepted.clone());
        self.child_list.event(ctx, event, &mut inner_data, env);
        data.0 = inner_data.data;
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(I, D), env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            (self.filter_update)(
                &mut self.accepted,
                0,
                &data.0,
                0..(data.0.data_len()),
                &data.1,
            );
        }
        let inner_data = FilteredListIter::new(data.0.clone(), self.accepted.clone());
        self.child_list.lifecycle(ctx, event, &inner_data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &(I, D), data: &(I, D), env: &Env) {
        let old_inner = FilteredListIter::new(old_data.0.clone(), self.accepted.clone());

        if !old_data.same(data) {
            //TODO: do real diffing here
            self.accepted.clear();
            (self.filter_update)(
                &mut self.accepted,
                0,
                &data.0,
                0..(data.0.data_len()),
                &data.1,
            );
        }
        let inner_data = FilteredListIter::new(data.0.clone(), self.accepted.clone());
        self.child_list.update(ctx, &old_inner, &inner_data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &(I, D),
        env: &Env,
    ) -> Size {
        let inner_data = FilteredListIter::new(data.0.clone(), self.accepted.clone());
        self.child_list.layout(ctx, bc, &inner_data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(I, D), env: &Env) {
        let inner_data = FilteredListIter::new(data.0.clone(), self.accepted.clone());
        self.child_list.paint(ctx, &inner_data, env);
    }
}
