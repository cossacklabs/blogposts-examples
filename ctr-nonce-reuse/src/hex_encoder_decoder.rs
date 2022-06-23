use crate::{Message, Tab};
use iced::text_input::{self, TextInput};
use iced::{Alignment, Column, Container, Element, Length, Row, Text};
use iced_aw::TabLabel;

use crate::ctr_game::CtrGame;

#[derive(Default)]
pub struct HexEncDecTab {
    input_hex: text_input::State,
    data_hex: String,

    input_ascii: text_input::State,
    data_ascii: String,
}

#[derive(Debug, Clone)]
pub enum HexEncDecMessage {
    HexDataChanged(String),
    AsciiDataChanged(String),
}

impl HexEncDecTab {
    pub fn new() -> Self {
        HexEncDecTab { ..Self::default() }
    }

    pub fn update(&mut self, message: HexEncDecMessage) {
        match message {
            HexEncDecMessage::HexDataChanged(mut data) => {
                data.truncate(100);

                self.data_hex = data.clone();
                self.data_ascii = CtrGame::hex_to_text(data);
            }
            HexEncDecMessage::AsciiDataChanged(mut data) => {
                data.truncate(100);

                self.data_ascii = data.clone();
                self.data_hex = CtrGame::text_to_hex(data);
            }
        }
    }
}

impl Tab for HexEncDecTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("HEX ENCODER/DECODER")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(String::from("HEX ENC/DEX"))
        //TabLabel::IconText(Icon::User.into(), self.title())
    }

    fn content(&mut self) -> Element<'_, Self::Message> {
        let title = Text::new(self.title()).size(50).color([0.5, 0.5, 0.5]);

        let desc = Text::new("This is a convenient encoder designed for ASCII <-> Hex translations. It won't work for decoding hex to byte streams and will just show '[unprintable]' in that case.");

        let input_hex_title = Text::new("Enter hex here...").width(Length::Fill);
        let input_hex = TextInput::new(
            &mut self.input_hex,
            "Enter hex here...",
            &self.data_hex,
            HexEncDecMessage::HexDataChanged,
        )
        //.size()
        .width(Length::Fill)
        .padding(15);

        let input_text_title = Text::new("Enter text here...").width(Length::Fill);
        let input_ascii = TextInput::new(
            &mut self.input_ascii,
            "Enter text here...",
            &self.data_ascii,
            HexEncDecMessage::AsciiDataChanged,
        )
        //.size(30)
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
            .padding(15)
            .push(title)
            .push(desc)
            .push(input_title_row)
            .push(input_row);

        let container: Element<'_, HexEncDecMessage> = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();

        container.map(Message::HexEncDec)
    }
}
