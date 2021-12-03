mod choose_library;
mod choose_method;
mod choose_target;
mod process_preview;

use {
    crate::{
        states::State,
        streams::{crawler, process},
    },
    choose_library::ChooseLibrary,
    choose_method::ChooseMethod,
    choose_target::ChooseTarget,
    iced::{Element, Subscription},
    mosaic_video_diff::{CalculationUnit, ColorSpace, DistanceAlgorithm, Filter},
    process_preview::ProcessPreview,
    std::path::PathBuf,
};

pub use choose_target::TargetType;

#[derive(Debug, Clone)]
pub enum StepMessage {
    TargetType(TargetType),
    AddLocalLibrary,
    DeleteLocalLibrary(PathBuf),
    AddCrawler,
    EditKeyword(usize, String),
    EditNumber(usize, String),
    StartCrawler(usize),
    DeleteCrawler(usize),
    CrawlerMessage(crawler::Progress),
    CalculationUnit(CalculationUnit),
    ColorSpace(ColorSpace),
    DistanceAlgorithm(DistanceAlgorithm),
    Filter(Filter),
    K(u8),
    Hamerly(bool),
    Size(u16),
    Quad(bool),
    QuadValue(u16),
    Start,
    ProcessMessage(process::Progress),
}

trait Step<'a> {
    fn title(&self, state: &State) -> &str;

    fn can_back(&self, _state: &State) -> bool {
        true
    }

    fn can_next(&self, _state: &State) -> bool {
        true
    }

    fn subscription(&self, _state: &State) -> Subscription<StepMessage> {
        Subscription::none()
    }

    fn view(&mut self, state: &State) -> Element<StepMessage>;
}

const STEPS_NUM: usize = 4;

pub struct Steps<'a> {
    cur: usize,
    steps: [Box<dyn Step<'a>>; STEPS_NUM],
}

impl Steps<'_> {
    #[inline(always)]
    fn new() -> Self {
        Self {
            cur: 0,
            steps: [
                Box::new(ChooseTarget::default()),
                Box::new(ChooseLibrary::default()),
                Box::new(ChooseMethod::default()),
                Box::new(ProcessPreview::default()),
            ],
        }
    }

    #[inline(always)]
    pub fn title(&self, state: &State) -> &str {
        self.steps[self.cur].title(state)
    }

    #[inline(always)]
    pub fn can_back(&self, state: &State) -> bool {
        self.cur > 0 && self.steps[self.cur].can_back(state)
    }

    #[inline(always)]
    pub fn can_next(&self, state: &State) -> bool {
        self.cur < STEPS_NUM - 1 && self.steps[self.cur].can_next(state)
    }

    #[inline(always)]
    pub fn back(&mut self, state: &State) {
        if self.can_back(state) {
            self.cur -= 1;
        }
    }

    #[inline(always)]
    pub fn next(&mut self, state: &State) {
        if self.can_next(state) {
            self.cur += 1;
        }
    }

    #[inline(always)]
    pub fn subscription(&self, state: &State) -> Subscription<StepMessage> {
        self.steps[self.cur].subscription(state)
    }

    #[inline(always)]
    pub fn view(&mut self, state: &State) -> Element<StepMessage> {
        self.steps[self.cur].view(state)
    }
}

impl Default for Steps<'_> {
    fn default() -> Self {
        Self::new()
    }
}
