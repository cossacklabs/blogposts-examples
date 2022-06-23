use iced::alignment::Horizontal;
use iced::button::{self, Button};
use iced::text_input::{self, TextInput};
use iced::{scrollable, Alignment, Column, Container, Element, Length, Row, Scrollable, Text};
use iced_aw::{modal, Card, Modal, TabLabel};

use crate::{ctr_game, Message, Tab};

#[derive(Default)]
pub(crate) struct CTRGameTab {
    ctr_game: ctr_game::CtrGame,

    input_eavesdrops: Vec<text_input::State>,
    known_plaintext: text_input::State,

    data_flag: String,
    input_flag: text_input::State,

    button_restart: button::State,
    button_submit_flag: button::State,

    scrollable_state: scrollable::State,

    modal_state: modal::State<ModalState>,
}

#[derive(Default)]
struct ModalState {
    cancel_state: button::State,
    ok_state: button::State,
}

#[derive(Debug, Clone)]
pub(crate) enum CTRGameMessage {
    EavesdropChanged(String),
    FlagSubmit,
    FlagChanged(String),
    RestartPressed,
    CloseModal,
    CancelButtonPressed,
    OkButtonPressed,
}

impl CTRGameTab {
    pub fn new() -> Self {
        CTRGameTab {
            ctr_game: ctr_game::CtrGame::new(),
            ..Self::default()
        }
    }

    pub fn update(&mut self, message: CTRGameMessage) {
        match message {
            CTRGameMessage::EavesdropChanged(_data) => {}
            CTRGameMessage::FlagChanged(data) => {
                self.data_flag = data;
            }
            CTRGameMessage::RestartPressed => {
                self.ctr_game.restart();
            }
            CTRGameMessage::FlagSubmit => {
                if self.ctr_game.submit_flag(self.data_flag.clone()) {
                    self.modal_state.show(true);
                }
            }
            CTRGameMessage::CloseModal => self.modal_state.show(false),
            CTRGameMessage::CancelButtonPressed => self.modal_state.show(false),
            CTRGameMessage::OkButtonPressed => self.modal_state.show(false),
        }
    }
}

impl Tab for CTRGameTab {
    type Message = Message;

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(String::from("CTR Game"))
    }

    fn title(&self) -> String {
        String::from("Flaw CTR (nonce reuse) Game")
    }

    fn content(&mut self) -> Element<'_, Self::Message> {
        if self.ctr_game.get_hex_ciphertexts().is_empty() {
            self.ctr_game.restart();
        }

        let title = Text::new(self.title()).size(40).color([0.5, 0.5, 0.5]);

        let left_column_desc = Text::new("Known eavesdropped message:");
        let known_plaintext_text = TextInput::new(
            &mut self.known_plaintext,
            "",
            &self.ctr_game.get_known_plaintext(),
            CTRGameMessage::EavesdropChanged,
        )
        .padding(15)
        .width(Length::Fill);

        let left_column_title = Text::new("Eavesdropped Messages:").size(30);

        let mut left_column = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(left_column_desc)
            .push(known_plaintext_text)
            .push(left_column_title);

        let eavesdropped_msg = self.ctr_game.get_hex_ciphertexts();
        let delta: i32 = (eavesdropped_msg.len() - self.input_eavesdrops.len()) as i32;
        if delta >= 0 {
            for _ in 0..delta {
                self.input_eavesdrops.push(text_input::State::default());
            }
        } else {
            let delta = delta.abs();
            for _ in 0..delta {
                self.input_eavesdrops.pop();
            }
        }

        for (input_eavesdrop, eavesdrop_message) in
            self.input_eavesdrops.iter_mut().zip(&eavesdropped_msg)
        {
            left_column = left_column.push(
                TextInput::new(
                    input_eavesdrop,
                    "",
                    eavesdrop_message,
                    CTRGameMessage::EavesdropChanged,
                )
                .width(Length::Fill)
                .padding(15),
            );
        }

        let right_column_title = Text::new("Flag Input").size(40);
        let right_column_desc = Text::new("Input flag here. FLAG{}");

        let flag_input = TextInput::new(
            &mut self.input_flag,
            "Type the flag here...",
            &self.data_flag,
            CTRGameMessage::FlagChanged,
        )
        .size(30)
        .padding(15)
        .on_submit(CTRGameMessage::FlagSubmit);

        let restart_button = Button::new(&mut self.button_restart, Text::new("RESTART"))
            .on_press(CTRGameMessage::RestartPressed)
            .padding(10);

        let submit_flag_button = Button::new(&mut self.button_submit_flag, Text::new("SUBMIT"))
            .on_press(CTRGameMessage::FlagSubmit)
            .padding(10);

        let button_row = Row::new()
            .width(Length::Shrink)
            .spacing(20)
            .padding(10)
            .align_items(Alignment::Center)
            .push(restart_button)
            .push(submit_flag_button);

        let right_column = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(right_column_title)
            .push(right_column_desc)
            .push(flag_input)
            .push(button_row);

        let content_row = Row::new()
            .width(Length::Fill)
            .spacing(20)
            .align_items(Alignment::Center)
            .push(left_column)
            .push(right_column);

        let main_column = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .align_items(Alignment::Center)
            .padding(15)
            .push(title)
            .push(content_row);

        let scrollable_content = Scrollable::new(&mut self.scrollable_state)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .scrollbar_width(4)
            .scroller_width(4)
            .scrollbar_margin(5)
            .push(main_column);

        let content = Container::new(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        let modal: Element<'_, CTRGameMessage> =
            Modal::new(&mut self.modal_state, content, |state| {
                Card::new(
                    Text::new("YOU WON"),
                    Text::new(
                        "Congratulations, You've won!\nAnd rememner, never roll your own crypto!",
                    ),
                )
                .foot(
                    Row::new()
                        .spacing(10)
                        .padding(5)
                        .width(Length::Fill)
                        .push(
                            Button::new(
                                &mut state.cancel_state,
                                Text::new("Cancel").horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .on_press(CTRGameMessage::CancelButtonPressed),
                        )
                        .push(
                            Button::new(
                                &mut state.ok_state,
                                Text::new("Ok").horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .on_press(CTRGameMessage::OkButtonPressed),
                        ),
                )
                .max_width(300)
                .on_close(CTRGameMessage::CloseModal)
                .into()
            })
            .backdrop(CTRGameMessage::CloseModal)
            .on_esc(CTRGameMessage::CloseModal)
            .into();

        modal.map(Message::CtrGame)
    }
}
