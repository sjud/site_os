use std::str::FromStr;
use std::collections::{HashSet, HashMap};

use super::*;
/* 

#[derive(Debug,PartialEq,Clone,Default)]
pub struct TaskBarDataList{
    pub list:HashSet<TaskBarDataItem>,
    pub spaces:usize,
}

#[derive(Hash,Debug)]
pub struct TaskBarDataItem{
    id:Uuid,
    idx:usize,
}
impl PartialEq for TaskBarDataItem{
    fn eq(&self, other: &Self) -> bool {
        // We ignore 
        self.id == other.id
    }
}
impl Eq for TaskBarDataItem{}

impl PartialOrd for TaskBarDataItem{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskBarDataItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl TaskBarDataList {
    pub fn new(ordered_ids:Vec<Uuid>) -> Self{
        Self{  
            list:{
                let mut set = HashSet::new();
                for (idx,id) in ordered_ids.into_iter().enumerate() {
                    set.insert(TaskBarDataItem{id,idx});
                }
                set
            },
            spaces:ordered_ids.len(),
        }
    }
    pub fn add_space(&mut self) {
        self.spaces += 1;
    }
    pub fn remove_space(&mut self) {
        if self.list.len() < self.spaces {
            self.spaces -= 1;
        } else {
            panic!("attempting to reduce dock spaces before removing icon list items.")
        }
    }
    pub fn shift_elements_at_pos_right(&mut self, pos:usize) {
        if self.spaces == self.list + 1 {
            for TaskBarDataItem{idx,..} in self.list.iter_mut() {
                if idx >= pos {
                    idx += 1;
                }
            }
        } else {
            panic!("attempting to shift icon on the dock when dock spaces isn't equal to current icon list + 1")
        }
    }
    pub fn shift_elements_at_pos_left(&mut self, pos:usize) {
        if self.spaces == self.list + 1 {
            for TaskBarDataItem{idx,..} in self.list.iter_mut() {
                if idx >= pos {
                    idx += 1;
                }
            }
        } else {
            panic!("attempting to shift icons on the dock when dock spaces isn't equal to current icon list + 1")
        }
    }

    pub fn add_icon(&mut self, file_id:Uuid,pos:usize,) {
        self.
        for TaskBarDataItem{idx,..} in self.list.iter_mut() {
            if idx >= pos {
                idx += 1;
            }
        }
    }
    
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct RwSpacesIds(RwSignal<Vec<Option<Uuid>>>);
#[derive(PartialEq,Clone,Debug,Copy)]
pub struct DraggingIcon{
    pub id: Uuid,
    pub offset_x: u32,
    pub offset_y : u32,
}

#[component]
pub fn ProjectDraggingIcon() -> impl IntoView {
    let system: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let icon: RwSignal<Option<DraggingIcon>> = expect_context::<RwSignal<Option<DraggingIcon>>>();
    let src = create_read_slice(
        system,move |system| {
            if let Some(icon)= icon() {
                system.img_src(icon.id)
            } else {
                "".to_string()
            }
        }
    );
    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();
    view!{
        <Show when= move || !src().is_empty() fallback = move || ()>
        <img 
        class="w-[4.5rem] z-[1000]"
        src=src style = move || {
            let DraggingIcon{offset_x,offset_y,..} = icon().unwrap();
            format!("position:absolute;left:{}px;top:{}px;",x() as u32 - offset_x , y() as u32 - offset_y )
        }
        />
        </Show>
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct TaskBarRepresentation{
    pub id:Uuid,
    pub side_spacing:Option<SideVariant>,
    pub idx:usize,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct RwTaskBarMapping(pub RwSignal<HashMap::<Uuid,TaskBarRepresentation>>);

#[component]
pub fn TaskBar() -> impl IntoView {
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
    let task_bar_ids = create_read_slice(
        system_runtime,
        |state|state.task_bar_ids());
    provide_context(create_rw_signal(None::<DraggingIcon>));
    provide_context(RwTaskBarMapping(create_rw_signal(HashMap::<Uuid,TaskBarRepresentation>::new())));
    let mapping = expect_context::<RwTaskBarMapping>();

    view!{
        <ProjectDraggingIcon/>
        <div class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 flex">
        <div class="flex">
        { move || {
            let mut new_mapping = HashMap::new();
            for (idx,id) in task_bar_ids().into_iter().enumerate() {
                new_mapping.insert(id,TaskBarRepresentation{
                    side_spacing:None,
                    idx,
                    id,
                    });
            }
            mapping.0.write_only()(new_mapping);
            view!{}.into_view()
        } }
        { move || {
            let binding = mapping.0.read_only()();
            let mut values : Vec<_> = binding.values().into_iter().collect();
            values.sort_by(|a,b|a.idx.cmp(&b.idx));
            values.into_iter().map(|val|
                if let Some(side) = val.side_spacing {
                    match side {
                        SideVariant::Left => {
                            view!{
                                <TaskBarSpace/>
                                <TaskBarItem id=val.id/>
                            }
                        },
                        SideVariant::Right => {
                            view!{
                                <TaskBarItem id=val.id/>
                                <TaskBarSpace/>
                            } 
                        }
                    }.into_view()
                } else {
                    view!{
                        <TaskBarItem id=val.id/>
                    }.into_view() 
                }).collect_view()
        }}      
            </div>
        </div>
    }
}
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum SideVariant{
    Left,
    Right
}
 // TODO make task bar item less recursive, push up the side variants and I guess add an index into a shared states.. :(
#[component]
pub fn TaskBarSpace() -> impl IntoView {
    view!{
        <div class="w-[4.5rem] h-[4.5rem]">
        </div>
    }
}
#[component]
pub fn TaskBarItem(id:Uuid) -> impl IntoView {
    let icon: RwSignal<Option<DraggingIcon>> = expect_context::<RwSignal<Option<DraggingIcon>>>();
    let mapping = expect_context::<RwTaskBarMapping>();
    let (offset,set_offset) = create_signal((0,0));
    let (dragging,set_dragging) = create_signal(false);
    let (drag_over,set_drag_over) = create_signal(false);
    view!{
        <div 
        class:hidden=dragging
        on:dragstart=  move |ev| { 
            set_dragging(true);
            icon.set(Some(DraggingIcon{
                id,
                offset_x:offset().0,
                offset_y:offset().1,
            }));
            let img = web_sys::HtmlImageElement::new().expect("should create HtmlImageElement");
            img.set_src(" ");
            ev.data_transfer().unwrap().set_drag_image(&img,0,0);
            ev.data_transfer().unwrap().set_data("text",id.to_string().as_str()).unwrap();  
        } 
        on:mouseleave = move |_| {
            mapping.0.write_only().update(|map|{map.get_mut(&id).map(|val|val.side_spacing=None);});
        }
        on:dragover = move |ev| {
            let el = event_target::<web_sys::HtmlElement>(&ev);
            let width = el.offset_width();
            let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
            if (cursor_position < width / 2) { 
                mapping.0.write_only().update(|map|{map.get_mut(&id).map(|val|val.side_spacing=Some(SideVariant::Left));});
            } else {
                mapping.0.write_only().update(|map|{map.get_mut(&id).map(|val|val.side_spacing=Some(SideVariant::Right));});
            }       
         }
        on:dragend = move |_| icon.set(None)
        on:drop=move |ev| {
            ev.prevent_default();
            let new_id = ev.data_transfer().unwrap().get_data("text").unwrap();
        }
        on:mousedown=move |ev| {
            let el = event_target::<web_sys::HtmlElement>(&ev);
            let rect = el.get_bounding_client_rect();
            let offset_x = ev.client_x() as f64 - rect.left();
            let offset_y = ev.client_y() as f64 - rect.top();
            set_offset((offset_x as u32,offset_y as u32));
        }  
        >
        <TaskBarItemButton id=id/>
        </div>
    }
}

#[component]
pub fn TaskBarItemButton(id:Uuid,) -> impl IntoView {
    let system_runtime: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let dragging_icon: RwSignal<Option<DraggingIcon>> = expect_context::<RwSignal<Option<DraggingIcon>>>();
    let img_src = create_read_slice(system_runtime,move |state|state.img_src(id));
    let (is_running,run_app) = create_slice(
        system_runtime,
        move |state| state.is_running(id),
        move |state,()| state.run_app(id,0.) 
    );
    let (is_jumping,set_jumping) = create_signal(false);
    let run_app = create_write_slice(system_runtime,
        move |state,_|state.run_app(id,0.));
    view!{
        <button 
        class=("animate-jump",move || is_jumping())
        class=" w-[4.5rem]"        //hover:scale-[1.50] hover:-translate-y-2

        id=id.to_string()
        on:click =  move |_| {
            if !is_running() {
                run_app(());
                set_jumping(true);
            }
        }>
        <img 
        //class=("hidden", move || dragging_icon().map(|icon|icon.id == id).unwrap_or_default())

        class="rounded-md" src=img_src/>
        </button> 
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
    }
}*/