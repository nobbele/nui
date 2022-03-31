use crate::types::{
    next_id, Anchor, AppState, Bounds, Element, Id, Message, Renderer, Scale, Size,
};

pub struct AnchorContainerProps<E> {
    pub id: Id,
    pub entries: Vec<AnchorEntry<E>>,
}

impl<E> Default for AnchorContainerProps<E> {
    fn default() -> Self {
        Self {
            id: next_id(),
            entries: vec![],
        }
    }
}

pub struct AnchorEntry<E> {
    pub scale: Scale,
    pub anchor: Anchor,
    pub child: E,
}

pub struct AnchorContainer<E> {
    pub id: Id,
    pub bounds: Bounds,
    pub entries: Vec<AnchorEntry<E>>,
}

impl<E: Element> AnchorContainer<E> {
    pub fn new(props: AnchorContainerProps<E>) -> Self {
        AnchorContainer {
            id: props.id,
            bounds: Bounds {
                x: 0.,
                y: 0.,
                size: Size { w: 0., h: 0. },
            },
            entries: props.entries,
        }
    }
}

impl<E: Element> Element for AnchorContainer<E> {
    fn handle(&mut self, msg: &Message) {
        for entry in &mut self.entries {
            entry.child.handle(msg);
        }
    }

    fn update(&mut self, state: &AppState) {
        for entry in &mut self.entries {
            entry.child.update(state);
        }
    }

    fn draw(&self, renderer: &dyn Renderer) {
        for entry in &self.entries {
            entry.child.draw(renderer);
        }
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
        for entry in &mut self.entries {
            let size = Size {
                w: bounds.size.w * entry.scale.x,
                h: bounds.size.h * entry.scale.y,
            };
            let position = entry.anchor.get_point(bounds.size);
            let bounds = Bounds {
                x: position.x,
                y: position.y,
                size,
            };
            entry.child.set_bounds(bounds);
        }
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn min_size(&self) -> Size {
        todo!()
    }

    fn space(&self) -> crate::types::Space {
        todo!()
    }

    fn write_all_bounds(&self, v: &mut Vec<Bounds>) {
        v.push(self.bounds());
        for entry in &self.entries {
            entry.child.write_all_bounds(v)
        }
    }
}
