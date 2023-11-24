use std::rc::Rc;

use super::*;


use web_sys::MouseEvent;

#[derive(PartialEq,Clone)]
pub struct ProgramTopBarData{
    pub file_id:Uuid,
    pub data: Vec<TopBarData>,
}
impl std::fmt::Debug for ProgramTopBarData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgramTopBarData")
            .field("file_id", &self.file_id)
            .finish()
    }
}

#[derive(Clone)]
pub struct TopBarData{
    pub header:String,
    pub fields:Vec<Rc<dyn TopBarField>>,
}
impl PartialEq for TopBarData {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
    }
}



pub trait TopBarField{
    fn name(&self) -> &'static str;
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool;
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent);
    fn on_highlight(&self,_system:RwSignal<GlobalState>) -> View {view!{}.into_view()}
}

#[derive(Clone,Debug,PartialEq)]
pub struct FinderAbout;
impl TopBarField for FinderAbout{
    fn name(&self,) -> &'static str {"About"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderPreferences;
impl TopBarField for FinderPreferences{
    fn name(&self,) -> &'static str {"Preferences"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}


#[derive(Clone,Debug,PartialEq)]
pub struct LogoAboutSiteOs;
impl TopBarField for LogoAboutSiteOs {
    fn name(&self,) -> &'static str {"About site_os"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct LogoSystemPreferences;
impl TopBarField for LogoSystemPreferences {
    fn name(&self,) -> &'static str {"System Preferences"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderFileNewFolder;
impl TopBarField for FinderFileNewFolder {
    fn name(&self,) -> &'static str {"New Folder"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderFileRename;
impl TopBarField for FinderFileRename {
    fn name(&self,) -> &'static str {"Rename"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}

#[derive(Clone,Debug,PartialEq)]
pub struct FinderEditUndo;
impl TopBarField for FinderEditUndo {
    fn name(&self,) -> &'static str {"Undo"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderEditRedo;
impl TopBarField for FinderEditRedo {
    fn name(&self,) -> &'static str {"Redo"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderEditCut;
impl TopBarField for FinderEditCut {
    fn name(&self,) -> &'static str {"Cut"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderEditCopy;
impl TopBarField for FinderEditCopy {
    fn name(&self,) -> &'static str {"Copy"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderEditPaste;
impl TopBarField for FinderEditPaste {
    fn name(&self,) -> &'static str {"Paste"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderEditShowClipboard;
impl TopBarField for FinderEditShowClipboard {
    fn name(&self,) -> &'static str {"Show Clipboard"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderViewAsColumns;
impl TopBarField for FinderViewAsColumns {
    fn name(&self,) -> &'static str {"As Columns"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderViewAsIcons;
impl TopBarField for FinderViewAsIcons{
    fn name(&self,) -> &'static str {"As Icons"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderViewAsList;
impl TopBarField for FinderViewAsList {
    fn name(&self,) -> &'static str {"As List"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderViewAsGallery;
impl TopBarField for FinderViewAsGallery {
    fn name(&self,) -> &'static str {"As Gallery"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
}
#[derive(Clone,Debug,PartialEq)]
pub struct FinderViewAsSortBy;
impl TopBarField for FinderViewAsSortBy {
    fn name(&self,) -> &'static str {"As SortBy"}
    fn available(&self,_system:ReadSignal<GlobalState>) -> bool {true}
    fn on_click(&self,_system:RwSignal<GlobalState>,_ev:MouseEvent) {}
    fn on_highlight(&self,_system:RwSignal<GlobalState>) -> View{
        view!{
            // Make Drop down more general and add it here.
        }.into_view()
    }
}
impl ProgramTopBarData {
    pub fn finder() -> Self {
        Self { 
            file_id: Uuid::from_u128(0), 
            data: vec![
                TopBarData { 
                    header: "Finder".to_string(), 
                    fields: vec![  // about, preferences, hide finder, hide others
                        Rc::new(FinderAbout{}),
                        Rc::new(FinderPreferences{}),
                    ]
                },
                TopBarData { 
                    header: "File".to_string(), 
                    fields: vec![ 
                        Rc::new(FinderFileNewFolder{}),
                        Rc::new(FinderFileRename{}),
                    ]
                },
                TopBarData { 
                    header: "Edit".to_string(), 
                    fields: vec![ 
                        Rc::new(FinderEditUndo{}),
                        Rc::new(FinderEditRedo{}),
                        Rc::new(FinderEditCut{}),
                        Rc::new(FinderEditCopy{}),
                        Rc::new(FinderEditPaste{}),
                        Rc::new(FinderEditShowClipboard{}),
                    ]
                },
                TopBarData { 
                    header: "View".to_string(), 
                    fields: vec![ 
                        Rc::new(FinderViewAsColumns{}),
                        Rc::new(FinderViewAsIcons{}),
                        Rc::new(FinderViewAsList{}),
                        Rc::new(FinderViewAsGallery{}),
                        Rc::new(FinderViewAsSortBy{}),
                    ]
                },
            ]
        }
    }
}

pub const DROP_DOWN_LIST_ITEM_ID: &'static str = "drop_down_list_item";
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DropDownShow(Option<usize>);
#[component]
pub fn TopBar() -> impl IntoView {
    let system =expect_context::<SystemState>().0;
    let top_bar_data = create_read_slice(system,|state|state.program_top_bar.clone()
        .unwrap_or(
            ProgramTopBarData::finder()
        )
    );
    // if some(idx) shows the drop down menu of the item, where 0 is logo
    let show = create_rw_signal(DropDownShow(None::<usize>));
    provide_context::<RwSignal<DropDownShow>>(show.clone());
    provide_context::<RwSignal<DropDownXY>>(create_rw_signal(DropDownXY((0,0))));
    view!{
        <div class="w-full h-6 bg-slate-500 bg-opacity-20 backdrop-blur-md flex justify-start fixed top-0">
            <div class="flex">
            <DropDownButton show=DropDownShow(Some(0))>
                <TopLeftEye/>    
            </DropDownButton>
            {
                move || {
                    let mut views = Vec::new();
                    for (i,d) in top_bar_data().data.into_iter().enumerate() {
                        views.push(
                            view!{
                                <DropDownButton show=DropDownShow(Some(i+1))>
                                    {d.header}
                                </DropDownButton>
                            }
                        )
                    }
                    views.collect_view()
                }
            }
        </div>
        <div class="ml-auto">
        {
            move || {
                //let time = chrono::DateTime::from_timestamp(timestamp.get_untracked() as i64,0).unwrap();
                //time.to_string()
            }
        }
        </div>
        <DropDown/>
        </div>
    }
}

#[component]
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
                set_show(DropDownShow(None))
            } else {
                set_show(show);
            }
            ev.stop_immediate_propagation();
        }
        on:mouseover=move |ev| {
            let current = read_show.get_untracked();
            if  current != show && current != DropDownShow(None) {
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
#[component]
pub fn DropDown() -> impl IntoView {
    let state = expect_context::<GlobalState>();
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
                        set_show(DropDownShow(None));
                    });
                    timeout.forget();
                });
                timeout.forget();
            });
            timeout.forget();
        } else if target.id() != (*node).id() {
            set_show(DropDownShow(None));
        }
    });
    on_cleanup(move || handle.remove());
    let top_bar_data = move || (state.program_top_bar)().unwrap_or(ProgramTopBarData::finder());

