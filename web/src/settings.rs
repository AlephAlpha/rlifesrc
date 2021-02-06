use rlifesrc_lib::{rules::NtLifeGen, Config, NewState, SearchOrder, Symmetry, Transform};
use std::matches;
use wasm_bindgen::prelude::wasm_bindgen;
use yew::{
    html, html::ChangeData, Callback, Component, ComponentLink, Html, Properties, ShouldRender,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["mui", "tabs"])]
    fn activate(tab: &str);
}

pub struct Settings {
    link: ComponentLink<Self>,
    callback: Callback<Config>,
    config: Config,
    rule_is_valid: bool,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub config: Config,
    pub callback: Callback<Config>,
}

pub enum Msg {
    Apply,
    SetWidth(i32),
    SetHeight(i32),
    SetPeriod(i32),
    SetDx(i32),
    SetDy(i32),
    SetTrans(Transform),
    SetSym(Symmetry),
    SetRule(String),
    SetOrder(Option<SearchOrder>),
    SetChoose(NewState),
    SetMax(Option<usize>),
    SetDiag(Option<i32>),
    SetReduce,
    SetSkipSubperiod,
    SetSkipSubsym,
    None,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let rule_is_valid = props.config.rule_string.parse::<NtLifeGen>().is_ok();
        Settings {
            link,
            callback: props.callback,
            config: props.config,
            rule_is_valid,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetWidth(width) => {
                self.config.width = width;
                if self.config.require_square_world() {
                    self.config.height = width;
                }
            }
            Msg::SetHeight(height) => {
                self.config.height = height;
                if self.config.require_square_world() {
                    self.config.width = height;
                }
            }
            Msg::SetPeriod(period) => self.config.period = period,
            Msg::SetDx(dx) => self.config.dx = dx,
            Msg::SetDy(dy) => self.config.dy = dy,
            Msg::SetTrans(transform) => self.config.transform = transform,
            Msg::SetSym(symmetry) => self.config.symmetry = symmetry,
            Msg::SetRule(rule_string) => {
                self.rule_is_valid = rule_string.parse::<NtLifeGen>().is_ok();
                self.config.rule_string = rule_string;
            }
            Msg::SetOrder(search_order) => self.config.search_order = search_order,
            Msg::SetChoose(new_state) => self.config.new_state = new_state,
            Msg::SetMax(max_cell_count) => self.config.max_cell_count = max_cell_count,
            Msg::SetDiag(diagonal_width) => self.config.diagonal_width = diagonal_width,
            Msg::SetReduce => self.config.reduce_max ^= true,
            Msg::SetSkipSubperiod => self.config.skip_subperiod ^= true,
            Msg::SetSkipSubsym => self.config.skip_subsymmetry ^= true,
            Msg::Apply => {
                self.callback.emit(self.config.clone());
                return false;
            }
            Msg::None => return false,
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.config != props.config && {
            self.config = props.config;
            self.rule_is_valid = self.config.rule_string.parse::<NtLifeGen>().is_ok();
            true
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="mui-form">
                { self.apply_button() }
                { self.settings() }
            </div>
        }
    }
}

impl Settings {
    fn apply_button(&self) -> Html {
        let onclick = self.link.callback(|_| {
            activate("pane-world");
            Msg::Apply
        });
        html! {
            <div class="buttons">
                <button class="mui-btn mui-btn--raised"
                    type="submit"
                    onclick=onclick>
                    <i class="fas fa-check"></i>
                    <span>
                        <abbr title="Apply the settings and restart the search.">
                            { "Apply Settings" }
                        </abbr>
                    </span>
                </button>
            </div>
        }
    }

    fn settings(&self) -> Html {
        html! {
            <div id="settings">
                { self.set_rule() }
                { self.set_width() }
                { self.set_height() }
                { self.set_period() }
                { self.set_dx() }
                { self.set_dy() }
                { self.set_diag() }
                { self.set_trans() }
                { self.set_sym() }
                { self.set_max() }
                { self.set_order() }
                { self.set_choose() }
                { self.set_reduce() }
                { self.set_skip_subperiod() }
                { self.set_skip_subsym() }
            </div>
        }
    }

    fn set_rule(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                Msg::SetRule(v)
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_rule">
                    <abbr title="Rule of the cellular automaton. \
                        Supports Life-like, isotropic non-totalistic, hexagonal, \
                        MAP rules, and their corresponding Generations rules.">
                        { "Rule" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_rule"
                    type="text"
                    class=if self.rule_is_valid { "" } else { "mui--is-invalid" }
                    value=self.config.rule_string.clone()
                    onchange=onchange/>
            </div>
        }
    }

    fn set_width(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                Msg::SetWidth(v.parse().unwrap())
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_width">
                    <abbr title="Width of the pattern.">
                        { "Width" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_width"
                    type="number"
                    value=self.config.width
                    min="1"
                    onchange=onchange/>
            </div>
        }
    }

