mod rules;
mod search;
mod web;
mod world;


use web::Model;

fn main() {
    yew::start_app::<Model>();
}