    view!{
        <div class="flex flex-col bg-slate-700 bg-opacity-50 backdrop-blur-md rounded-[0.25rem] pt-1 pb-1" 
        style=drop_down_xy_style
        _ref=div_ref id="dropdown">
            {   
            move || if let Some(idx) = show().0 {

                if idx == 0 {
                    view!{
                        <DropDownListItem top_bar_field=Rc::new(LogoAboutSiteOs{})/>
                        <DropDownListItem top_bar_field=Rc::new(LogoSystemPreferences{})/>
                    }.into_view()
                } else {
                    let mut views = Vec::new();
                    let fields = &top_bar_data().data[idx-1].fields;
                    for field in fields {
                        views.push(
                            view!{
                                <DropDownListItem top_bar_field=field.clone()/>
                            }
                        )
                    }
                    views.collect_view()
                }
            } else {
                view!{}.into_view()
            }
            }
        </div>

    }
   
}

#[component]
fn DropDownListItem(top_bar_field:Rc<dyn TopBarField>) -> impl IntoView{
    let read_hover_highlight = expect_context::<RwSignal<HoverHighlight>>().read_only();
    view!{
        <div class="ml-3 mr-3 rounded-[0.25rem]" id=DROP_DOWN_LIST_ITEM_ID
        class=("hover:bg-slate-400",move || read_hover_highlight().0 )>
        <button 
            on:click={
                let top_bar_field = top_bar_field.clone();
                move |ev| top_bar_field.on_click(system,ev)
            } 
            class="rounded-md pl-2 pr-2" 
            id=DROP_DOWN_LIST_ITEM_ID
            >
        { 
            top_bar_field.name()
        }
        </button>
        </div>
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct DropDownXY((i32,i32));


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

