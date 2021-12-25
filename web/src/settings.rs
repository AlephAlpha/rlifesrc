use gloo::dialogs;
use log::warn;
use rlifesrc_lib::{
    rules::NtLifeGen, Config, KnownCell, NewState, SearchOrder, Symmetry, Transform,
};
use std::matches;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement};
use yew::{html, Callback, Component, Context, Html, Properties};

pub struct Settings {
    config: Config,
    rule_is_valid: bool,
    known_cells_string: Option<String>,
}

#[derive(Clone, PartialEq, Properties)]
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
    SetMax(Option<u32>),
    SetDiag(Option<i32>),
    SetKnown(String),
    SetReduce,
    SetSkipSubperiod,
    SetSkipSubsym,
    SetBackjump,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let rule_is_valid = ctx.props().config.rule_string.parse::<NtLifeGen>().is_ok();
        Self {
            config: ctx.props().config.clone(),
            rule_is_valid,
            known_cells_string: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
            Msg::SetKnown(known_cells_string) => {
                if known_cells_string.is_empty() {
                    self.config.known_cells = Vec::new();
                    self.known_cells_string = None;
                } else {
                    let known_cells = serde_json::from_str(&known_cells_string);
                    match known_cells {
                        Ok(known_cells) => {
                            self.config.known_cells = known_cells;
                            self.known_cells_string = None;
                        }
                        Err(json_err) => match KnownCell::from_rles(known_cells_string.as_str()) {
                            Ok(known_cells) => {
                                self.config.known_cells = known_cells;
                                self.known_cells_string = None;
                            }
                            Err(rle_err) => {
                                warn!("Invalid JSON format: {}", json_err);
                                warn!("Invalid RLE format: {}", rle_err);
                                self.known_cells_string = Some(known_cells_string);
                            }
                        },
                    }
                }
            }
            Msg::SetReduce => self.config.reduce_max ^= true,
            Msg::SetSkipSubperiod => self.config.skip_subperiod ^= true,
            Msg::SetSkipSubsym => self.config.skip_subsymmetry ^= true,
            Msg::SetBackjump => self.config.backjump ^= true,
            Msg::Apply => {
                if self.known_cells_string.is_some() {
                    dialogs::alert("Invalid format for known cells.");
                } else {
                    ctx.props().callback.emit(self.config.clone());
                }
                return false;
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.config != ctx.props().config && {
            self.config = ctx.props().config.clone();
            self.rule_is_valid = self.config.rule_string.parse::<NtLifeGen>().is_ok();
            self.known_cells_string = None;
            true
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mui-form">
                { self.apply_button(ctx) }
                { self.settings(ctx) }
            </div>
        }
    }
}

impl Settings {
    fn apply_button(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="buttons">
                <button class="mui-btn mui-btn--raised"
                    type="submit"
                    onclick={ctx.link().callback(|_| Msg::Apply)} >
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

    fn settings(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="settings">
                { self.set_rule(ctx) }
                { self.set_width(ctx) }
                { self.set_height(ctx) }
                { self.set_period(ctx) }
                { self.set_dx(ctx) }
                { self.set_dy(ctx) }
                { self.set_diag(ctx) }
                { self.set_trans(ctx) }
                { self.set_sym(ctx) }
                { self.set_max(ctx) }
                { self.set_order(ctx) }
                { self.set_choose(ctx) }
                { self.set_known(ctx) }
                { self.set_reduce(ctx) }
                { self.set_skip_subperiod(ctx) }
                { self.set_skip_subsym(ctx) }
                { self.set_backjump(ctx) }
            </div>
        }
    }

