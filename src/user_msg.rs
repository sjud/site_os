use super::*;

#[component]
pub fn UserMsg() -> impl IntoView {
    let (msg,msg_set) = expect_context::<GlobalState>().user_msg.split();
    let clear_msg = move |_| msg_set.set(user_msg::UserMsg::default());
    view! {
        { move || {
            let header = msg().header.clone();
            let body = msg().body.clone();
            let theme = msg().theme.clone();
            view!{
                <div 
                class="fixed bottom-1/2 left-1/2 rounded translate-x-[-50%] \
                 border-t border-b   px-4 py-3 z-50" 
                class=("bg-red-100", move || theme == MsgTheme::Red)
                class=("border-red-500", move || theme == MsgTheme::Red)
                class=("text-red-700", move || theme == MsgTheme::Red)
                class=("bg-green-100", move || theme == MsgTheme::Green)
                class=("border-green-500", move || theme == MsgTheme::Green)
                class=("text-green-700", move || theme == MsgTheme::Green)
                class=("hidden", move || theme ==  MsgTheme::Clear)
        
                role="alert">
                <p class="font-bold">{header}</p>
                 <p class="text-sm">{body}</p>
                    <button on:click=clear_msg
                    class=("bg-red-700", move || theme == MsgTheme::Red)
                    class=("bg-green-700", move || theme == MsgTheme::Red)
                    class="p-1  rounded text-black underline">{"Close"}</button>
                    </div>
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct UserMsg {
    pub theme: MsgTheme,
    pub header: String,
    pub body: String,
}

#[derive(PartialEq, Debug,Copy, Eq, Clone, Default)]
pub enum MsgTheme {
    Green,
    Red,
    #[default]
    Clear,
}