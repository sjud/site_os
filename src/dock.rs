use std::{collections::HashSet, hash::{Hash, Hasher}, cmp::Ordering, f32::consts::E};

use web_sys::{DragEvent, MouseEvent};

use super::*;
/*
Model 

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
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Shift{
    Left,
    Right,
}
impl DockList {
 
    pub fn new(list:Vec<Uuid>) -> Self{
        Self{  
            list,
        }
    }
}





#[component]
pub fn ProjectDraggingIcon() -> impl IntoView {
    let system: RwSignal<SystemRuntime> =expect_context::<SystemState>().0;
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let drag_data = create_read_slice(system,|state|state.drag_data.clone());

    let src = create_read_slice(
        system,move |system| 
        drag_data().map(|data|system.img_src(data.file_id))
    );

    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();

    view!{
        {move || zboost.write_only()(!drag_data().is_none())}
        <AbyssTarp/>
        <Show when= move || drag_data().is_some() fallback = move || ()>
        <img 
        class="w-[4.5rem] z-[100]"
        src=src().unwrap() style = move || {
            format!("pointer-events: none;position:absolute;left:{}px;top:{}px;",
            x() - drag_data().unwrap().offset_x,
            y() - drag_data().unwrap().offset_y,
            )
        }
        />
        </Show>
    }
}

#[component]
pub fn AbyssTarp() -> impl IntoView{
    let state = expect_context::<SystemState>().0;
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let (drag_data,set_drag_data) = create_write_slice(
        system,
        |state,data|state.drag_data = data,
        |state|state.drag_data.clone()
    );

    let remove = create_write_slice(state,move |state,idx:usize| {
        state.dock_list.list.remove(idx);
    });

    view!{
        <div class="w-[99vw] h-[99vh] z-[-98] absolute top-0"
            class=("z-[98]",move || zboost())
            on:drop=move|ev|{
                if let Some(data) = drag_data() {
                    remove(data.idx);
                }
            }
            on:dragover=move|ev|{
                ev.prevent_default();
                let mut data = drag_data();
                if let Some(idx) = data.dock_idx {
                    let get_id = create_read_slice(state,move |state|state.dock_list.list.get(idx).cloned());
                    if let Some(id) = get_id() {
                        if id == data.file_id {
                            data.dock_idx = None;
                            set_drag_data(data)
                        }
                    }
                }
            }
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
    let system =expect_context::<SystemState>().0;
    let items = create_read_slice(
        system,
        |state|state.dock_list.list.clone());

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
pub fn DockingItem(file_id:Uuid,idx:usize) -> impl IntoView {
    let system = expect_context::<SystemState>().0;
    let img_src = create_read_slice(system,move |state|state.img_src(file_id));
 
    let (is_dragging,set_is_dragging) = create_signal(false);
   
    let (is_running,run_app) = create_slice(
        system,
        move |state| state.is_running(file_id),
        move |state,()| state.run_app(file_id,0.) 
    );
    
    let (is_jumping,set_jumping) = create_signal(false);
    let run_app = create_write_slice(system,
        move |state,_|state.run_app(file_id,0.));
        
    let swap = create_write_slice(system,move |state,dragging_idx|{
        state.dock_list.list.swap(dragging_idx,idx);
    });
 
    let insert = create_write_slice(system,move |state,file_id|{
        state.dock_list.list.insert(idx,file_id);
    });
 
    view!{
        <div>
        <div 
        class=("animate-jump",move || is_jumping())
        class=" w-[4.5rem]"        //hover:scale-[1.50] hover:-translate-y-2
        >
        <img 
        on:dragstart=move|ev| {
            let el = event_target::<web_sys::HtmlElement>(&ev);
            let rect = el.get_bounding_client_rect();
            let offset_x = (ev.client_x() as f64 - rect.left());
            let offset_y = (ev.client_y() as f64 - rect.top());
            ev.data_transfer().unwrap().set_data("text/plain", 
                &serde_json::to_string(&DragTransferData{file_id,idx:Some(idx),offset_x,offset_y}).unwrap()
            ).unwrap();
            set_is_dragging(true);
        }
        on:dragend=move|ev| {
            set_is_dragging(false);
        }
        on:click =  move |_| 
            if !is_running() {
                run_app(());
                set_jumping(true);
            }
        on:dragover=move |ev| {
            let mut data = DragTransferData::from_event(&ev).unwrap();
            let el = event_target::<web_sys::HtmlElement>(&ev);
            let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
           if let Some(dragging_idx) = data.idx {
            // We're hovering on the left side of the icon.
            if cursor_position < el.offset_width() / 2
            // and we're not trying to drag over the finder...
            && idx != 0 // we won't eval next line if 0
            && dragging_idx != idx -1 {
                swap(dragging_idx);
            } else if dragging_idx != idx + 1 {
                swap(dragging_idx);
            }
           } else {
            let contains = create_read_slice(system,move |state| state.dock_list.list.contains(&data.file_id));
            if !contains() {
                insert(data.file_id);
                data.idx = Some(idx);
                ev.data_transfer()
                    .unwrap()
                    .set_data("text/plain",&serde_json::to_string(&data).unwrap())
                    .unwrap();
            }
           }
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