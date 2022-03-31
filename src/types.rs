use macroquad::prelude::*;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{button::ButtonMessage, expandable_button::ExpandableButtonMessage};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub size: Size,
}

impl Bounds {
    pub fn contains(&self, position: Position) -> bool {
        position.x >= self.x
            && position.y >= self.y
            && position.x <= self.x + self.size.w
            && position.y <= self.y + self.size.h
    }
}

#[test]
pub fn test_bounds() {
    let bounds = Bounds {
        x: 10.,
        y: 15.,
        size: Size { w: 10., h: 5. },
    };
    assert!(bounds.contains(Position { x: 12., y: 18. }));
    assert!(!bounds.contains(Position { x: 12., y: 21. }));
    let bounds = Bounds {
        x: 480.,
        y: 0.,
        size: Size { w: 320., h: 600. },
    };
    assert!(bounds.contains(Position { x: 782., y: 598. }));
    assert!(!bounds.contains(Position { x: 12., y: 21. }));
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Space {
    Fill,
    Minimize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorX {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorY {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub x: AnchorX,
    pub y: AnchorY,
}

impl Anchor {
    pub fn apply_to(&self, bounds: Bounds) -> Bounds {
        let position = self.get_point(bounds.size);
        Bounds {
            x: bounds.x - position.x,
            y: bounds.y - position.y,
            size: bounds.size,
        }
    }

    pub fn get_point(&self, size: Size) -> Position {
        let mut position = Position { x: 0., y: 0. };
        match self.x {
            AnchorX::Left => (),
            AnchorX::Middle => position.x += size.w / 2.0,
            AnchorX::Right => position.x += size.w,
        }
        match self.y {
            AnchorY::Top => (),
            AnchorY::Middle => position.y += size.h / 2.0,
            AnchorY::Bottom => position.y += size.h,
        }
        position
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActionState {
    Start,
    End,
}

static ID: AtomicU32 = AtomicU32::new(0);

pub fn next_id() -> Id {
    Id(ID.fetch_add(1, Ordering::SeqCst))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KeyState {
    /// Key was pressed this frame.
    Pressed,
    /// Key has been held for longer than 1 frame.
    Held,
    /// Key was unpressed this frame.
    Released,
    /// Key has been unpressed for longer than 1 frame.
    Unpressed,
}

#[derive(Debug)]
pub struct Key {
    pub key: KeyCode,
    pub state: KeyState,
}

#[derive(Debug)]
pub struct AppState {
    pub mouse_position: Position,
    pub right_click: KeyState,
    pub left_click: KeyState,
    pub input: Option<char>,
    pub keys: Vec<Key>,
    pub dt: f32,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub target: Id,
    pub data: MessageData,
}

#[derive(Debug, Clone)]
pub enum MessageData {
    Button(ButtonMessage),
    ExpandableButton(ExpandableButtonMessage),
    Null,
}

pub struct UIContext {
    pub rx: flume::Receiver<Message>,
    pub tx: flume::Sender<Message>,
}

impl UIContext {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        UIContext { rx, tx }
    }
}

pub struct EventObserver {
    pub targets: HashMap<Id, flume::Sender<MessageData>>,
}

impl EventObserver {
    pub fn new() -> Self {
        EventObserver {
            targets: HashMap::new(),
        }
    }

    pub fn observe(&mut self, target: Id) -> flume::Receiver<MessageData> {
        let (tx, rx) = flume::unbounded();
        self.targets.insert(target, tx);
        rx
    }

    pub fn handle(&self, msg: Message) {
        for (&id, tx) in &self.targets {
            if id == msg.target {
                tx.send(msg.data).unwrap();
                break;
            }
        }
    }
}

pub trait Renderer {
    fn draw_rectangle(&self, bounds: Bounds, texture: Option<u32>, color: Color);
    fn draw_text(&self, bounds: Bounds, text: &str, color: Color);
}

pub trait Element {
    fn update(&mut self, state: &AppState);
    fn handle(&mut self, msg: &Message);
    fn draw(&self, renderer: &dyn Renderer);

    fn set_bounds(&mut self, bounds: Bounds);
    fn bounds(&self) -> Bounds;
    fn write_all_bounds(&self, v: &mut Vec<Bounds>) {
        v.push(self.bounds());
    }
    fn all_bounds(&self) -> Vec<Bounds> {
        let mut v = Vec::new();
        self.write_all_bounds(&mut v);
        v
    }

    fn min_size(&self) -> Size;
    fn space(&self) -> Space;
}

/*pub trait ElementBase {
    fn update(&mut self, state: &AppState);
    fn handle(&mut self, msg: &Message);
}

impl<E: Element> ElementBase for E {
    fn handle(&mut self, msg: &Message) {
        self.handle(msg);
    }

    fn update(&mut self, state: &AppState) {
        self.update(state);
    }
}

pub struct BasicElement<B> {
    bounds: Bounds,
    space: Space,
    base: B,
}

impl<B: ElementBase> BasicElement<B> {
    pub fn new(base: B) -> Self {
        BasicElement {
            base,
            space: Space::Fill,
            bounds: Bounds {
                x: 0.,
                y: 0.,
                size: Size { w: 0., h: 0. },
            },
        }
    }
}

impl<B: ElementBase> Element for BasicElement<B> {
    fn handle(&mut self, msg: &Message) {
        self.base.handle(msg);
    }

    fn update(&mut self, state: &AppState) {
        self.base.update(state);
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn min_size(&self) -> Size {
        Size { w: 0., h: 0. }
    }

    fn space(&self) -> Space {
        self.space
    }
}*/

pub struct MacroquadRenderer {
    textures: Vec<Texture2D>,
}

impl MacroquadRenderer {
    pub fn new() -> Self {
        MacroquadRenderer {
            textures: Vec::new(),
        }
    }

    pub fn add_texture(&mut self, tex: Texture2D) -> u32 {
        self.textures.push(tex);
        self.textures.len() as u32 - 1
    }
}

impl Renderer for MacroquadRenderer {
    fn draw_rectangle(&self, bounds: Bounds, texture: Option<u32>, color: Color) {
        if let Some(texture) = texture {
            draw_texture_ex(
                self.textures[texture as usize],
                bounds.x,
                bounds.y,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(bounds.size.w, bounds.size.h)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(bounds.x, bounds.y, bounds.size.w, bounds.size.h, color);
        }
    }

    fn draw_text(&self, bounds: Bounds, text: &str, color: Color) {
        draw_text_ex(
            text,
            bounds.x,
            bounds.y,
            TextParams {
                color,
                ..Default::default()
            },
        );
    }
}

#[derive(Debug, Clone)]
pub enum PreserveRatio {
    Height(f32),
    Width(f32),
    None,
}
