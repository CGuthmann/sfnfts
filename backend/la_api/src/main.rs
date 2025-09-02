use rocket::{launch, routes};

mod app;

#[launch]
fn rocket() -> _ { 
    // mount each module 
    rocket::build()
       .mount(app::BASE, routes![
           app::app_controller::get_root,
           app::app_controller::head_root,
           app::app_controller::get_nullifier,
           app::app_controller::get_trust_list,
           app::app_controller::get_merkle_tree,
           app::app_controller::post_mint,
           app::app_controller::post_transfer,
           app::app_controller::post_burn,
           app::app_controller::post_transfer_child,
           app::app_controller::get_merkle_proof,
           app::app_controller::get_trust_proof,
       ])
    // maybe register catcher later
}
