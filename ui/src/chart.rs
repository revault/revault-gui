use iced::{
    alignment,
    canvas::event::{self, Event},
    canvas::{self, Canvas, Cursor, Frame, Geometry, Path, Stroke},
    mouse, Color, Element, Length, Point, Rectangle, Vector,
};

pub use bitcoin::{Network, Transaction, TxIn, TxOut, Txid};

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum FlowChartMessage {
    TxSelected(Option<Txid>),
}

#[derive(Debug)]
pub struct FlowChart {
    interaction: Interaction,
    translation: Vector,
    scaling: f32,
    cache: canvas::Cache,
    transactions: BTreeMap<Txid, TxNode>,
    selected: Option<Txid>,
}

impl FlowChart {
    const MIN_SCALING: f32 = 0.2;
    const MAX_SCALING: f32 = 2.0;

    /// transactions must be ordered by blockheight.
    pub fn new(network: Network, txs: Vec<Transaction>) -> Self {
        let mut transactions = BTreeMap::new();
        for (i, tx) in txs.into_iter().enumerate() {
            transactions.insert(
                tx.txid(),
                TxNode::new(
                    tx,
                    &transactions,
                    Point::new(100.0, 400.0 + 100.0 * i as f32),
                    network,
                ),
            );
        }

        Self {
            interaction: Interaction::None,
            cache: canvas::Cache::default(),
            translation: Vector::default(),
            scaling: 1.0,
            transactions,
            selected: None,
        }
    }

    pub fn view<'a>(&'a mut self) -> Element<'a, FlowChartMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn scale(&self, cursor_position: Point) -> Point {
        Point::new(
            cursor_position.x / &self.scaling,
            cursor_position.y / &self.scaling,
        )
    }
}

impl<'a> canvas::Program<FlowChartMessage> for FlowChart {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<FlowChartMessage>) {
        if let Event::Mouse(mouse::Event::ButtonReleased(_)) = event {
            self.interaction = Interaction::None;
        }

        let cursor_position = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => {
                            for tx in self.transactions.values() {
                                if tx.hovered(self.scale(cursor_position) - self.translation) {
                                    if self.selected.is_none() {
                                        self.selected = Some(tx.txid);
                                    } else {
                                        self.selected = None;
                                    }
                                    self.cache.clear();
                                    return (
                                        event::Status::Captured,
                                        Some(FlowChartMessage::TxSelected(Some(tx.txid.clone()))),
                                    );
                                }
                            }

                            self.interaction = Interaction::Panning {
                                translation: self.translation,
                                start: cursor_position,
                            };

                            None
                        }
                        _ => None,
                    };

                    (event::Status::Captured, message)
                }
                mouse::Event::CursorMoved { .. } => match self.interaction {
                    Interaction::Panning { translation, start } => {
                        self.translation =
                            translation + (cursor_position - start) * (1.0 / self.scaling);

                        self.cache.clear();
                        (event::Status::Captured, None)
                    }
                    Interaction::Hovering => {
                        self.interaction = Interaction::None;
                        for tx in self.transactions.values() {
                            if tx.hovered(self.scale(cursor_position) - self.translation) {
                                self.interaction = Interaction::Hovering;
                                return (event::Status::Captured, None);
                            }
                        }
                        (event::Status::Ignored, None)
                    }
                    Interaction::None => {
                        for tx in self.transactions.values() {
                            if tx.hovered(self.scale(cursor_position) - self.translation) {
                                self.interaction = Interaction::Hovering;
                                return (event::Status::Captured, None);
                            }
                        }
                        (event::Status::Ignored, None)
                    }
                },
                mouse::Event::WheelScrolled { delta } => match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        if y < 0.0 && self.scaling > Self::MIN_SCALING
                            || y > 0.0 && self.scaling < Self::MAX_SCALING
                        {
                            let old_scaling = self.scaling;

                            self.scaling = (self.scaling * (1.0 + y / 30.0))
                                .max(Self::MIN_SCALING)
                                .min(Self::MAX_SCALING);

                            if let Some(cursor_to_center) = cursor.position_from(bounds.center()) {
                                let factor = self.scaling - old_scaling;

                                self.translation = self.translation
                                    - Vector::new(
                                        cursor_to_center.x * factor / (old_scaling * old_scaling),
                                        cursor_to_center.y * factor / (old_scaling * old_scaling),
                                    );
                            }

                            self.cache.clear();
                        }

                        (event::Status::Captured, None)
                    }
                },
                _ => (event::Status::Ignored, None),
            },
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let content = self.cache.draw(bounds.size(), |frame: &mut Frame| {
            frame.with_save(|frame| {
                frame.scale(self.scaling);
                frame.translate(self.translation);
                if let Some(i) = self.selected {
                    self.transactions.get(&i).unwrap().draw(
                        frame,
                        self.scaling,
                        true,
                        &self.transactions,
                    );
                } else {
                    for tx in self.transactions.values() {
                        tx.draw(frame, self.scaling, false, &self.transactions);
                    }
                }
            });
        });

        vec![content]
    }

    fn mouse_interaction(&self, _bounds: Rectangle, _cursor: Cursor) -> mouse::Interaction {
        match self.interaction {
            Interaction::Hovering => mouse::Interaction::Pointer,
            Interaction::Panning { .. } => mouse::Interaction::Grabbing,
            _ => mouse::Interaction::default(),
        }
    }
}

