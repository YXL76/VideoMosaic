use {
    super::{Step, StepMessage},
    crate::{states::State, styles::spacings},
    iced::{
        scrollable, slider, Checkbox, Column, Element, Length, Radio, Row, Scrollable, Slider,
        Text, Toggler,
    },
    mosaic_video_diff::{CalculationUnit, ColorSpace, DistanceAlgorithm, Filter},
};

#[derive(Default)]
pub struct ChooseMethod {
    scroll: scrollable::State,
    k_slider: slider::State,
    size_slider: slider::State,
    quad_slider: slider::State,
}

impl<'a> Step<'a> for ChooseMethod {
    fn title(&self, state: &State) -> &str {
        state.i18n.choose_method
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        let Self {
            scroll,
            k_slider,
            size_slider,
            quad_slider,
        } = self;

        let calc_unit = [
            CalculationUnit::Average,
            CalculationUnit::Pixel,
            CalculationUnit::KMeans,
        ]
        .into_iter()
        .fold(
            Column::new()
                .spacing(spacings::_6)
                .push(Text::new(state.i18n.calc_unit).size(spacings::_8)),
            |col, item| {
                col.push(
                    Radio::new(
                        item,
                        cu_label(&item, state),
                        Some(state.config.calc_unit),
                        StepMessage::CalculationUnit,
                    )
                    .style(state.theme),
                )
            },
        );

        let color_space = [ColorSpace::RGB, ColorSpace::HSV, ColorSpace::CIELAB]
            .into_iter()
            .fold(
                Column::new()
                    .spacing(spacings::_6)
                    .push(Text::new(state.i18n.color_space).size(spacings::_8)),
                |col, item| {
                    col.push(
                        Radio::new(
                            item,
                            item,
                            Some(state.config.color_space),
                            StepMessage::ColorSpace,
                        )
                        .style(state.theme),
                    )
                },
            );

        let dist_algo = [DistanceAlgorithm::Euclidean, DistanceAlgorithm::CIEDE2000]
            .into_iter()
            .fold(
                Column::new()
                    .spacing(spacings::_6)
                    .push(Text::new(state.i18n.dist_algo).size(spacings::_8)),
                |col, item| {
                    col.push(
                        Radio::new(
                            item,
                            item,
                            Some(state.config.dist_algo),
                            StepMessage::DistanceAlgorithm,
                        )
                        .style(state.theme),
                    )
                },
            );

        let filter = [
            Filter::Nearest,
            Filter::Triangle,
            Filter::CatmullRom,
            Filter::Gaussian,
            Filter::Lanczos3,
        ]
        .into_iter()
        .fold(
            Column::new()
                .spacing(spacings::_6)
                .push(Text::new(state.i18n.sampling).size(spacings::_8)),
            |col, item| {
                col.push(
                    Radio::new(
                        item,
                        fl_label(&item, state),
                        Some(state.config.filter),
                        StepMessage::Filter,
                    )
                    .style(state.theme),
                )
            },
        );

        let k_means = Column::new()
            .spacing(spacings::_6)
            .push(Text::new(state.i18n.k_means).size(spacings::_8))
            .push(
                Row::new()
                    .spacing(spacings::_6)
                    .push(Text::new(format!("K: {}", state.config.k)))
                    .push(
                        Slider::new(k_slider, 1..=5, state.config.k, StepMessage::K)
                            .style(state.theme),
                    ),
            )
            .push(
                Toggler::new(
                    state.config.hamerly,
                    Some(String::from("Hamerly")),
                    StepMessage::Hamerly,
                )
                .style(state.theme),
            );

        let quad_iter = state.config.quad_iter.unwrap_or(256) as u16;
        let config = Column::new()
            .spacing(spacings::_6)
            .push(Text::new(state.i18n.configuration).size(spacings::_8))
            .push(
                Row::new()
                    .spacing(spacings::_6)
                    .push(Text::new(format!(
                        "{}: {}",
                        state.i18n.size,
                        state.config.size * 10
                    )))
                    .push(
                        Slider::new(size_slider, 3..=30, state.config.size, StepMessage::Size)
                            .style(state.theme),
                    ),
            )
            .push(
                Row::new()
                    .spacing(spacings::_6)
                    .push(
                        Checkbox::new(
                            state.config.quad_iter.is_some(),
                            format!("{}: {}", state.i18n.quad, quad_iter),
                            StepMessage::Quad,
                        )
                        .style(state.theme),
                    )
                    .push(
                        Slider::new(quad_slider, 256..=2048, quad_iter, StepMessage::QuadValue)
                            .width(Length::Fill)
                            .style(state.theme),
                    ),
            );

        let l = Length::FillPortion(1);
        Scrollable::new(scroll)
            .spacing(spacings::_6)
            .push(
                [calc_unit, color_space, dist_algo]
                    .into_iter()
                    .fold(Row::new().spacing(spacings::_8), |r, i| r.push(i.width(l))),
            )
            .push(
                [filter, k_means, config]
                    .into_iter()
                    .fold(Row::new().spacing(spacings::_8), |r, i| r.push(i.width(l))),
            )
            .height(Length::Fill)
            .style(state.theme)
            .into()
    }
}

fn cu_label(item: &CalculationUnit, state: &State) -> &'static str {
    match item {
        CalculationUnit::Average => state.i18n.average,
        CalculationUnit::Pixel => state.i18n.pixel,
        CalculationUnit::KMeans => state.i18n.k_means,
    }
}

fn fl_label(item: &Filter, state: &State) -> &'static str {
    match item {
        Filter::Nearest => state.i18n.nearest,
        Filter::Triangle => state.i18n.triangle,
        Filter::CatmullRom => state.i18n.catmull_rom,
        Filter::Gaussian => state.i18n.gaussian,
        Filter::Lanczos3 => state.i18n.lanczos3,
    }
}
