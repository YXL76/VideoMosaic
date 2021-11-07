use {
    super::{Step, StepMessage},
    crate::{
        states::{I18n, State, TargetType},
        styles::spacings,
        widgets::pri_btn,
    },
    iced::{button, Column, Element, Image, Length, Row, Space, Text},
};

#[derive(Default)]
pub struct ChooseTarget {
    image_btn: button::State,
    video_btn: button::State,
}

impl<'a> Step<'a> for ChooseTarget {
    fn title(&self, i18n: &I18n) -> &str {
        i18n.choose_target
    }

    fn can_next(&self) -> bool {
        true
    }

    fn view(&mut self, state: &State, i18n: &I18n) -> Element<StepMessage> {
        let left_side: Element<_> = Column::new()
            .padding(spacings::_4)
            .spacing(spacings::_4)
            .push(Space::with_height(Length::FillPortion(1)))
            .push(
                pri_btn(&mut self.image_btn, i18n.choose_image, spacings::_32)
                    .on_press(StepMessage::TargetType(TargetType::Image)),
            )
            .push(Space::with_height(Length::Units(spacings::_3)))
            .push(
                pri_btn(&mut self.video_btn, i18n.choose_video, spacings::_32)
                    .on_press(StepMessage::TargetType(TargetType::Video)),
            )
            .push(Space::with_height(Length::FillPortion(1)))
            .max_height(spacings::_128 as u32)
            .into();

        let path = state.target_path.to_str().unwrap_or("");
        let right_side: Element<_> = Column::new()
            .push(Image::new(path))
            .push(Text::new(path))
            .padding(spacings::_4)
            .width(Length::Fill)
            .into();

        let content: Element<_> = Row::new()
            .spacing(spacings::_4)
            .push(left_side)
            .push(right_side)
            .into();

        content
    }
}
