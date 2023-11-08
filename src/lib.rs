#![feature(lazy_cell)]
use serde::{Serialize,Deserialize};

use cfg_if::cfg_if;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
pub mod fallback;
pub mod error_template;
pub mod client_state;
#[cfg(feature="ssr")]
pub mod backend_utils;
#[cfg(feature="ssr")]
pub mod server_state;
pub mod user_msg;
pub mod topbar;
pub mod bottombar;
pub mod desktop;
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
 
    view! {
        <Stylesheet id="leptos" href="/pkg/site_os.css"/>
        <Stylesheet id="googlefont" href="https://fonts.googleapis.com/css?family=Lunasima"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Meta name="description" content="SiteOs is a site that looks like an os."/>
        <Title text="site_os"/>
        //<DisableRightClick/>
        <Router>
            <main>
            <Body attr:style="
            font-family: 'Lunasima';
            color:white;
            background-image: url(bg.png);
            background-repeat: repeat;"/>
                <Routes>
                    <Route path="" view=|| view!{
                        <topbar::TopBar/>
                        <desktop::Desktop/>
                        <bottombar::BottomBar/>
                    }/>
                </Routes>
            </main>
        </Router>
    }
}

#[island]
pub fn DisableRightClick() -> impl IntoView {
    let _ = window_event_listener(ev::contextmenu, |ev| {
        ev.prevent_default();
    });
}




// Needs to be in lib.rs AFAIK because wasm-bindgen needs us to be compiling a lib. I may be wrong.
cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        pub fn hydrate() {
            #[cfg(debug_assertions)]
            console_error_panic_hook::set_once();
            leptos::leptos_dom::HydrationCtx::stop_hydrating();
        }
    }
}