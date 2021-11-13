use {
    super::{Step, StepMessage},
    crate::{states::State, styles::spacings},
    iced::{Column, Element, Length, Radio, Row, Text},
    image_diff::{CalculationUnit, ColorSpace, DistanceAlgorithm},
};

#[derive(Default)]
pub struct ChooseMethod {}

impl<'a> Step<'a> for ChooseMethod {
    fn title(&self, state: &State) -> &str {
        state.i18n.choose_method
    }

    fn can_next(&self, _state: &State) -> bool {
        true
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        for i in state.libraries.values() {
            image_diff::ProcessFactory::new(image_diff::ProcessConfig::new(
                50,
                CalculationUnit::Average,
                ColorSpace::HSV,
                DistanceAlgorithm::Euclidean,
            ))
            .run(&state.target_path, i);
            break;
        }

        let calc_unit = [
            CalculationUnit::Average,
            CalculationUnit::Pixel,
            CalculationUnit::KMeans,
        ]
        .iter()
        .fold(
            Column::new()
                .spacing(spacings::_8)
                .push(Text::new(state.i18n.calc_unit).size(spacings::_8)),
            |col, &item| {
                col.push(Radio::new(
                    item,
                    calc_unit_label(&item, state),
                    Some(state.calc_unit),
                    StepMessage::CalculationUnit,
                ))
            },
        );

        let color_space = [ColorSpace::RGB, ColorSpace::HSV, ColorSpace::CIEXYZ]
            .iter()
            .fold(
                Column::new()
                    .spacing(spacings::_8)
                    .push(Text::new(state.i18n.color_space).size(spacings::_8)),
                |col, &item| {
                    col.push(Radio::new(
                        item,
                        color_space_label(&item, state),
                        Some(state.color_space),
                        StepMessage::ColorSpace,
                    ))
                },
            );

        let dist_algo = [DistanceAlgorithm::Euclidean, DistanceAlgorithm::CIEDE2000]
            .iter()
            .fold(
                Column::new()
                    .spacing(spacings::_8)
                    .push(Text::new(state.i18n.dist_algo).size(spacings::_8)),
                |col, &item| {
                    col.push(Radio::new(
                        item,
                        dist_algo_label(&item, state),
                        Some(state.dist_algo),
                        StepMessage::DistanceAlgorithm,
                    ))
                },
            );

        Row::new()
            .push(calc_unit.width(Length::FillPortion(1)))
            .push(color_space.width(Length::FillPortion(1)))
            .push(dist_algo.width(Length::FillPortion(1)))
            .into()
    }
}

fn calc_unit_label(item: &CalculationUnit, state: &State) -> &'static str {
    match item {
        CalculationUnit::Average => state.i18n.calc_unit_average,
        CalculationUnit::Pixel => state.i18n.calc_unit_pixel,
        CalculationUnit::KMeans => state.i18n.calc_unit_k_means,
    }
}

fn color_space_label(item: &ColorSpace, state: &State) -> &'static str {
    match item {
        ColorSpace::RGB => state.i18n.color_space_rgb,
        ColorSpace::HSV => state.i18n.color_space_hsv,
        ColorSpace::CIEXYZ => state.i18n.color_space_ciexyz,
    }
}

fn dist_algo_label(item: &DistanceAlgorithm, state: &State) -> &'static str {
    match item {
        DistanceAlgorithm::Euclidean => state.i18n.dist_algo_euclidean,
        DistanceAlgorithm::CIEDE2000 => state.i18n.dist_algo_ciede2000,
    }
}
