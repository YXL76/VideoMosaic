use {
    super::{Step, StepMessage},
    crate::{states::State, styles::spacings, widgets::pri_btn},
    iced::{
        alignment, button, scrollable, Column, Element, Image, Length, Row, Scrollable, Space, Text,
    },
};

#[derive(Default, Copy, Clone)]
pub struct ChooseTarget {
    scroll: scrollable::State,
    image_btn: button::State,
    video_btn: button::State,
}

impl<'a> Step<'a> for ChooseTarget {
    fn title(&self, state: &State) -> &str {
        state.i18n.choose_target
    }

    fn can_next(&self, state: &State) -> bool {
        state.target_type != TargetType::None
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        let Self {
            scroll,
            image_btn,
            video_btn,
        } = self;

        let ci_l = format!("\u{f603} {}", state.i18n.choose_image);
        let cv_l = format!("\u{f5fb} {}", state.i18n.choose_video);

        let left_side = Column::new()
            .padding(spacings::_4)
            .spacing(spacings::_4)
            .push(Space::with_height(Length::FillPortion(1)))
            .push(
                pri_btn(image_btn, ci_l, spacings::_32, &state.theme)
                    .on_press(StepMessage::TargetType(TargetType::Image)),
            )
            .push(Space::with_height(Length::Units(spacings::_3)))
            .push(
                pri_btn(video_btn, cv_l, spacings::_32, &state.theme)
                    .on_press(StepMessage::TargetType(TargetType::Video)),
            )
            .push(Space::with_height(Length::FillPortion(1)));

        let mut right_side = Scrollable::new(scroll)
            .padding(spacings::_4)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(state.theme)
            .align_items(alignment::Alignment::Center);
        if let Some(img) = state.target_preview.as_ref() {
            right_side = right_side
                .push(Image::new(img.clone()).width(Length::Shrink))
                .push(Text::new(state.target_path.to_str().unwrap_or("")));
        }

        Row::new()
            .spacing(spacings::_4)
            .push(left_side)
            .push(right_side)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TargetType {
    None,
    Image,
    Video,
}

impl Default for TargetType {
    fn default() -> Self {
        Self::None
    }
}