#[derive(Debug)]
pub struct TxNode {
    pub position: Point,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<OutputNode>,
    txid: Txid,
}

impl TxNode {
    const BODY_WIDTH: f32 = 120.0;
    const BODY_HEIGHT: f32 = 40.0;
    const OUTPUT_GAP: f32 = 80.0;
    const DISTANCE_FROM_OUTPUTS: f32 = 200.0;

    pub fn new(
        transaction: Transaction,
        previous_transactions: &BTreeMap<Txid, TxNode>,
        default_position: Point,
        network: Network,
    ) -> Self {
        let mut inputs: Vec<&OutputNode> = Vec::new();
        for input in &transaction.input {
            if let Some(previous) = previous_transactions.get(&input.previous_output.txid) {
                inputs.push(&previous.outputs[input.previous_output.vout as usize])
            }
        }

        let mut position = if inputs.is_empty() {
            default_position
        } else {
            Self::position_from_inputs(inputs)
        };

        // check that a node is not too close.
        for node in previous_transactions.values() {
            let minimal_distance =
                (Self::total_height(transaction.output.len()) + node.height()) / 2.0;
            if node.position.x == position.x && node.position.distance(position) <= minimal_distance
            {
                if position.y > node.position.y {
                    position.y += minimal_distance - node.position.distance(position) + 100.0
                } else {
                    position.y -= minimal_distance + node.position.distance(position) - 100.0
                }
            }
        }

        let mut y = position.y + Self::BODY_HEIGHT / 2.0
            - Self::total_height(transaction.output.len()) / 2.0;
        let mut outputs: Vec<OutputNode> = Vec::new();
        for output in transaction.output.iter() {
            outputs.push(OutputNode::new(
                Point::new(
                    position.x + Self::BODY_WIDTH + Self::DISTANCE_FROM_OUTPUTS,
                    y,
                ),
                output,
                network,
            ));
            y += Self::OUTPUT_GAP;
        }
        Self {
            txid: transaction.txid(),
            inputs: transaction.input,
            position,
            outputs,
        }
    }

    pub fn hovered(&self, p: Point) -> bool {
        p.x >= self.position.x
            && p.x <= self.position.x + Self::BODY_WIDTH
            && p.y >= self.position.y
            && p.y <= self.position.y + Self::BODY_HEIGHT
    }

    pub fn position_from_inputs(inputs: Vec<&OutputNode>) -> Point {
        let mut min_y = 0.0;
        let mut max_y = 0.0;
        let mut max_x = 0.0;
        for input in &inputs {
            if input.position.y < min_y || min_y == 0.0 {
                min_y = input.position.y;
            }
            if input.position.y >= max_y || max_y == 0.0 {
                max_y = input.position.y;
            }
            if input.position.x >= max_x || max_x == 0.0 {
                max_x = input.position.x;
            }
        }

        Point::new(
            max_x + Self::DISTANCE_FROM_OUTPUTS,
            min_y + (max_y - min_y) / 2.0 - Self::BODY_HEIGHT / 2.0,
        )
    }

    pub fn total_height(n_outputs: usize) -> f32 {
        (n_outputs as f32 - 1.0) * Self::OUTPUT_GAP
    }

    pub fn height(&self) -> f32 {
        Self::total_height(self.outputs.len())
    }

    pub fn width(&self) -> f32 {
        Self::BODY_WIDTH + Self::DISTANCE_FROM_OUTPUTS
    }

