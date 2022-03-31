use crate::{
    button::{Button, ButtonMessage, ButtonProps},
    types::{
        AppState, Bounds, Element, Id, Message, MessageData, Renderer, Size, Space, UIContext,
    },
};

pub struct ExpandableButtonProps<L> {
    pub id: Id,
    pub main: ButtonProps,
    pub list: L,
}

#[derive(Debug, Clone)]
pub enum ExpandableButtonMessage {
    Expand,
    Fold,
}

pub struct ExpandableButton<L> {
    id: Id,
    main: Button,
    list: L,
    bounds: Bounds,

    expanded: f32,
    expand_inc: bool,
    tx: flume::Sender<Message>,
}

impl<L> ExpandableButton<L> {
    pub fn new(props: ExpandableButtonProps<L>, ctx: &UIContext) -> Self {
        ExpandableButton {
            id: props.id,
            main: Button::new(props.main, ctx),
            list: props.list,
            bounds: Bounds {
                x: 0.,
                y: 0.,
                size: Size { w: 0., h: 0. },
            },
            expanded: 0.,
            expand_inc: false,
            tx: ctx.tx.clone(),
        }
    }
}

impl<L: Element> Element for ExpandableButton<L> {
    fn update(&mut self, state: &AppState) {
        self.expanded += 1.6 * if self.expand_inc { state.dt } else { -state.dt };
        self.expanded = self.expanded.clamp(0., 1.);
        self.list.set_bounds(Bounds {
            x: self.bounds.x,
            y: self.bounds.y + self.main.bounds().size.h,
            size: Size {
                w: self.bounds.size.w,
                h: self.bounds.size.h * self.expanded,
            },
        });

        self.main.update(state);
        if self.expanded != 0. {
            self.list.update(state);
        }
    }

    fn handle(&mut self, msg: &Message) {
        if msg.target == self.main.id {
            if let MessageData::Button(ButtonMessage::Click) = msg.data {
                self.tx
                    .send(Message {
                        target: self.id,
                        data: MessageData::ExpandableButton(if self.expand_inc {
                            ExpandableButtonMessage::Fold
                        } else {
                            ExpandableButtonMessage::Expand
                        }),
                    })
                    .unwrap();
            }
        }

        self.main.handle(msg);
        if self.expanded != 0. {
            self.list.handle(msg);
        }

        if msg.target == self.id {
            if let MessageData::ExpandableButton(msg) = &msg.data {
                match msg {
                    ExpandableButtonMessage::Expand => {
                        self.expand_inc = true;
                    }
                    ExpandableButtonMessage::Fold => {
                        self.expand_inc = false;
                    }
                }
            } else {
                unreachable!()
            }
        }
    }

    fn draw(&self, renderer: &dyn Renderer) {
        self.main.draw(renderer);
        if self.expanded != 0. {
            self.list.draw(renderer);
        }
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.main.set_bounds(bounds);
        self.bounds = bounds;
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn min_size(&self) -> Size {
        let main_min_size = self.main.min_size();
        if self.expanded != 0. {
            let list_min_size = self.list.min_size();
            Size {
                w: main_min_size.w + list_min_size.w,
                h: main_min_size.h + list_min_size.h,
            }
        } else {
            main_min_size
        }
    }

    fn space(&self) -> Space {
        Space::Minimize
    }

    fn write_all_bounds(&self, v: &mut Vec<Bounds>) {
        self.main.write_all_bounds(v);
        if self.expanded != 0. {
            self.list.write_all_bounds(v);
        }
    }
}
