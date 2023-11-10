#![feature(lazy_cell)]
use serde::{Serialize,Deserialize};

use web_sys::wasm_bindgen::JsCast;

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
pub mod taskbar;
pub mod desktop;
pub mod file_system;
use file_system::SystemRuntime;
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
                    <Route path="" view=|| view!{<Desktop/>}/>
                </Routes>
            </main>
        </Router>
    }
}



#[component]
pub fn Desktop() -> impl IntoView {
    view!{
        <OperatingSystemProvider>
        <topbar::TopBar/>
        <desktop::Desktop/>
        <taskbar::TaskBar/>
        </OperatingSystemProvider>
    }
}



#[island]
pub fn OperatingSystemProvider(children:Children) -> impl IntoView {
    use file_system::*;
    provide_context::<RwSignal<SystemRuntime>>(create_rw_signal(SystemRuntime::new(
        {
            let mut file_system = FileSystem::new();
            file_system.add_file("/finder".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/folder.png".to_string(),
            });
            file_system.add_file("/browser".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/browser.png".to_string(),
            });
            file_system.add_file("/calendar".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/calendar.png".to_string(),
            });
            file_system.add_file("/calculator".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/calculator.png".to_string(),
            });
             file_system.add_file("/text".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/text.png".to_string(),
            });
            file_system.add_file("/csv".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/csv-file.png".to_string(),
            });
            file_system.add_file("/picture".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/picture.png".to_string(),
            });
            file_system.add_file("/terminal".to_string(),Metadata{
                accessed:0,
                created:0,
                modified:0,
                file_type:FileType::File,
                img_src:"/terminal.png".to_string(),
            });
            file_system
        },
        vec![
            "/finder".to_string(),
            "/browser".to_string(),
            "/calendar".to_string(),
            "/calculator".to_string(),
            "/text".to_string(),
            "/csv".to_string(),
            "/picture".to_string(),
            "/terminal".to_string(),
            ]
    )));

    children()
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