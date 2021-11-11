use {
    super::{Step, StepMessage},
    crate::{
        states::State,
        styles::spacings,
        widgets::{btn_icon, btn_text, pri_btn},
    },
    iced::{
        alignment, button, image, scrollable, Column, Element, Image, Length, Row, Scrollable,
        Space, Text,
    },
    image_diff::first_frame,
};

#[derive(Default)]
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

        let ci_i = btn_icon("\u{f104} ");
        let ci_l = btn_text(state.i18n.choose_image);
        let cv_i = btn_icon("\u{f0fc} ");
        let cv_l = btn_text(state.i18n.choose_video);

        let left_side = Column::new()
            .padding(spacings::_4)
            .spacing(spacings::_4)
            .push(Space::with_height(Length::FillPortion(1)))
            .push(
                pri_btn(image_btn, ci_i, ci_l, spacings::_32, &state.theme)
                    .on_press(StepMessage::TargetType(TargetType::Image)),
            )
            .push(Space::with_height(Length::Units(spacings::_3)))
            .push(
                pri_btn(video_btn, cv_i, cv_l, spacings::_32, &state.theme)
                    .on_press(StepMessage::TargetType(TargetType::Video)),
            )
            .push(Space::with_height(Length::FillPortion(1)))
            .max_height(spacings::_128 as u32);

        let mut right_side = Column::new()
            .padding(spacings::_4)
            .width(Length::Fill)
            .align_items(alignment::Alignment::Center);
        let path = state.target_path.to_str().unwrap_or("");
        match state.target_type {
            TargetType::Image => {
                right_side = right_side
                    .push(Image::new(path).width(Length::Fill))
                    .push(Text::new(path))
            }
            TargetType::Video => {
                if let Ok((width, height, pixels)) = first_frame(&state.target_path) {
                    println!("{}", pixels.len());
                    let img = image::Handle::from_pixels(width, height, pixels);
                    right_side = right_side
                        .push(Image::new(img).width(Length::Fill))
                        .push(Text::new(path));
                }
            }
            _ => (),
        };

        Scrollable::new(scroll)
            .push(
                Row::new()
                    .spacing(spacings::_4)
                    .push(left_side)
                    .push(right_side),
            )
            .height(Length::Fill)
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
