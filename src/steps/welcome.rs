use {
    super::{Step, StepMessage},
    crate::states::State,
    iced::{Column, Element, Text},
};

pub struct Welcome;

impl<'a> Step<'a> for Welcome {
    fn title(&self, _state: &State) -> &str {
        "Welcome"
    }

    fn can_next(&self) -> bool {
        true
    }

    fn view(&mut self, _state: &State) -> Element<StepMessage> {
        Column::new()
            .push(Text::new(
                "This is a simple tour meant to showcase a bunch of widgets \
                 that can be easily implemented on top of Iced.",
            ))
            .push(Text::new(
                "Iced is a cross-platform GUI library for Rust focused on \
                 simplicity and type-safety. It is heavily inspired by Elm.",
            ))
            .push(Text::new(
                "It was originally born as part of Coffee, an opinionated \
                 2D game engine for Rust.",
            ))
            .push(Text::new(
                "On native platforms, Iced provides by default a renderer \
                 built on top of wgpu, a graphics library supporting Vulkan, \
                 Metal, DX11, and DX12.",
            ))
            .push(Text::new(
                "Additionally, this tour can also run on WebAssembly thanks \
                 to dodrio, an experimental VDOM library for Rust.",
            ))
            .push(Text::new(
                "You will need to interact with the UI in order to reach the \
                 end!",
            ))
            .push(
                Text::new(
                    "You will need to interact with the UI in order to reach the \
                 end!",
                )
                .size(50),
            )
            .push(
                Text::new(
                    "You will need to interact with the UI in order to reach the \
                 end!",
                )
                .size(50),
            )
            .push(
                Text::new(
                    "You will need to interact with the UI in order to reach the \
                 end!",
                )
                .size(50),
            )
            .into()
    }
}
