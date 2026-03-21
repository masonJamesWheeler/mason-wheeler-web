#![allow(unused)]

use leptos::prelude::*;

#[component]
pub fn SignaturePad(
    on_sign: Callback<String>,
    #[prop(default = 400)] width: u32,
    #[prop(default = 150)] height: u32,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let (is_empty, set_is_empty) = signal(true);

    // Drawing state managed via cfg(feature = "hydrate")
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        use web_sys::{
            CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, TouchEvent,
        };

        let drawing = std::rc::Rc::new(std::cell::Cell::new(false));

        let get_ctx = move || -> Option<CanvasRenderingContext2d> {
            let canvas = canvas_ref.get()?;
            let canvas: &HtmlCanvasElement = canvas.as_ref();
            canvas
                .get_context("2d")
                .ok()?
                .and_then(|obj| obj.dyn_into::<CanvasRenderingContext2d>().ok())
        };

        let clear_canvas = move || {
            if let Some(ctx) = get_ctx() {
                let canvas = canvas_ref.get().unwrap();
                let canvas: &HtmlCanvasElement = canvas.as_ref();
                ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
                // Draw placeholder text
                ctx.set_font("14px sans-serif");
                ctx.set_fill_style_str("#9ca3af");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                let _ = ctx.fill_text(
                    "Sign here",
                    canvas.width() as f64 / 2.0,
                    canvas.height() as f64 / 2.0,
                );
                set_is_empty.set(true);
            }
        };

        // Initialize placeholder text after mount
        Effect::new(move || {
            canvas_ref.get();
            clear_canvas();
        });

        let on_pointer_down = move |x: f64, y: f64| {
            drawing.set(true);
            if let Some(ctx) = get_ctx() {
                // Clear placeholder on first stroke
                if is_empty.get_untracked() {
                    let canvas = canvas_ref.get().unwrap();
                    let canvas: &HtmlCanvasElement = canvas.as_ref();
                    ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
                    set_is_empty.set(false);
                }
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.set_stroke_style_str("#1c1917");
                ctx.set_line_width(2.0);
                ctx.set_line_cap("round");
                ctx.set_line_join("round");
            }
        };

        let on_pointer_move = move |x: f64, y: f64| {
            if drawing.get() {
                if let Some(ctx) = get_ctx() {
                    ctx.line_to(x, y);
                    ctx.stroke();
                }
            }
        };

        let on_pointer_up = move || {
            drawing.set(false);
        };

        let mouse_down = move |ev: MouseEvent| {
            let canvas = canvas_ref.get().unwrap();
            let canvas: &HtmlCanvasElement = canvas.as_ref();
            let rect = canvas.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            on_pointer_down(x, y);
        };

        let mouse_move = move |ev: MouseEvent| {
            let canvas = canvas_ref.get().unwrap();
            let canvas: &HtmlCanvasElement = canvas.as_ref();
            let rect = canvas.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            on_pointer_move(x, y);
        };

        let mouse_up = move |_ev: MouseEvent| {
            on_pointer_up();
        };

        let touch_start = move |ev: TouchEvent| {
            ev.prevent_default();
            if let Some(touch) = ev.touches().get(0) {
                let canvas = canvas_ref.get().unwrap();
                let canvas: &HtmlCanvasElement = canvas.as_ref();
                let rect = canvas.get_bounding_client_rect();
                let x = touch.client_x() as f64 - rect.left();
                let y = touch.client_y() as f64 - rect.top();
                on_pointer_down(x, y);
            }
        };

        let touch_move = move |ev: TouchEvent| {
            ev.prevent_default();
            if let Some(touch) = ev.touches().get(0) {
                let canvas = canvas_ref.get().unwrap();
                let canvas: &HtmlCanvasElement = canvas.as_ref();
                let rect = canvas.get_bounding_client_rect();
                let x = touch.client_x() as f64 - rect.left();
                let y = touch.client_y() as f64 - rect.top();
                on_pointer_move(x, y);
            }
        };

        let touch_end = move |ev: TouchEvent| {
            ev.prevent_default();
            on_pointer_up();
        };

        let on_clear = move |_| {
            clear_canvas();
        };

        let on_sign_click = move |_| {
            if is_empty.get_untracked() {
                return;
            }
            if let Some(canvas_el) = canvas_ref.get() {
                let canvas: &HtmlCanvasElement = canvas_el.as_ref();
                if let Ok(data_url) = canvas.to_data_url_with_type("image/png") {
                    on_sign.run(data_url);
                }
            }
        };

        view! {
            <div class="flex flex-col gap-2">
                <canvas
                    node_ref=canvas_ref
                    width=width
                    height=height
                    class="border-2 border-dashed border-stone-300 rounded-lg cursor-crosshair bg-white touch-none"
                    on:mousedown=mouse_down
                    on:mousemove=mouse_move
                    on:mouseup=mouse_up
                    on:mouseleave=mouse_up
                    on:touchstart=touch_start
                    on:touchmove=touch_move
                    on:touchend=touch_end
                />
                <div class="flex gap-2">
                    <button
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-stone-600 bg-stone-100 hover:bg-stone-200 rounded-lg transition-colors"
                        on:click=on_clear
                    >
                        "Clear"
                    </button>
                    <button
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-white bg-stone-800 hover:bg-stone-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || is_empty.get()
                        on:click=on_sign_click
                    >
                        "Sign"
                    </button>
                </div>
            </div>
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        view! {
            <div class="flex flex-col gap-2">
                <canvas
                    node_ref=canvas_ref
                    width=width
                    height=height
                    class="border-2 border-dashed border-stone-300 rounded-lg cursor-crosshair bg-white touch-none"
                />
                <div class="flex gap-2">
                    <button
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-stone-600 bg-stone-100 hover:bg-stone-200 rounded-lg transition-colors"
                    >
                        "Clear"
                    </button>
                    <button
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-white bg-stone-800 hover:bg-stone-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=true
                    >
                        "Sign"
                    </button>
                </div>
            </div>
        }
    }
}
