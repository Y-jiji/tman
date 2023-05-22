#![feature(drain_filter)]
#![feature(btree_drain_filter)]

mod data;
mod view;
mod util;

fn main() {
    let mut data = crate::data::Data::load();
    let mut view = crate::view::View::load(&mut data);
    view.runapp();
    data.save();
}