    pub fn connect_right(&self) -> Point {
        self.position + Vector::new(Self::BODY_WIDTH, Self::BODY_HEIGHT / 2.0)
    }

    pub fn connect_left(&self) -> Point {
        self.position + Vector::new(0.0, Self::BODY_HEIGHT / 2.0)
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        scale: f32,
        selected: bool,
        previous_transactions: &BTreeMap<Txid, TxNode>,
    ) {
        let rectangle = Path::rectangle(
            self.position,
            iced::Size::new(Self::BODY_WIDTH, Self::BODY_HEIGHT),
        );
        frame.stroke(&rectangle, Stroke::default().with_width(2.0));

        // txid
        let mut text = canvas::Text::from(&self.txid.to_string()[0..5]);
        text.horizontal_alignment = alignment::Horizontal::Center;
        text.vertical_alignment = alignment::Vertical::Center;
        text.size = text.size * scale;
        text.position =
            self.position + Vector::new(Self::BODY_WIDTH / 2.0, Self::BODY_HEIGHT / 2.0);
        frame.fill_text(text);

        // outputs
        for (i, output) in self.outputs.iter().enumerate() {
            output.draw(frame, i as u32, scale, selected, false);
            let link = Path::new(|p| {
                p.move_to(self.connect_right());
                p.bezier_curve_to(
                    self.connect_right() + Vector::new(25.0, 0.0),
                    output.connect_left() + Vector::new(-25.0, 0.0),
                    output.connect_left(),
                );
            });
            frame.stroke(&link, Stroke::default().with_width(2.0));
        }

        for input in self.inputs.iter() {
            if let Some(previous) = previous_transactions.get(&input.previous_output.txid) {
                let node = &previous.outputs[input.previous_output.vout as usize];
                if selected {
                    node.draw(frame, input.previous_output.vout, scale, selected, true);
                }
                let link = Path::new(|p| {
                    p.move_to(node.connect_right());
                    p.bezier_curve_to(
                        node.connect_right() + Vector::new(30.0, 0.0),
                        self.connect_left() + Vector::new(-30.0, 0.0),
                        self.connect_left(),
                    );
                });
                frame.stroke(&link, Stroke::default().with_width(2.0));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputNode {
    position: Point,
    address: bitcoin::Address,
    value: bitcoin::Amount,
}

impl OutputNode {
    const TXO_RADIUS: f32 = 10.0;
    pub fn new(position: Point, txo: &TxOut, network: Network) -> Self {
        Self {
            position,
            address: bitcoin::Address::from_script(&txo.script_pubkey, network).unwrap(),
            value: bitcoin::Amount::from_sat(txo.value),
        }
    }
    pub fn connect_left(&self) -> Point {
        self.position + Vector::new(-Self::TXO_RADIUS, 0.0)
    }
    pub fn connect_right(&self) -> Point {
        self.position + Vector::new(Self::TXO_RADIUS, 0.0)
    }
    pub fn draw(&self, frame: &mut Frame, vout: u32, scale: f32, selected: bool, as_input: bool) {
        if selected {
            let mut address = canvas::Text::from(format!("#{}: {}", vout, self.address));
            address.vertical_alignment = alignment::Vertical::Center;
            address.size = address.size * scale;
            if as_input {
                address.horizontal_alignment = alignment::Horizontal::Right;
                address.position = self.position + Vector::new(-Self::TXO_RADIUS - 15.0, 0.0);
            } else {
                address.horizontal_alignment = alignment::Horizontal::Left;
                address.position = self.position + Vector::new(Self::TXO_RADIUS + 15.0, 0.0);
            }
            frame.fill_text(address);
        }
        let mut amount = canvas::Text::from(format!("{:.8} BTC", self.value.as_btc(),));
        amount.horizontal_alignment = alignment::Horizontal::Center;
        amount.vertical_alignment = alignment::Vertical::Center;
        amount.size = amount.size * scale;
        amount.position = self.position + Vector::new(0.0, -Self::TXO_RADIUS - 15.0);
        frame.fill_text(amount);
        let circle = Path::circle(self.position, Self::TXO_RADIUS);
        frame.stroke(&circle, Stroke::default().with_width(2.0));
        frame.fill(&circle, Color::BLACK);
    }
}

#[derive(Debug)]
enum Interaction {
    None,
    Hovering,
    Panning { translation: Vector, start: Point },
}
