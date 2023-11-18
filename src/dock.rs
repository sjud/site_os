use std::{collections::HashSet, hash::{Hash, Hasher}, cmp::Ordering, f32::consts::E};

use web_sys::{DragEvent, MouseEvent};

use super::*;
/*
Specification
A dock is a visual representation of an array of DockingSpaces
|_|_|_|...

Each docking space "holds" onto an icon
|x|y|z|...

When you remove an icon from in the Dock
y
|x|_|z|... the docking space is associated with the icon, y has idx of 1, but the docking space is hiding y's icon
y 

|x|z|...|_| we move far enough away, than y is in limbo it has no idx, and elements greater than y shift left
y


|x|z|... the docking space is deleted

|x|z|... when we stop dragging y, y is removed from limbo.

when you pickup an item and drag it
   y

|x|y|... it's added to limbo

  y

|x|z|_|... the docking space is added to the right 
|x|_|z|... the elements at and including y's hover positon (where z was) are shifted to the right,
|x|y|z|... y is added to the list of items, it's idx is z's old positon and is shown in the dock.


when you drag an icon along the bar from one position to the other (this is remove and add without deleting/adding spaces)
|a|b|c|d|e|
   b
|a|_|c|d|e| it's docking space is emptied
     b
|a|c|_|d|e| as you move move to the right, the elment you drag over shifts to the left and their docking space is kept empty
 b 
|_|a|c|d|e| or if you move to the right, all elements shift right and the hovered over position's docking space is kept empty.


Docking Space Rule:
There will always be N docking spaces, where N is the number of item in the list who have an idx.

Docking Space Icon Visibility Rule:
If the icon id in the docking space is equal to the dragging id in the dock list then the docking
space will not show the icon.


*/
#[derive(Debug,PartialEq,Clone,Default)]
pub struct DockList{
    list:Vec<Uuid>,
    /// An ID without an idx.
    limbo_id:Option<Uuid>,
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Shift{
    Left,
    Right,
}
impl DockList {
    pub fn on_drop(&mut self) {
        self.limbo_id = None;
    }
    /// When we dragover a docking space, we adjust the docking item indexes.
    pub fn drag_over(&mut self,ev:DragEvent,drag_over_idx:usize,dragging_idx:Option<usize>,dragging_id:Uuid) {
        let el = event_target::<web_sys::HtmlElement>(&ev);
       
        let width = el.offset_width();
       
        let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;

       if let Some(dragging_idx) = dragging_idx {
        if cursor_position < width / 2
        && drag_over_idx != 0 // we won't eval next line if 0
        && dragging_idx != drag_over_idx -1{
            self.list.swap(drag_over_idx,dragging_idx)
        } else if dragging_idx != drag_over_idx + 1 {
            self.list.swap(drag_over_idx,dragging_idx)

        }
       } else {
        if !self.list.contains(&dragging_id) {
            self.list.insert(drag_over_idx,dragging_id);
        }
       }
    }

    pub fn spaces_count(&self) -> usize {
        self.list.len()
    }

    pub fn put_in_limbo(&mut self,idx:usize) {
        // we'll get the actualy idx back since we don't include it in our hash or eq.
        let id = self.list.get(idx).cloned().unwrap();
        self.limbo_id = Some(id);
        self.list.remove(idx);
    }

    pub fn new(list:Vec<Uuid>) -> Self{
        Self{  
            limbo_id:None,
            list,
        }
    }

   

    pub fn items(&self) -> Vec<Uuid>{
        self.list.clone()
    }
    
}

#[derive(Debug,PartialEq,Clone,Default)]
pub struct FileDraggingData{
    file_id:Option<Uuid>,
    dock_idx:Option<usize>,
    offset_x:Option<i32>,
    offset_y:Option<i32>,
}

impl FileDraggingData{
    pub fn on_drop(&mut self) {
        self.file_id=None;self.offset_x=None;self.offset_y=None;

    }
    /// When we click on a file we store the position of the click as offset incase we drag later.
    pub fn mouse_down(&mut self, ev:MouseEvent) {
        let el = event_target::<web_sys::HtmlElement>(&ev);
        let rect = el.get_bounding_client_rect();
        self.offset_x = Some((ev.client_x() as f64 - rect.left()) as i32);
        self.offset_y = Some((ev.client_y() as f64 - rect.top()) as i32);
    }

