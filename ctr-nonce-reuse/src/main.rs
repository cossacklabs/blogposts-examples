use iced::{
    alignment::{Horizontal, Vertical},
    Column, Container, Element, Length, Sandbox,
};

use iced_aw::{TabLabel, Tabs};

mod ctr_game;

mod ctr_game_tab;
use ctr_game_tab::{CTRGameMessage, CTRGameTab};

mod xor_tool;
use xor_tool::{XorToolMessage, XorToolTab};

mod hex_encoder_decoder;
use hex_encoder_decoder::{HexEncDecMessage, HexEncDecTab};

mod theme;
use theme::Theme;

const TAB_PADDING: u16 = 0;

fn main() -> iced::Result {
    let mut window_settings = iced::Settings::default();
    window_settings.window.size = (960, 560);
    TabBarExample::run(window_settings)
}

struct TabBarExample {
    active_tab: usize,
    ctr_game_tab: CTRGameTab,
    xor_tool_tab: XorToolTab,
    hex_encode_decode_tab: HexEncDecTab,
}

#[derive(Clone, Debug)]
enum Message {
    TabSelected(usize),
    CtrGame(CTRGameMessage),
    XorTool(XorToolMessage),
    HexEncDec(HexEncDecMessage),
}

impl Sandbox for TabBarExample {
    type Message = Message;

    fn new() -> Self {
        TabBarExample {
            active_tab: 0,
            ctr_game_tab: CTRGameTab::new(),
            xor_tool_tab: XorToolTab::new(),
            hex_encode_decode_tab: HexEncDecTab::new(),
        }
    }

    fn title(&self) -> String {
        String::from("CTR (nonce reuse) Game")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::TabSelected(selected) => self.active_tab = selected,
            Message::CtrGame(message) => self.ctr_game_tab.update(message),
            Message::XorTool(message) => self.xor_tool_tab.update(message),
            Message::HexEncDec(message) => self.hex_encode_decode_tab.update(message),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let theme = Theme::default();

        Tabs::new(self.active_tab, Message::TabSelected)
            .push(self.ctr_game_tab.tab_label(), self.ctr_game_tab.view())
            .push(self.xor_tool_tab.tab_label(), self.xor_tool_tab.view())
            .push(
                self.hex_encode_decode_tab.tab_label(),
                self.hex_encode_decode_tab.view(),
            )
            .tab_bar_style(theme)
            .icon_size(0)
            .tab_bar_position(iced_aw::TabBarPosition::Top)
            .into()
    }
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&mut self) -> Element<'_, Self::Message> {
        let column = Column::new()
            .spacing(20)
            //.push(Text::new(self.title()).size(HEADER_SIZE))
            .push(self.content());

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&mut self) -> Element<'_, Self::Message>;
}
