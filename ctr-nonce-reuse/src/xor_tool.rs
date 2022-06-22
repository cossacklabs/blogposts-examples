use crate::{Message, Tab};
use iced::text_input::{self, TextInput};
use iced::{Alignment, Column, Container, Element, Length, Row, Text};
use iced_aw::TabLabel;

use crate::ctr_game::CtrGame;

#[derive(Default)]
pub struct XorToolTab {
    input_hex_1: text_input::State,
    data_hex_1: String,

    input_hex_2: text_input::State,
    data_hex_2: String,

    input_output: text_input::State,
    data_output: String,
}

#[derive(Debug, Clone)]
pub enum XorToolMessage {
    Hex1DataChanged(String),
    Hex2DataChanged(String),
    OutputEvent(String),
}

impl XorToolTab {
    pub fn new() -> Self {
        XorToolTab { ..Self::default() }
    }

    pub fn update(&mut self, message: XorToolMessage) {
        match message {
            XorToolMessage::Hex1DataChanged(data) => {
                self.data_hex_1 = data;

                self.data_output = CtrGame::xor_hex(vec![&self.data_hex_1, &self.data_hex_2]);
            }
            XorToolMessage::Hex2DataChanged(data) => {
                self.data_hex_2 = data;

                self.data_output = CtrGame::xor_hex(vec![&self.data_hex_1, &self.data_hex_2]);
            }
            XorToolMessage::OutputEvent(_data) => {}
        }
    }
}

impl Tab for XorToolTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("XOR TOOL")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
        //TabLabel::IconText(Icon::User.into(), self.title())
    }

    fn content(&mut self) -> Element<'_, Self::Message> {
        let title = Text::new("XOR TOOL").size(50).color([0.5, 0.5, 0.5]);

        let desc = Text::new("Use this form to XOR two hex strings together.");

        let input_hex_title = Text::new("Enter hex here...").width(Length::Fill);
        let input_hex = TextInput::new(
            &mut self.input_hex_1,
            "Enter hex here...",
            &self.data_hex_1,
            XorToolMessage::Hex1DataChanged,
        )
        //.size()
        .width(Length::Fill)
        .padding(15);

        let input_text_title = Text::new("Enter hex here...").width(Length::Fill);
        let input_ascii = TextInput::new(
            &mut self.input_hex_2,
            "Enter hex here...",
            &self.data_hex_2,
            XorToolMessage::Hex2DataChanged,
        )
        //.size(30)
        .width(Length::Fill)
        .padding(15);

        let output_title = Text::new("OUTPUT");
        let input_output = TextInput::new(
            &mut self.input_output,
            "Output will be here...",
            &self.data_output,
            XorToolMessage::OutputEvent,
        )
        .width(Length::Fill)
        .padding(15);

        let input_title_row = Row::new()
            .width(Length::Shrink)
            .align_items(Alignment::Start)
            .spacing(20)
            .push(input_hex_title)
            .push(input_text_title);

        let input_row = Row::new()
            .width(Length::Fill)
            .align_items(Alignment::Fill)
            .spacing(20)
            .push(input_hex)
            .push(input_ascii);

        let content = Column::new()
            .width(Length::Shrink)
            .spacing(20)
            .align_items(Alignment::Fill)
            .push(title)
            .push(desc)
            .push(input_title_row)
            .push(input_row)
            .push(output_title)
            .push(input_output);

        let container: Element<'_, XorToolMessage> = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into();

        container.map(Message::XorTool)
    }
}
