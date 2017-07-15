#![feature(fn_traits, unboxed_closures)]

extern crate dht22_pi;
extern crate hyper;
extern crate prometheus;

pub mod closures;
pub mod core;
pub mod poller;
pub mod sensors;
pub mod web;