    /// When we start dragging a file. We need the file id, the img_src of the file.
    pub fn drag_start(&mut self,id:Uuid,dock_idx:Option<usize>) {
        self.file_id=Some(id);self.dock_idx=dock_idx;
    }

}


#[component]
pub fn ProjectDraggingIcon() -> impl IntoView {
    let system: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;

    let src = create_read_slice(
        system,move |system| {
            if let Some(file_id)= system.file_dragging_data.file_id {
                system.img_src(file_id)
            } else {
                "".to_string()
            }
        }
    );
    let offsets = create_read_slice(system,|system|
        (system.file_dragging_data.offset_x,system.file_dragging_data.offset_y)
    );
    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();

    view!{
        {move || zboost.write_only()(!src().is_empty())}
        <Show when= move || !src().is_empty() fallback = move || ()>
        <AbyssTarp/>
        <img 
        class="w-[4.5rem] z-[100]"
        src=src style = move || {
            format!("pointer-events: none;position:absolute;left:{}px;top:{}px;",
            x() as i32 - offsets().0.unwrap_or_default(),
            y() as i32 - offsets().1.unwrap_or_default() 
            )
        }
        />
        </Show>
    }
}
#[component]
pub fn AbyssTarp() -> impl IntoView{
    let system: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let drop_item = create_write_slice(system,move |state,()| {
        state.file_dragging_data.on_drop();
        state.dock_list.on_drop();
    });
    view!{
        <div class="w-[99vw] h-[99vh] z-[98] absolute top-0"
            on:drop=move|ev|{drop_item(());}
            on:dragover=move|ev|ev.prevent_default()
            >
        </div>
    }
}
#[derive(PartialEq,Clone,Copy,Debug)]
pub struct ZBoostDock(pub RwSignal<bool>);
#[component]
pub fn Dock() -> impl IntoView{
    provide_context(ZBoostDock(create_rw_signal(false)));
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let system: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let items = create_read_slice(
        system,
        |state|state.dock_list.items());
    create_effect(move |_| leptos::logging::log!("{:?}",items()));
    view!{
            <ProjectDraggingIcon/>
            <div class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
            left-1/2 transform -translate-x-1/2 flex"
            class=("z-[99]",move || zboost())
            >
            <div class="flex">
            <For 
                each=move || items().into_iter().enumerate()
                key=|(_,id)| *id
                children=move |(idx,id)| {
                    view! {
                        <DockingItem file_id=id idx=idx/>
                        }
                  }
            />    
                </div>
            </div>
        }
}




#[component]
pub fn DockingItem(file_id:Uuid,idx:usize) -> impl IntoView{
    let system: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let img_src = create_read_slice(system,move |state|state.img_src(file_id));
    let mouse_down = create_write_slice(system,move |state,ev:MouseEvent|
        state.file_dragging_data.mouse_down(ev));
     
    let is_dragging = create_read_slice(system,move |state|
        state.file_dragging_data.file_id==Some(file_id));
   
    let (is_running,run_app) = create_slice(
        system,
        move |state| state.is_running(file_id),
        move |state,()| state.run_app(file_id,0.) 
    );
    let (is_jumping,set_jumping) = create_signal(false);
    let run_app = create_write_slice(system,
        move |state,_|state.run_app(file_id,0.));
        
    let drag_over = create_write_slice(system,
        move |state,ev|{
            if let (Some(dragging_id),dragging_idx) = (state.file_dragging_data.file_id,state.file_dragging_data.dock_idx) {
                state.dock_list.drag_over(ev,idx,dragging_idx,dragging_id);
            }
    });
    let mut throttled_drag_over = leptos_use::use_throttle_fn_with_arg(
        move |ev| {
            drag_over(ev)
        },
        2000.0,
    );
    let drag_start = create_write_slice(system,move |state,ev:DragEvent| {
            state.file_dragging_data.drag_start(file_id,Some(idx));
    });
    
    view!{
        <div>
        <div 
        class=("animate-jump",move || is_jumping())
        class=" w-[4.5rem]"        //hover:scale-[1.50] hover:-translate-y-2
        >
        <img 
        on:mousedown = move |ev| {
            mouse_down(ev)
        }
        on:dragstart=move|ev| {
            drag_start(ev);
        }
        on:click =  move |_| 
            if !is_running() {
                run_app(());
                set_jumping(true);
            }
        on:dragover=move |ev| {
            throttled_drag_over(ev);
        }
        class=("opacity-0",move || is_dragging())
        class="rounded-md" src=img_src
        />
        </div> 
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
        </div>
    }
}