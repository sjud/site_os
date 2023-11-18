#![feature(lazy_cell)]
use serde::{Serialize,Deserialize};
use std::str::FromStr;
use web_sys::wasm_bindgen::JsCast;
use uuid::Uuid;
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
pub mod system_runtime;
pub mod active_procceses;
pub mod application;
pub mod folder;
pub mod dock;
use system_runtime::SystemRuntime;
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
                    <Route path="" view=|| view!{<OperatingSystem/>}/>
                </Routes>
            </main>
        </Router>
    }
}


#[component]
pub fn OperatingSystem() -> impl IntoView {
    use system_runtime::*;
    let finder_id = Uuid::new_v4();
    let firefox_id = Uuid::new_v4();
    let calendar_id = Uuid::new_v4();
    let calculator_id = Uuid::new_v4();
    let text_id = Uuid::new_v4();
    let csv_id = Uuid::new_v4();
    let picture_id = Uuid::new_v4();
    let terminal_id = Uuid::new_v4();
    let read_id = Uuid::new_v4();

    let dock_list = vec![
        finder_id,
        firefox_id,
        calendar_id,
        calculator_id,
        text_id,
        csv_id,
        picture_id,
        terminal_id,
        read_id,
    ];
    let mut file_system = FileSystem::new();
          
    file_system.add_file(
        Uuid::from_u128(1),
        "/".to_string(),
        Metadata{
        file_type:FileType::Directory,
        img_src:"/hard-disk.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        Uuid::new_v4(),
        "/bin".to_string(),
        Metadata{
        file_type:FileType::Directory,
        img_src:"/folder2.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        finder_id,
    "/bin/finder".to_string(),
    Metadata{
        file_type:FileType::File,
        img_src:"/finder.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        firefox_id,
        "/bin/firefox".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/firefox.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        calendar_id,
        "/bin/calendar".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/calendar.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        calculator_id,
        "/calculator".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/calculator.png".to_string(),
        ..Default::default()
    });
     file_system.add_file(
        text_id,
        "/text".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/text.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        csv_id,
        "/csv".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/csv-file.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        picture_id,
        "/picture".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/picture.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        terminal_id,
        "/terminal".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/terminal.png".to_string(),
        ..Default::default()
    });
    file_system.add_file(
        read_id,
        "/reader".to_string(),
        Metadata{
        file_type:FileType::File,
        img_src:"/reader.png".to_string(),
        ..Default::default()
    });
    provide_context::<RwSignal<SystemRuntime>>(create_rw_signal(SystemRuntime::new(file_system,dock_list)));
    view!{
        <topbar::TopBar/>
        <desktop::Desktop/>
        <active_procceses::ActiveProcesses/>
        <dock::Dock/>
    }
}




#[component]
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
            leptos::mount_to_body(App);
        }
    }
}