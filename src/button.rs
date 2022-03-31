use crate::types::{
    next_id, ActionState, Anchor, AnchorX, AnchorY, AppState, Bounds, Element, Id, KeyState,
    Message, MessageData, PreserveRatio, Renderer, Size, Space, UIContext,
};
use assert_float_eq::{afe_is_f32_near, afe_near_error_msg, assert_f32_near};
use macroquad::prelude::*;
use std::ops::RangeInclusive;

fn bounce(x: f32) -> f32 {
    let x = x.clamp(0., 1.);
    let c1 = 1.70158;
    let c3 = c1 + 1.;

    1. + c3 * (x - 1.).powi(3) + c1 * (x - 1.).powi(2)
}

#[derive(Debug, Clone)]
pub enum ButtonMessage {
    Hover(ActionState),
    Click,
}

#[derive(Debug)]
pub struct ButtonProps {
    pub id: Id,
    pub text: String,
    pub space: Space,
    pub anchor: Anchor,
    pub image: Option<u32>,
    pub color: Color,
    pub preserve_ratio: PreserveRatio,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            text: String::new(),
            space: Space::Fill,
            anchor: Anchor {
                x: AnchorX::Middle,
                y: AnchorY::Middle,
            },
            id: next_id(),
            image: None,
            color: WHITE,
            preserve_ratio: PreserveRatio::None,
        }
    }
}

impl Clone for ButtonProps {
    fn clone(&self) -> Self {
        Self {
            id: next_id(),
            text: self.text.clone(),
            space: self.space.clone(),
            anchor: self.anchor.clone(),
            image: self.image.clone(),
            color: self.color.clone(),
            preserve_ratio: self.preserve_ratio.clone(),
        }
    }
}

pub struct Button {
    pub id: Id,
    pub outer_bounds: Bounds,
    pub inner_bounds: Bounds,
    pub text: String,
    pub space: Space,
    pub anchor: Anchor,
    image: Option<u32>,
    color: Color,
    preserve_ratio: PreserveRatio,

    tx: flume::Sender<Message>,

    hover: bool,

    offset: f32,
    progress: f32,
    progress_inc: bool,

    hover_offset: f32,
}

impl Button {
    pub fn new(props: ButtonProps, ctx: &UIContext) -> Self {
        Button {
            text: props.text,
            space: props.space,
            outer_bounds: Bounds {
                x: 0.,
                y: 0.,
                size: Size { w: 0., h: 0. },
            },
            inner_bounds: Bounds {
                x: 0.,
                y: 0.,
                size: Size { w: 0., h: 0. },
            },
            offset: 0.,
            progress: 0.,
            hover_offset: 0.,
            hover: false,
            id: props.id,
            tx: ctx.tx.clone(),
            anchor: props.anchor,
            image: props.image,
            color: props.color,
            preserve_ratio: props.preserve_ratio,
            progress_inc: false,
        }
    }

    pub fn handle_message(&mut self, msg: &ButtonMessage) {
        let prev_hover = self.hover;
        match msg {
            ButtonMessage::Hover(state) => match state {
                ActionState::Start => self.hover = true,
                ActionState::End => self.hover = false,
            },
            ButtonMessage::Click => (),
        }

        if prev_hover != self.hover {
            if self.hover {
                self.progress_inc = true;
            } else {
                self.progress_inc = false;
                let reverse_progress = self.offset / self.hover_offset;
                self.progress = reverse_progress;
            }
        }
    }
}

impl Element for Button {
    fn handle(&mut self, msg: &Message) {
        if msg.target == self.id {
            if let MessageData::Button(btn_msg) = &msg.data {
                self.handle_message(btn_msg);
            }
        }
    }

    fn update(&mut self, state: &AppState) {
        fn lerp(range: RangeInclusive<f32>, percent: f32) -> f32 {
            *range.start() + percent * (*range.end() - *range.start())
        }

        if (self.progress_inc && self.progress <= 1.) || (!self.progress_inc && self.progress >= 0.)
        {
            self.progress += 1.6
                * if self.progress_inc {
                    state.dt
                } else {
                    2.0 * -state.dt
                };
            self.progress = self.progress.clamp(0., 1.);
            let target_offset = lerp(
                0.0..=self.hover_offset,
                if self.progress_inc {
                    bounce(self.progress)
                } else {
                    self.progress.powi(2)
                },
            );
            self.offset += (target_offset - self.offset) / 2.0;
        }
        let offset = self.hover_offset - self.offset;
        self.inner_bounds.x = self.outer_bounds.x + offset;
        self.inner_bounds.size.w = self.outer_bounds.size.w + offset;

        if self.inner_bounds.contains(state.mouse_position) {
            if state.left_click == KeyState::Pressed {
                self.tx
                    .send(Message {
                        target: self.id,
                        data: MessageData::Button(ButtonMessage::Click),
                    })
                    .unwrap();
            } else if !self.hover {
                self.tx
                    .send(Message {
                        target: self.id,
                        data: MessageData::Button(ButtonMessage::Hover(ActionState::Start)),
                    })
                    .unwrap();
            }
        } else if self.hover {
            self.tx
                .send(Message {
                    target: self.id,
                    data: MessageData::Button(ButtonMessage::Hover(ActionState::End)),
                })
                .unwrap();
        }
    }

    fn draw(&self, renderer: &dyn Renderer) {
        renderer.draw_rectangle(self.inner_bounds, self.image, self.color);
        renderer.draw_text(
            Bounds {
                x: self.inner_bounds.x + self.inner_bounds.size.w / 3.
                    - measure_text(&self.text, None, 20, 1.0).width / 2.,
                y: self.inner_bounds.y
                    + self.inner_bounds.size.h / 2.
                    + measure_text(&self.text, None, 20, 1.0).height / 2.,
                size: Size { w: 0., h: 0. },
            },
            &self.text,
            WHITE,
        );
    }

    // Sets the outer bounds.
    fn set_bounds(&mut self, bounds: Bounds) {
        let bounds = self.anchor.apply_to(bounds);
        self.outer_bounds = bounds;

        self.hover_offset = bounds.size.w / 5.;
        self.inner_bounds = Bounds {
            x: self.outer_bounds.x + self.hover_offset,
            y: self.outer_bounds.y,
            size: Size {
                w: self.outer_bounds.size.w - self.hover_offset,
                h: self.outer_bounds.size.h,
            },
        };

        match self.preserve_ratio {
            PreserveRatio::Height(ratio) => {
                let calculated_width = self.inner_bounds.size.h * ratio;
                let max_width = self.inner_bounds.size.w;
                if calculated_width > max_width {
                    self.inner_bounds.size.h = max_width / ratio;
                }
                let new_width = self.inner_bounds.size.h * ratio;
                self.inner_bounds.size.w = new_width;
                assert_f32_near!(self.inner_bounds.size.w / self.inner_bounds.size.h, ratio);
            }
            PreserveRatio::Width(_ratio) => todo!(),
            PreserveRatio::None => (),
        }
    }

    fn min_size(&self) -> Size {
        let dim = measure_text(&self.text, None, 16, 1.0);
        Size {
            w: dim.width,
            h: dim.height,
        }
    }

    fn space(&self) -> Space {
        self.space
    }

    fn bounds(&self) -> Bounds {
        self.inner_bounds
    }
}