    fn set_height(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                Msg::SetHeight(v.parse().unwrap())
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_height">
                    <abbr title="Height of the pattern.">
                        { "Height" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_height"
                    type="number"
                    value=self.config.height
                    min="1"
                    onchange=onchange/>
            </div>
        }
    }

    fn set_period(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                Msg::SetPeriod(v.parse().unwrap())
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_period">
                    <abbr title="Period of the pattern.">
                        { "Period" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_period"
                    type="number"
                    value=self.config.period
                    min="1"
                    onchange=onchange/>
            </div>
        }
    }

    fn set_dx(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                Msg::SetDx(v.parse().unwrap())
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_dx">
                    <abbr title="Horizontal translation.">
                        { "dx" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_dx"
                    type="number"
                    value=self.config.dx
                    onchange=onchange/>
            </div>
        }
    }

    fn set_dy(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                Msg::SetDy(v.parse().unwrap())
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_dy">
                    <abbr title="Vertical translation.">
                        { "dy" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_dy"
                    type="number"
                    value=self.config.dy
                    onchange=onchange/>
            </div>
        }
    }

    fn set_diag(&self) -> Html {
        let value = self.config.diagonal_width.unwrap_or(0);
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                let diagonal_width = v.parse().unwrap();
                let diagonal_width = match diagonal_width {
                    0 => None,
                    i => Some(i),
                };
                Msg::SetDiag(diagonal_width)
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_diag">
                    <abbr title="If the diagonal width is n > 0, the cells at position (x, y)\
                        where abs(x - y) >= n are assumed to be dead.\n\
                        If this value is set to 0, it would be ignored.">
                        { "Diagonal width" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_diag"
                    type="number"
                    value=value
                    min="0"
                    max=self.config.width.max(self.config.height)
                    disabled=self.config.require_no_diagonal_width()
                    onchange=onchange/>
            </div>
        }
    }

