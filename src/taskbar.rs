use std::str::FromStr;
use std::collections::HashSet;

use super::*;


#[derive(Debug,PartialEq,Clone,Default)]
pub struct TaskBarDataList{
    files_on_taskbar:HashSet<Uuid>,
}
impl TaskBarDataList {
    pub fn new(task_bar_ids:Vec<Uuid>) -> Self{
        Self{  
            files_on_taskbar:{
                let mut set = HashSet::new();
                for id in task_bar_ids {
                    set.insert(id);
                }
                set
            }
        }
    }
    pub fn add_file(&mut self, file_id:Uuid) {
        self.files_on_taskbar.insert(file_id);
    }

    pub fn is_on_taskbar(&self,id:Uuid) -> bool {
        self.files_on_taskbar.contains(&id)
    }
  

    pub fn list(&self) -> Vec<Uuid> {
        self.files_on_taskbar.clone().into_iter().collect::<_>()
     
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

#[component]
pub fn TaskBar() -> impl IntoView {
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
    let task_bar_ids = create_read_slice(
        system_runtime,
        |state|state.task_bar_ids());
    provide_context(create_rw_signal(None::<DraggingIcon>));
       view!{
        <ProjectDraggingIcon/>
        <div class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 flex">
        <div class="flex">
        { move || {
            task_bar_ids().into_iter().map(|id|view!{
                <TaskBarItem id=Some(id)/>
            }).collect_view()
        }
        }        
            </div>
        </div>
    }
}
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum SideVariant{
    None,
    Left,
    Right
}
 // TODO make task bar item less recursive, push up the side variants and I guess add an index into a shared states.. :(

#[component]
pub fn TaskBarItem(id:Option<Uuid>) -> impl IntoView {
    let icon: RwSignal<Option<DraggingIcon>> = expect_context::<RwSignal<Option<DraggingIcon>>>();
    let (offset,set_offset) = create_signal((0,0));
    let (dragging,set_dragging) = create_signal(false);
    let (drag_over,set_drag_over) = create_signal(false);
    let (side_variant,set_side_variant) = create_signal(SideVariant::None);
    create_effect(move |_|  leptos::logging::log!("drag over: {}; side_variant : {:?}",drag_over(),side_variant()));
    view!{
        <div  //on:mouseleave = move |ev| set_side_variant(SideVariant::None) 
        >

        <Show when=move || side_variant() == SideVariant::Left fallback=||()>
        <TaskBarItem id=None/>
        </Show>

        <div 
        class:hidden=dragging
        on:dragstart=  move |ev| { 
            if let Some(id) = id {
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
        } 
        on:mousemove = move |ev| {
            if id.is_some() && drag_over() {
                leptos::logging::log!("here");
                let el = event_target::<web_sys::HtmlElement>(&ev);
                let width = el.offset_width();
                let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
                if (cursor_position < width / 2) { 
                    set_side_variant(SideVariant::Left)
                } else {
                    set_side_variant(SideVariant::Right)
                }
            }
        }
    
        on:dragover = move |ev| {
            if id.is_some() {
                set_drag_over(true);leptos::logging::log!("drag over?")
            }
        }
        on:dragend = move |_| icon.set(None)
        on:drop=move |ev| {
            ev.prevent_default();
            let new_id = ev.data_transfer().unwrap().get_data("text").unwrap();
        }
        on:mousedown=move |ev| {
            if id.is_some() {
                let el = event_target::<web_sys::HtmlElement>(&ev);
                let rect = el.get_bounding_client_rect();
                let offset_x = ev.client_x() as f64 - rect.left();
                let offset_y = ev.client_y() as f64 - rect.top();
                set_offset((offset_x as u32,offset_y as u32));
            }
        }
        >
        {
            move || id.map(|id|view!{<TaskBarItemButton id/>}).unwrap_or(().into_view())
        }
        </div>

        <Show when=move || side_variant() == SideVariant::Right fallback=||()>
        <TaskBarItem id=None/>
        </Show>

        </div>
    }
}

#[component]
pub fn TaskBarItemButton(id:Uuid,) -> impl IntoView {
    let system_runtime: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let icon: RwSignal<Option<DraggingIcon>> = expect_context::<RwSignal<Option<DraggingIcon>>>();
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
        //class=("hidden", move || icon().map(|icon|icon.id == id).unwrap_or_default())

        class="rounded-md" src=img_src/>
        </button> 
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
    }
}