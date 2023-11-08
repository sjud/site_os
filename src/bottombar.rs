use super::*;

#[component]
pub fn BottomBar() -> impl IntoView {
    view!{
        <div class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 flex">
        <img class="w-16" src="/folder.png"/>
        <img class="w-16" src="/browser.png"/>
        <img class="w-16" src="/calendar.png"/>
        <img class="w-16" src="/calculator.png"/>
        <img class="w-16" src="/text.png"/>
        <img class="w-16" src="/csv-file.png"/>
        <img class="w-16" src="/picture.png"/>
        <img class="w-16" src="/terminal.png"/>
        </div>
    }
}