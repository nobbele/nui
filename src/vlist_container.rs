use crate::types::{next_id, AppState, Bounds, Element, Id, Message, Renderer, Size, Space};

pub struct VListContainerProps<E> {
    pub id: Id,
    pub children: Vec<E>,
    pub space: Space,
    pub spacing: f32,
}

impl<E> Default for VListContainerProps<E> {
    fn default() -> Self {
        Self {
            id: next_id(),
            children: vec![],
            space: Space::Fill,
            spacing: 5.,
        }
    }
}

pub struct VListContainer<E> {
    pub id: Id,
    pub bounds: Bounds,
    pub children: Vec<E>,
    pub space: Space,
    pub spacing: f32,
}

impl<E> VListContainer<E> {
    pub fn new(props: VListContainerProps<E>) -> Self {
        VListContainer {
            id: props.id,
            bounds: Bounds {
                x: 0.,
                y: 0.,
                size: Size { w: 0., h: 0. },
            },
            children: props.children,
            space: props.space,
            spacing: props.spacing,
        }
    }
}

impl<E: Element> Element for VListContainer<E> {
    fn handle(&mut self, msg: &Message) {
        for child in &mut self.children {
            child.handle(msg);
        }
    }

    fn update(&mut self, state: &AppState) {
        for child in &mut self.children {
            child.update(state);
        }
    }

    fn draw(&self, renderer: &dyn Renderer) {
        for child in &self.children {
            child.draw(renderer)
        }
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        let mut min_size = Size { w: 0., h: 0. };
        let mut child_min_size = 0.;
        let mut fill_count = 0;

        for child in &self.children {
            let min = child.min_size();
            min_size.w = min_size.w.max(min.w);
            min_size.h += min.h;

            match child.space() {
                Space::Fill => {
                    fill_count += 1;
                }
                Space::Minimize => {
                    child_min_size += min.h;
                }
            }
        }
        let total_padding = self.spacing * (self.children.len() - 1) as f32;
        min_size.h += total_padding;

        let size = match self.space {
            Space::Fill => Size {
                w: bounds.size.w,
                h: bounds.size.h,
            },
            Space::Minimize => min_size,
        };
        let size_without_padding = Size {
            w: size.w,
            h: size.h - total_padding,
        };

        let free_height = size_without_padding.h - child_min_size;

        let mut y = 0.;
        for child in &mut self.children {
            let min = child.min_size();
            let child_size = match child.space() {
                Space::Fill => Size {
                    w: size_without_padding.w,
                    h: free_height / fill_count as f32,
                },
                Space::Minimize => min,
            };

            child.set_bounds(Bounds {
                x: bounds.x,
                y: bounds.y + y,
                size: child_size,
            });
            y += child.bounds().size.h + self.spacing;
        }

        self.bounds = Bounds {
            x: bounds.x,
            y: bounds.y,
            size,
        };
    }

    fn min_size(&self) -> Size {
        self.children.iter().map(|child| child.min_size()).fold(
            Size { w: 0., h: 0. },
            |acc, child| Size {
                w: acc.w.max(child.w),
                h: acc.h + child.h,
            },
        )
    }

    fn space(&self) -> Space {
        self.space
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn write_all_bounds(&self, v: &mut Vec<Bounds>) {
        v.push(self.bounds());
        for child in &self.children {
            child.write_all_bounds(v)
        }
    }
}
