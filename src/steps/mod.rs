mod choose_library;
mod choose_method;
mod choose_target;

use {
    crate::states::State,
    choose_library::ChooseLibrary,
    choose_method::ChooseAlgorithm,
    choose_target::ChooseTarget,
    iced::Element,
    image_diff::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    std::path::PathBuf,
};

pub use choose_target::TargetType;

#[derive(Debug, Clone)]
pub enum StepMessage {
    TargetType(TargetType),
    AddLocalLibrary,
    DeleteLocalLibrary(PathBuf),
    Spider,
    CalculationUnit(CalculationUnit),
    ColorSpace(ColorSpace),
    DistanceAlgorithm(DistanceAlgorithm),
}

trait Step<'a> {
    fn title(&self, state: &State) -> &str;

    fn can_next(&self, state: &State) -> bool;

    fn view(&mut self, state: &State) -> Element<StepMessage>;
}

const STEPS_NUM: usize = 3;

pub struct Steps<'a> {
    cur: usize,
    steps: [Box<dyn Step<'a>>; STEPS_NUM],
}

impl Steps<'_> {
    fn new() -> Self {
        Self {
            cur: 0,
            steps: [
                Box::new(ChooseTarget::default()),
                Box::new(ChooseLibrary::default()),
                Box::new(ChooseAlgorithm::default()),
            ],
        }
    }

    pub fn title(&self, state: &State) -> &str {
        self.steps[self.cur].title(state)
    }

    pub fn can_back(&self) -> bool {
        self.cur > 0
    }

    pub fn can_next(&self, state: &State) -> bool {
        self.cur < STEPS_NUM - 1 && self.steps[self.cur].can_next(state)
    }

    pub fn back(&mut self) {
        if self.can_back() {
            self.cur -= 1;
        }
    }

    pub fn next(&mut self, state: &State) {
        if self.can_next(state) {
            self.cur += 1;
        }
    }

    pub fn view(&mut self, state: &State) -> Element<StepMessage> {
        self.steps[self.cur].view(state)
    }
}

impl Default for Steps<'_> {
    fn default() -> Self {
        Self::new()
    }
}
