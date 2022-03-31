use anchor_container::{AnchorContainer, AnchorContainerProps, AnchorEntry};
use button::{Button, ButtonMessage, ButtonProps};
use expandable_button::{ExpandableButton, ExpandableButtonProps};
use macroquad::prelude::*;
use types::{
    next_id, Anchor, AnchorX, AnchorY, AppState, Bounds, Element, EventObserver, KeyState,
    MacroquadRenderer, MessageData, Position, PreserveRatio, Scale, Size, Space, UIContext,
};
use vlist_container::{VListContainer, VListContainerProps};

pub mod anchor_container;
pub mod button;
pub mod container;
pub mod expandable_button;
pub mod types;
pub mod vlist_container;

#[macroquad::main("nui")]
async fn main() {
    let mut renderer = MacroquadRenderer::new();
    let button_texture = renderer.add_texture(Texture2D::from_file_with_format(
        include_bytes!("../resources/button.png"),
        None,
    ));

    let ctx = UIContext::new();
    let mut observer = EventObserver::new();
    let first_hello_world = next_id();
    let first_hello_world_observer = observer.observe(first_hello_world);
    let button_props = std::iter::once(ButtonProps {
        text: "Clickable".into(),
        space: Space::Fill,
        anchor: Anchor {
            x: AnchorX::Right,
            y: AnchorY::Top,
        },
        id: first_hello_world,
        image: Some(button_texture),
        preserve_ratio: PreserveRatio::Height(7. / 1.),
        ..Default::default()
    })
    .chain(
        std::iter::repeat(())
            .enumerate()
            .map(|(idx, _)| {
                if idx % 2 == 0 {
                    ButtonProps {
                        text: format!("Textured {}", idx),
                        space: Space::Fill,
                        anchor: Anchor {
                            x: AnchorX::Right,
                            y: AnchorY::Top,
                        },
                        image: Some(button_texture),
                        preserve_ratio: PreserveRatio::Height(7. / 1.),
                        ..Default::default()
                    }
                } else {
                    ButtonProps {
                        text: format!("Non-textured {}", idx),
                        space: Space::Fill,
                        anchor: Anchor {
                            x: AnchorX::Right,
                            y: AnchorY::Top,
                        },
                        image: None,
                        preserve_ratio: PreserveRatio::Height(4. / 1.),
                        color: Color {
                            r: rand::gen_range(0., 1.),
                            g: rand::gen_range(0., 1.),
                            b: rand::gen_range(0., 1.),
                            a: 1.0,
                        },
                        ..Default::default()
                    }
                }
            })
            .take(5),
    );
    /*let mut elem: Box<dyn Element> = Box::new(AnchorContainer::new(AnchorContainerProps {
        entries: vec![AnchorEntry {
            scale: Scale { x: 0.5, y: 1.0 },
            anchor: Anchor {
                x: AnchorX::Right,
                y: AnchorY::Top,
            },
            child: VListContainer::new(VListContainerProps {
                children: button_props.map(|prop| Button::new(prop, &ctx)).collect(),
                space: Space::Fill,
                spacing: 5.,
                ..Default::default()
            }),
        }],
        ..Default::default()
    }));*/

    let mut elem: Box<dyn Element> = Box::new(AnchorContainer::new(AnchorContainerProps {
        entries: vec![AnchorEntry {
            scale: Scale { x: 0.5, y: 1.0 },
            anchor: Anchor {
                x: AnchorX::Right,
                y: AnchorY::Top,
            },
            child: ExpandableButton::new(
                ExpandableButtonProps {
                    id: next_id(),
                    main: ButtonProps {
                        text: "Expandable".into(),
                        space: Space::Fill,
                        anchor: Anchor {
                            x: AnchorX::Right,
                            y: AnchorY::Top,
                        },
                        image: Some(button_texture),
                        preserve_ratio: PreserveRatio::Height(7. / 1.),
                        ..Default::default()
                    },
                    list: VListContainer::new(VListContainerProps {
                        children: button_props.map(|prop| Button::new(prop, &ctx)).collect(),
                        ..Default::default()
                    }),
                },
                &ctx,
            ),
        }],
        ..Default::default()
    }));

    let mut screen_size = (0., 0.);
    let mut left_click = KeyState::Released;
    let mut right_click = KeyState::Released;

    loop {
        rand::srand(0);
        clear_background(WHITE);

        let frame_screen_size = (screen_width(), screen_height());
        if frame_screen_size != screen_size {
            elem.set_bounds(Bounds {
                x: 0.,
                y: 0.,
                size: Size {
                    w: screen_width(),
                    h: screen_height(),
                },
            });
            screen_size = frame_screen_size;
        }

        if is_mouse_button_down(MouseButton::Left) {
            left_click = match left_click {
                KeyState::Released | KeyState::Unpressed => KeyState::Pressed,
                KeyState::Held | KeyState::Pressed => KeyState::Held,
            }
        } else {
            left_click = match left_click {
                KeyState::Released | KeyState::Unpressed => KeyState::Unpressed,
                KeyState::Held | KeyState::Pressed => KeyState::Released,
            }
        }

        if is_mouse_button_down(MouseButton::Right) {
            right_click = match right_click {
                KeyState::Released | KeyState::Unpressed => KeyState::Pressed,
                KeyState::Held | KeyState::Pressed => KeyState::Held,
            }
        } else {
            right_click = match right_click {
                KeyState::Released | KeyState::Unpressed => KeyState::Unpressed,
                KeyState::Held | KeyState::Pressed => KeyState::Released,
            }
        }

        elem.update(&AppState {
            mouse_position: Position {
                x: mouse_position().0,
                y: mouse_position().1,
            },
            right_click,
            left_click,
            input: None,
            keys: vec![],
            dt: get_frame_time(),
        });

        for msg in ctx.rx.drain() {
            elem.handle(&msg);
            observer.handle(msg);
        }

        for msg in first_hello_world_observer.drain() {
            if let MessageData::Button(ButtonMessage::Click) = msg {
                println!("Click!");
            }
        }

        elem.draw(&renderer);

        /*let mut bounds = Vec::new();
        elem.write_all_bounds(&mut bounds);
        for bound in bounds {
            draw_rectangle(
                bound.x,
                bound.y,
                bound.size.w,
                bound.size.h,
                Color {
                    r: rand::gen_range(0., 1.),
                    g: rand::gen_range(0., 1.),
                    b: rand::gen_range(0., 1.),
                    a: 0.5,
                },
            );
        }*/

        next_frame().await
    }
}
