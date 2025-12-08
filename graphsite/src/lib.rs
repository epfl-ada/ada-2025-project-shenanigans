#![cfg(target_arch = "wasm32")]
use std::cell::RefCell;

use crate::app::Adapp;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures as _; // ensure the crate is linked for wasm_bindgen async
use web_sys::HtmlCanvasElement;

mod app;

thread_local! {
    static RUNNER: RefCell<Option<eframe::WebRunner>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    // Fire and forget: kick off the async runner
    wasm_bindgen_futures::spawn_local(async {
        let _ = run().await;
    });
    Ok(())
}

#[wasm_bindgen]
pub fn stop() {
    RUNNER.with(|e| {
        if let Some(runner) = e.borrow_mut().take() {
            runner.destroy();
        }
    });
}

#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id("the_canvas_id")
        .ok_or_else(|| {
            JsValue::from_str("canvas with id 'the_canvas_id' not found")
        })?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| {
            JsValue::from_str("failed to cast to HtmlCanvasElement")
        })?;

    let web_options = eframe::WebOptions::default();

    let runner = eframe::WebRunner::new();
    RUNNER.with(|e| e.borrow_mut().replace(runner));
    let runref = RUNNER.with(|e| e.borrow().as_ref().unwrap().clone());
    runref
        .start(
            canvas,
            web_options,
            Box::new(|cc| {
                Ok::<Box<dyn eframe::App>, _>(Box::new(Adapp::new(cc)))
            }),
        )
        .await?;

    Ok(())
}
