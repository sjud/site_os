use super::*;

pub const DROP_DOWN_LIST_ITEM_ID: &'static str = "drop_down_list_item";
#[component]
pub fn TopBar() -> impl IntoView {
    view!{
        <div class="w-full h-6 bg-slate-500 bg-opacity-20 backdrop-blur-md flex justify-start fixed top-0">
            <TopBarProvider>
            <div class="flex">
            <DropDownButton show=DropDownShow::Logo>
                <TopLeftEye/>    
            </DropDownButton>
            <DropDownButton show=DropDownShow::File>
            "File"
            </DropDownButton>
            <DropDownButton show=DropDownShow::Edit>
            "Edit"
            </DropDownButton>
            <DropDownButton show=DropDownShow::View>
            "View"
            </DropDownButton>
            <DropDownButton show=DropDownShow::Help>
            "Help"
            </DropDownButton>
            </div>
            <DropDown/>
            </TopBarProvider>
        </div>
    }
}

#[island]
fn DropDownButton(children:Children,show:DropDownShow) -> impl IntoView{
    let set_show = expect_context::<RwSignal<DropDownShow>>().write_only();
    let read_show = expect_context::<RwSignal<DropDownShow>>().read_only();
    let set_xy = expect_context::<RwSignal<DropDownXY>>().write_only();
    let btn_ref = create_node_ref::<leptos::html::Button>();
    create_effect(move |_|{
        if read_show() == show {
            let elem = btn_ref.get().unwrap();
            let y = elem.offset_top() + elem.offset_height();
            let x = elem.offset_left();
            set_xy(DropDownXY((x,y)));           
        }
    });
    view!{
        <button
        _ref=btn_ref
         id="topbar_btn"
        class="pl-3 pr-3 rounded-[0.25rem] "
        on:click=move |ev| {
            if read_show.get_untracked() == show {
                set_show(DropDownShow::None)
            } else {
                set_show(show);
            }
            ev.stop_immediate_propagation();
        }
        on:mouseover=move |ev| {
            let current = read_show.get_untracked();
            if  current != show && current != DropDownShow::None {
                set_show(show)
            }
            ev.stop_immediate_propagation();
        }
        class=("bg-slate-400", move || read_show() == show)
        >
        {children()}
        </button>
    }
}

#[derive(PartialEq,Clone,Debug)]
pub struct HoverHighlight(pub bool);
#[island]
pub fn DropDown() -> impl IntoView {
    let div_ref = create_node_ref::<leptos::html::Div>();
    let show = expect_context::<RwSignal<DropDownShow>>().read_only();
    let set_show = expect_context::<RwSignal<DropDownShow>>().write_only();
    let read_dropdown_xy = expect_context::<RwSignal<DropDownXY>>().read_only();
    let drop_down_xy_style = move || {
        let DropDownXY((x,y)) = read_dropdown_xy();
        format!("position:absolute;top:{}px;left:{}px;",y+2,x)
    };
    provide_context::<RwSignal<HoverHighlight>>(create_rw_signal(HoverHighlight(true)));
    let set_hover_highlight = expect_context::<RwSignal<HoverHighlight>>().write_only();
    let handle = window_event_listener(ev::click, move |ev| {
        let target = event_target::<web_sys::HtmlElement>(&ev);
        let node = div_ref.get_untracked().expect("div to be set here.");
        if target.id() == DROP_DOWN_LIST_ITEM_ID {
            let timeout = gloo::timers::callback::Timeout::new(20, move || {
                set_hover_highlight(HoverHighlight(false));
                let timeout = gloo::timers::callback::Timeout::new(50, move || {
                    set_hover_highlight(HoverHighlight(true));
                    let timeout = gloo::timers::callback::Timeout::new(150, move || {
                        set_show(DropDownShow::None);
                    });
                    timeout.forget();
                });
                timeout.forget();
            });
            timeout.forget();
        } else if target.id() != (*node).id() {
            set_show(DropDownShow::None);
        }
    });
    on_cleanup(move || handle.remove());
    view!{
        <div class="flex flex-col bg-slate-700 bg-opacity-50 backdrop-blur-md rounded-[0.25rem] pt-1 pb-1" 
        style=drop_down_xy_style
        _ref=div_ref id="dropdown">
            {   
            move || match show() {
                    DropDownShow::None => view!{}.into_view(),
                    DropDownShow::Logo => view!{
                        <DropDownListItem name="About site_os"/>
                        <DropDownListItem name="System Preferences"/>
                       // <DropDownListItem name="App Store"/>
                    }.into_view(),
                    DropDownShow::File => view!{
                        <DropDownListItem name="New Folder"/>
                        <DropDownListItem name="Find"/>
                    }.into_view(),
                    DropDownShow::Edit => view!{
                        <DropDownListItem name="Undo"/>
                        <DropDownListItem name="Redo"/>
                        <DropDownListItem name="Cut"/>
                        <DropDownListItem name="Copy"/>
                        <DropDownListItem name="Paste"/>
                    }.into_view(),
                    DropDownShow::View => view!{
                        <DropDownListItem name="As Icons"/>
                        <DropDownListItem name="As List"/>
                    }.into_view(),
                    DropDownShow::Help => view!{
                        <DropDownListItem name="site_os Help"/>
                    }.into_view(),
                }
            }
        </div>

    }
   
}

#[component]
fn DropDownListItem(name:&'static str) -> impl IntoView{
    let read_hover_highlight = expect_context::<RwSignal<HoverHighlight>>().read_only();
    view!{
        <div class="ml-3 mr-3 rounded-[0.25rem]" id=DROP_DOWN_LIST_ITEM_ID
        class=("hover:bg-slate-400",move || read_hover_highlight().0 )>
        <button class="rounded-md pl-2 pr-2" id=DROP_DOWN_LIST_ITEM_ID>{name}</button>
        </div>
    }
}
#[derive(Clone,Copy,Debug,Serialize,Deserialize,PartialEq)]
enum DropDownShow{
    None,
    Logo,
    File,
    Edit,
    View,
    Help,
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct DropDownXY((i32,i32));
#[island]
fn TopBarProvider(children:Children) -> impl IntoView {
    provide_context::<RwSignal<DropDownShow>>(create_rw_signal(DropDownShow::None));
    provide_context::<RwSignal<DropDownXY>>(create_rw_signal(DropDownXY((0,0))));
    children()
}

#[component]
pub fn TopLeftEye() -> impl IntoView {
    view!{
        <svg class="h-6" id="top_left_eye"
        version="1.0" xmlns="http://www.w3.org/2000/svg"
        width="32.000000pt" height="32.000000pt" viewBox="0 0 32.000000 32.000000"
        preserveAspectRatio="xMidYMid meet">

        <g class="fill-white" transform="translate(0.000000,32.000000) scale(0.100000,-0.100000)"
        fill="#000000" stroke="none">
        <path d="M65 212 c-16 -11 -36 -26 -44 -36 -12 -14 -11 -19 5 -37 30 -33 89
        -59 134 -59 45 0 104 26 134 59 19 21 19 21 0 42 -17 19 -65 49 -79 49 -3 0
        -1 -9 5 -19 16 -31 12 -56 -13 -79 -30 -28 -68 -28 -95 1 -24 26 -27 48 -12
        78 13 24 3 24 -35 1z"/>
        <path d="M134 196 c-10 -26 4 -48 28 -44 17 2 23 10 23 28 0 18 -6 26 -23 28
        -13 2 -25 -3 -28 -12z"/>
        </g>
        </svg>
    }
}

