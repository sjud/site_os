#![feature(lazy_cell)]
use serde::{Serialize,Deserialize};

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
    provide_context::<RwSignal<SystemRuntime>>(create_rw_signal(SystemRuntime::new(
        {
            let mut file_system = FileSystem::new();
            file_system.add_file(
                Uuid::new_v4(),
                "/".to_string(),
                Metadata{
                file_type:FileType::Directory,
                img_src:"/hard-disk.png".to_string(),
                task_bar:None,
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/bin".to_string(),
                Metadata{
                file_type:FileType::Directory,
                img_src:"/folder.png".to_string(),
                task_bar:None,
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
            "/bin/folder.png".to_string(),
            Metadata{
                file_type:FileType::File,
                img_src:"/folder.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:0,
                }),
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/bin/browser".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/browser.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:1,
                }),
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/bin/calendar".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/calendar.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:2,
                }),
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/calculator".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/calculator.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:3,
                }),
                ..Default::default()
            });
             file_system.add_file(
                Uuid::new_v4(),
                "/text".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/text.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:4,
                }),
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/csv".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/csv-file.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:5,
                }),
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/picture".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/picture.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:6,
                }),
                ..Default::default()
            });
            file_system.add_file(
                Uuid::new_v4(),
                "/terminal".to_string(),
                Metadata{
                file_type:FileType::File,
                img_src:"/terminal.png".to_string(),
                task_bar:Some(TaskBarData{
                    is_jumping:false,
                    idx:7,
                }),
                ..Default::default()
            });
            file_system
        },
    )));
    view!{
        <topbar::TopBar/>
        <desktop::Desktop/>
        <taskbar::TaskBar/>
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