    fn set_max(&self) -> Html {
        let value = self.config.max_cell_count.unwrap_or(0);
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Value(v) = e {
                let max_cell_count = v.parse().unwrap();
                let max_cell_count = match max_cell_count {
                    0 => None,
                    i => Some(i),
                };
                Msg::SetMax(max_cell_count)
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-textfield">
                <label for="set_max">
                    <abbr title="Upper bound of numbers of minimum living cells in all generations.\n\
                        If this value is set to 0, it means there is no limitation.">
                        { "Max cell count" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_max"
                    type="number"
                    value=value
                    min="0"
                    onchange=onchange/>
            </div>
        }
    }

    fn set_trans(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(s) = e {
                match s.value().as_ref() {
                    "Id" => Msg::SetTrans(Transform::Id),
                    "Rotate 90°" => Msg::SetTrans(Transform::Rotate90),
                    "Rotate 180°" => Msg::SetTrans(Transform::Rotate180),
                    "Rotate 270°" => Msg::SetTrans(Transform::Rotate270),
                    "Flip -" => Msg::SetTrans(Transform::FlipRow),
                    "Flip |" => Msg::SetTrans(Transform::FlipCol),
                    "Flip \\" => Msg::SetTrans(Transform::FlipDiag),
                    "Flip /" => Msg::SetTrans(Transform::FlipAntidiag),
                    _ => Msg::None,
                }
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-select">
                <label for="set_trans">
                    <abbr title="Transformations after the last generation in a period.\n\
                        After the last generation in a period, the pattern will return to \
                        the first generation, applying this transformation first, \
                        and then the translation defined by dx and dy.">
                        { "Transformation" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_trans" onchange=onchange>
                    <option selected=self.config.transform == Transform::Id>
                        { "Id" }
                    </option>
                    <option selected=self.config.transform == Transform::Rotate90
                        disabled=self.config.width != self.config.height || self.config.diagonal_width.is_some()>
                        { "Rotate 90°" }
                    </option>
                    <option selected=self.config.transform == Transform::Rotate180>
                        { "Rotate 180°" }
                    </option>
                    <option selected=self.config.transform == Transform::Rotate270
                        disabled=self.config.width != self.config.height || self.config.diagonal_width.is_some()>
                        { "Rotate 270°" }
                    </option>
                    <option selected=self.config.transform == Transform::FlipCol
                        disabled=self.config.diagonal_width.is_some()>
                        { "Flip |" }
                    </option>
                    <option selected=self.config.transform == Transform::FlipRow
                        disabled=self.config.diagonal_width.is_some()>
                        { "Flip -" }
                    </option>
                    <option selected=self.config.transform == Transform::FlipDiag
                        disabled=self.config.width != self.config.height>
                        { "Flip \\" }
                    </option>
                    <option selected=self.config.transform == Transform::FlipAntidiag
                        disabled=self.config.width != self.config.height>
                        { "Flip /" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_sym(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(s) = e {
                match s.value().as_ref() {
                    "C1" => Msg::SetSym(Symmetry::C1),
                    "C2" => Msg::SetSym(Symmetry::C2),
                    "C4" => Msg::SetSym(Symmetry::C4),
                    "D2-" => Msg::SetSym(Symmetry::D2Row),
                    "D2|" => Msg::SetSym(Symmetry::D2Col),
                    "D2\\" => Msg::SetSym(Symmetry::D2Diag),
                    "D2/" => Msg::SetSym(Symmetry::D2Antidiag),
                    "D4+" => Msg::SetSym(Symmetry::D4Ortho),
                    "D4X" => Msg::SetSym(Symmetry::D4Diag),
                    "D8" => Msg::SetSym(Symmetry::D8),
                    _ => Msg::None,
                }
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-select">
                <label for="set_sym">
                    <abbr title="Symmetry of the pattern.">
                        { "Symmetry" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_sym" onchange=onchange>
                    <option selected=self.config.symmetry == Symmetry::C1>
                        { "C1" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::C2>
                        { "C2" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::C4
                        disabled=self.config.width != self.config.height || self.config.diagonal_width.is_some()>
                        { "C4" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D2Col
                        disabled=self.config.diagonal_width.is_some()>
                        { "D2|" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D2Row
                        disabled=self.config.diagonal_width.is_some()>
                        { "D2-" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D2Diag
                        disabled=self.config.width != self.config.height>
                        { "D2\\" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D2Antidiag
                        disabled=self.config.width != self.config.height>
                        { "D2/" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D4Ortho
                        disabled=self.config.diagonal_width.is_some()>
                        { "D4+" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D4Diag
                        disabled=self.config.width != self.config.height>
                        { "D4X" }
                    </option>
                    <option selected=self.config.symmetry == Symmetry::D8
                        disabled=self.config.width != self.config.height || self.config.diagonal_width.is_some()>
                        { "D8" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_order(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(s) = e {
                match s.value().as_ref() {
                    "Automatic" => Msg::SetOrder(None),
                    "Column" => Msg::SetOrder(Some(SearchOrder::ColumnFirst)),
                    "Row" => Msg::SetOrder(Some(SearchOrder::RowFirst)),
                    "Diagonal" => Msg::SetOrder(Some(SearchOrder::Diagonal)),
                    _ => Msg::None,
                }
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-select">
                <label for="set_order">
                    <abbr title="The order to find a new unknown cell.\n\
                        It will always search all generations of one cell \
                        before going to another cell.">
                        { "Search order" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_order" onchange=onchange>
                    <option selected=self.config.search_order.is_none()>
                        { "Automatic" }
                    </option>
                    <option value="Column"
                        selected=self.config.search_order == Some(SearchOrder::ColumnFirst)>
                        { "Column first" }
                    </option>
                    <option value="Row"
                        selected=self.config.search_order == Some(SearchOrder::RowFirst)>
                        { "Row first" }
                    </option>
                    <option value="Diagonal"
                        disabled=self.config.width != self.config.height
                        selected=self.config.search_order == Some(SearchOrder::Diagonal)>
                        { "Diagonal" }
                    </option>
                    <option value=""
                        disabled=true
                        selected=matches!(self.config.search_order, Some(SearchOrder::FromVec(_)))>
                        { "Custom Order" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_choose(&self) -> Html {
        let onchange = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(s) = e {
                match s.value().as_ref() {
                    "Dead" => Msg::SetChoose(NewState::ChooseDead),
                    "Alive" => Msg::SetChoose(NewState::ChooseAlive),
                    "Random" => Msg::SetChoose(NewState::Random),
                    _ => Msg::None,
                }
            } else {
                Msg::None
            }
        });
        html! {
            <div class="mui-select">
                <label for="set_choose">
                    <abbr title="How to choose a state for unknown cells.">
                        { "Choice of state for unknown cells" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_choose" onchange=onchange>
                    <option selected=self.config.new_state == NewState::ChooseAlive>
                        { "Alive" }
                    </option>
                    <option selected=self.config.new_state == NewState::ChooseDead>
                        { "Dead" }
                    </option>
                    <option selected=self.config.new_state == NewState::Random>
                        { "Random" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_reduce(&self) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_reduce"
                        type="checkbox"
                        checked=self.config.reduce_max
                        onclick=self.link.callback(|_| Msg::SetReduce)/>
                    <abbr title="The new max cell count will be set to the cell count of \
                        the current result minus one.">
                        { "Reduce the max cell count when a result is found" }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_skip_subperiod(&self) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_skip_subperiod"
                        type="checkbox"
                        checked=self.config.skip_subperiod
                        onclick=self.link.callback(|_| Msg::SetSkipSubperiod)/>
                    <abbr title="Skip patterns whose fundamental period are smaller than \
                        the given period.">
                        { "Skip patterns with subperiod." }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_skip_subsym(&self) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_skip_subsym"
                        type="checkbox"
                        checked=self.config.skip_subsymmetry
                        onclick=self.link.callback(|_| Msg::SetSkipSubsym)/>
                    <abbr title="Skip patterns which are invariant under more transformations than \
                        required by the given symmetry.">
                        { "Skip patterns invariant under more transformations than the given symmetry." }
                    </abbr>
                </label>
            </div>
        }
    }
}
