use super::*;

#[component]
pub fn Desktop() -> impl IntoView{
    view!{
    <div 
        class="flex flex-wrap h-full w-full pt-8 pl-4 pr-4 pb-16" 
        on:dragover = move |ev| ev.prevent_default()>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>
         <div class="bg-blue-300 w-24 h-24 m-1"></div>


        <DesktopItem src="hard-disk.png"/>
    </div>
      
    }
}

#[component]
pub fn DesktopItem(src:&'static str) -> impl IntoView{
    view!{
        <div class="w-24" id=src >
        <img src=src clas="w-16" />
        </div>
    }
}