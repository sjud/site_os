use super::*;

#[component]
pub fn Desktop() -> impl IntoView{
    view!{
    <div 
        class="grid gap-10 p-16" 
        style="repeat(auto-fill, minmax(6rem, 1fr));"
        on:dragover = move |ev| ev.prevent_default()
         >
        <DesktopItem src="hard-disk.png"/>
    </div>
      
    }
}

#[component]
pub fn DesktopItem(src:&'static str) -> impl IntoView{
    view!{
        <div class="w-24" id=src
        on:dragstart = move |ev| {ev.data_transfer().unwrap().set_data("text/plain",event_target::<web_sys::HtmlElement>(&ev).id().as_str());}
        >
        <img src=src clas="w-16" />
        </div>
    }
}