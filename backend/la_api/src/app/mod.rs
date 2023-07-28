//! The root module for "/" routes

// export all controllers
pub mod app_controller;

// export services
pub mod app_service;

//TODO: move this its the base route of the controller not the module  

// export the base route for this module
pub const BASE: &str = "/";

// export modules DTOs, structs, traits, etc.

// services that are used in the module but not exported
// mod service

//TODO: define traits for module, controller and service?
//      - to inject context like Base route, db connections etc
//      - to ensure BASE is defined
//      - to ensure that all routes can be exported in some kind of array to bind the array instead
//        of each function/route
//      tl;dr: to mock the structure and behavior of NestJS?