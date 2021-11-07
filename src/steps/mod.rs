mod choose_target;
mod welcome;

use {
    crate::states::{I18n, State, TargetType},
    choose_target::ChooseTarget,
    iced::Element,
    welcome::Welcome,
};

#[derive(Debug, Clone, Copy)]
pub enum StepMessage {
    TargetType(TargetType),
}

trait Step<'a> {
    fn title(&self, i18n: &I18n) -> &str;

    fn can_next(&self) -> bool;

    fn view(&mut self, state: &State, i18n: &I18n) -> Element<StepMessage>;
}

const STEPS_NUM: usize = 2;

pub struct Steps<'a> {
    cur: usize,
    steps: [Box<dyn Step<'a>>; STEPS_NUM],
}

impl Steps<'_> {
    fn new() -> Self {
        Self {
            cur: 0,
            steps: [Box::new(Welcome), Box::new(ChooseTarget::default())],
        }
    }

    pub fn title(&self, i18n: &I18n) -> &str {
        self.steps[self.cur].title(i18n)
    }

    pub fn can_back(&self) -> bool {
        self.cur > 0
    }

    pub fn can_next(&self) -> bool {
        self.cur < STEPS_NUM - 1 && self.steps[self.cur].can_next()
    }

    pub fn back(&mut self) {
        if self.can_back() {
            self.cur -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.can_next() {
            self.cur += 1;
        }
    }

    pub fn view(&mut self, state: &State, i18n: &I18n) -> Element<StepMessage> {
        self.steps[self.cur].view(state, i18n)
    }
}

impl Default for Steps<'_> {
    fn default() -> Self {
        Self::new()
    }
}