    fn set_rule(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            Some(Msg::SetRule(input.value()))
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
                    class={(!self.rule_is_valid).then(|| "mui--is-invalid")}
                    value={self.config.rule_string.clone()}
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_width(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            input.value().parse().ok().map(Msg::SetWidth)
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
                    value={self.config.width.to_string()}
                    min="1"
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_height(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            input.value().parse().ok().map(Msg::SetHeight)
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
                    value={self.config.height.to_string()}
                    min="1"
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_period(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            input.value().parse().ok().map(Msg::SetPeriod)
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
                    value={self.config.period.to_string()}
                    min="1"
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_dx(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            input.value().parse().ok().map(Msg::SetDx)
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
                    value={self.config.dx.to_string()}
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_dy(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            input.value().parse().ok().map(Msg::SetDy)
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
                    value={self.config.dy.to_string()}
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_diag(&self, ctx: &Context<Self>) -> Html {
        let value = self.config.diagonal_width.unwrap_or(0);
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            let diagonal_width = match input.value().parse().ok()? {
                0 => None,
                i => Some(i),
            };
            Some(Msg::SetDiag(diagonal_width))
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
                    value={value.to_string()}
                    min="0"
                    max={self.config.width.max(self.config.height).to_string()}
                    disabled={self.config.require_no_diagonal_width()}
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_max(&self, ctx: &Context<Self>) -> Html {
        let value = self.config.max_cell_count.unwrap_or(0);
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            let max_cell_count = match input.value().parse().ok()? {
                0 => None,
                i => Some(i),
            };
            Some(Msg::SetMax(max_cell_count))
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
                    value={value.to_string()}
                    min="0"
                    onchange={onchange}/>
            </div>
        }
    }

    fn set_trans(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let select = e.target()?.dyn_into::<HtmlSelectElement>().ok()?;
            match select.value().as_ref() {
                "Id" => Some(Msg::SetTrans(Transform::Id)),
                "Rotate 90°" => Some(Msg::SetTrans(Transform::Rotate90)),
                "Rotate 180°" => Some(Msg::SetTrans(Transform::Rotate180)),
                "Rotate 270°" => Some(Msg::SetTrans(Transform::Rotate270)),
                "Flip -" => Some(Msg::SetTrans(Transform::FlipRow)),
                "Flip |" => Some(Msg::SetTrans(Transform::FlipCol)),
                "Flip \\" => Some(Msg::SetTrans(Transform::FlipDiag)),
                "Flip /" => Some(Msg::SetTrans(Transform::FlipAntidiag)),
                _ => None,
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
                <select id="set_trans" onchange={onchange}>
                    <option selected={self.config.transform == Transform::Id}>
                        { "Id" }
                    </option>
                    <option selected={self.config.transform == Transform::Rotate90}
                        disabled={self.config.width != self.config.height || self.config.diagonal_width.is_some()}>
                        { "Rotate 90°" }
                    </option>
                    <option selected={self.config.transform == Transform::Rotate180}>
                        { "Rotate 180°" }
                    </option>
                    <option selected={self.config.transform == Transform::Rotate270}
                        disabled={self.config.width != self.config.height || self.config.diagonal_width.is_some()}>
                        { "Rotate 270°" }
                    </option>
                    <option selected={self.config.transform == Transform::FlipCol}
                        disabled={self.config.diagonal_width.is_some()}>
                        { "Flip |" }
                    </option>
                    <option selected={self.config.transform == Transform::FlipRow}
                        disabled={self.config.diagonal_width.is_some()}>
                        { "Flip -" }
                    </option>
                    <option selected={self.config.transform == Transform::FlipDiag}
                        disabled={self.config.width != self.config.height}>
                        { "Flip \\" }
                    </option>
                    <option selected={self.config.transform == Transform::FlipAntidiag}
                        disabled={self.config.width != self.config.height}>
                        { "Flip /" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_sym(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let select = e.target()?.dyn_into::<HtmlSelectElement>().ok()?;
            match select.value().as_ref() {
                "C1" => Some(Msg::SetSym(Symmetry::C1)),
                "C2" => Some(Msg::SetSym(Symmetry::C2)),
                "C4" => Some(Msg::SetSym(Symmetry::C4)),
                "D2-" => Some(Msg::SetSym(Symmetry::D2Row)),
                "D2|" => Some(Msg::SetSym(Symmetry::D2Col)),
                "D2\\" => Some(Msg::SetSym(Symmetry::D2Diag)),
                "D2/" => Some(Msg::SetSym(Symmetry::D2Antidiag)),
                "D4+" => Some(Msg::SetSym(Symmetry::D4Ortho)),
                "D4X" => Some(Msg::SetSym(Symmetry::D4Diag)),
                "D8" => Some(Msg::SetSym(Symmetry::D8)),
                _ => None,
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
                <select id="set_sym" onchange={onchange}>
                    <option selected={self.config.symmetry == Symmetry::C1}>
                        { "C1" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::C2}>
                        { "C2" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::C4}
                        disabled={self.config.width != self.config.height || self.config.diagonal_width.is_some()}>
                        { "C4" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D2Col}
                        disabled={self.config.diagonal_width.is_some()}>
                        { "D2|" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D2Row}
                        disabled={self.config.diagonal_width.is_some()}>
                        { "D2-" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D2Diag}
                        disabled={self.config.width != self.config.height}>
                        { "D2\\" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D2Antidiag}
                        disabled={self.config.width != self.config.height}>
                        { "D2/" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D4Ortho}
                        disabled={self.config.diagonal_width.is_some()}>
                        { "D4+" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D4Diag}
                        disabled={self.config.width != self.config.height}>
                        { "D4X" }
                    </option>
                    <option selected={self.config.symmetry == Symmetry::D8}
                        disabled={self.config.width != self.config.height || self.config.diagonal_width.is_some()}>
                        { "D8" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_order(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let select = e.target()?.dyn_into::<HtmlSelectElement>().ok()?;
            match select.value().as_ref() {
                "Automatic" => Some(Msg::SetOrder(None)),
                "Column" => Some(Msg::SetOrder(Some(SearchOrder::ColumnFirst))),
                "Row" => Some(Msg::SetOrder(Some(SearchOrder::RowFirst))),
                "Diagonal" => Some(Msg::SetOrder(Some(SearchOrder::Diagonal))),
                _ => None,
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
                <select id="set_order" onchange={onchange}>
                    <option selected={self.config.search_order.is_none()}>
                        { "Automatic" }
                    </option>
                    <option value="Column"
                        selected={self.config.search_order == Some(SearchOrder::ColumnFirst)}>
                        { "Column first" }
                    </option>
                    <option value="Row"
                        selected={self.config.search_order == Some(SearchOrder::RowFirst)}>
                        { "Row first" }
                    </option>
                    <option value="Diagonal"
                        disabled={self.config.width != self.config.height}
                        selected={self.config.search_order == Some(SearchOrder::Diagonal)}>
                        { "Diagonal" }
                    </option>
                    <option value=""
                        disabled=true
                        selected={matches!(self.config.search_order, Some(SearchOrder::FromVec(_)))}>
                        { "Custom Order" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_choose(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().batch_callback(|e: Event| {
            let select = e.target()?.dyn_into::<HtmlSelectElement>().ok()?;
            match select.value().as_ref() {
                "Dead" => Some(Msg::SetChoose(NewState::ChooseDead)),
                "Alive" => Some(Msg::SetChoose(NewState::ChooseAlive)),
                "Random" => Some(Msg::SetChoose(NewState::Random)),
                _ => None,
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
                <select id="set_choose" onchange={onchange}>
                    <option selected={self.config.new_state == NewState::ChooseAlive}>
                        { "Alive" }
                    </option>
                    <option selected={self.config.new_state == NewState::ChooseDead}>
                        { "Dead" }
                    </option>
                    <option selected={self.config.new_state == NewState::Random}>
                        { "Random" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_known(&self, ctx: &Context<Self>) -> Html {
        let value = if let Some(known_cells_string) = &self.known_cells_string {
            known_cells_string.clone()
        } else if self.config.known_cells.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&self.config.known_cells).unwrap()
        };
        let onchange = ctx.link().batch_callback(|e: Event| {
            let input = e.target()?.dyn_into::<HtmlInputElement>().ok()?;
            Some(Msg::SetKnown(input.value()))
        });
        html! {
            <div class="mui-textfield">
                <label for="set_known">
                    <abbr title="Cells whose states are known before the search. \
                                 Please see the \"Help\" tab for input formats.">
                        { "Known cells." }
                    </abbr>
                    { ":" }
                </label>
                <textarea id="set_known"
                    class={self.known_cells_string.is_some().then(|| "mui--is-invalid")}
                    placeholder="Input in JSON, e.g. [{\"coord\":[0,0,0],\"state\":0},{\"coord\":[1,1,0],\"state\":1}]\n\
                                 Or in RLE, e.g. ?o$2bo$2?o!"
                    value={value}
                    onchange={onchange} />
            </div>
        }
    }

    fn set_reduce(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_reduce"
                        type="checkbox"
                        checked={self.config.reduce_max}
                        onclick={ctx.link().callback(|_| Msg::SetReduce)}/>
                    <abbr title="The new max cell count will be set to the cell count of \
                        the current result minus one.">
                        { "Reduce the max cell count when a result is found" }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_skip_subperiod(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_skip_subperiod"
                        type="checkbox"
                        checked={self.config.skip_subperiod}
                        onclick={ctx.link().callback(|_| Msg::SetSkipSubperiod)}/>
                    <abbr title="Skip patterns whose fundamental period are smaller than \
                        the given period.">
                        { "Skip patterns with subperiod." }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_skip_subsym(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_skip_subsym"
                        type="checkbox"
                        checked={self.config.skip_subsymmetry}
                        onclick={ctx.link().callback(|_| Msg::SetSkipSubsym)}/>
                    <abbr title="Skip patterns which are invariant under more transformations than \
                        required by the given symmetry.">
                        { "Skip patterns invariant under more transformations than the given symmetry." }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_backjump(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_backjump"
                        type="checkbox"
                        checked={self.config.backjump}
                        onclick={ctx.link().callback(|_| Msg::SetBackjump)}/>
                    <abbr title="The current implementation of backjumping is very slow, \
                        only useful for large (e.g., 64x64) still lifes.">
                        { "(Experimental) Enable backjumping." }
                    </abbr>
                </label>
            </div>
        }
    }
}
