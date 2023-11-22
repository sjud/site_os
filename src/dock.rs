use std::{collections::{HashSet, VecDeque, HashMap}, hash::{Hash, Hasher}, cmp::Ordering, f32::consts::E, rc::Rc, cell::RefCell};

use leptos::{html::Div, ev::drag};
use web_sys::{DragEvent, MouseEvent};

use crate::system_runtime::DragData;
/*
    Animated Dock Model

    A dock is a grey bar of a given width on the bottom of the screen.
    A dock item is an icon positioned on the grey bar such that it's never overlapping another icon.
    A dock has width W where W is a function of the number of items in the dock list and screen width. Such that
    each dock item will have a "reasonable" minimum/maximum size, minimum/maximum spacing, and the dock bar itself
    will shrink to not have "too much" space such as if it has only a few icons on a wide screen.
    A dock item has size and spacing given the width of the dock bar. All dock items have the same size and spacing.
    To calculate the left position of a dock icon, we being with dock bar padding and add to it  the dock idx and multiple it 
    by dock item spacing and size. 
    This should look like a div with children rendered by flex.
    When we drag an item, we change it's position based on mouse position. If an item is dragged off the dock,
    we remove it from the dock list, changing the dock list len, resizing the dock bar and possibly resizing/spacing the dock items.
    If an item is dragged over another item, depending on the idx of the dragged item and the idx of the drag over item, as well
    as where it was dragged over on that item. The idx of that item in the dock list will change, which will change it's left transformation
    variable which will cause it to move toward that position.
    When drop occurs on the dockbar, the item currently be dragged will have it's position changed back to a position that aligns
    with the other items on the dock bar and it will 'drift' back to its correct position.
    Item dragover rules
    If an item is directly to the left or to the right of another item, dragging that item over it's neighor's left or right (respectively)
    sides will cause no position shifts.
    When you drag over the opposite side, a right neighbor is dragged over the left side, an item will shift right.

    |xxx|yyy|zzz| start

         yyy 
    |xxx|___|zzz| item is lifted out of dock

       yyy
    |xxx|___|zzz| no change

    yyy
    |xxx|___|zzz| yyy is over left of it's left neighbor, so xxx switches indexes with it.

    yyy
    |___|xxx|zzz| xxx drifts right, as it's positon changes.

    |yyy|xxx|zzz| When letting go of yyy icon, it will drift into the empty space, because that's it's new position.

    */

use super::*;


#[derive(PartialEq,Clone,Copy,Debug)]
pub struct ZBoostDock(pub RwSignal<bool>);

#[derive(Debug,PartialEq,Clone,Copy,Eq,Default,Hash)]
pub struct MsgInner{
    dragging_id:Uuid,
    drag_over_id:Uuid,
}
#[derive(Debug,PartialEq,Copy,Clone,Eq,Hash)]
pub enum DockMsg{
    ShiftLeft(MsgInner),
    ShiftRight(MsgInner),
    InsertLeft(MsgInner),
    InsertRight(MsgInner),
    Remove(Uuid),
    Drop,
}
impl Default for DockMsg{
    fn default() -> Self {
        Self::ShiftLeft(MsgInner::default())
    }
}
#[derive(Clone,Default)]
pub struct DockList{
    list:Vec<Uuid>,
    msg_set:RwSignal<HashSet<DockMsg>>,
    queue:VecDeque<DockMsg>,
    icon_elements:HashMap<Uuid,leptos::html::HtmlElement<leptos::html::Div>>,
    left_pos:HashMap<usize,f64>,
    limbo_data:LimboData,
}

impl PartialEq for DockList {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list && self.msg_set == other.msg_set && self.queue == other.queue  && self.limbo_data == other.limbo_data
    }
}
#[derive(PartialEq,Clone,Default)]
pub struct LimboData{
    file_id:Option<Uuid>,
    limbo_ref:LimboRef,
    limbo_child:Option<web_sys::Node>,
}
impl LimboData{
    pub fn new() -> Self {
        Self {           
             limbo_ref:LimboRef(create_node_ref::<leptos::html::Div>()),
             ..Default::default()
        }
    }
}

#[derive(Clone,Default)]
pub struct LimboRef(NodeRef<leptos::html::Div>);
impl PartialEq for LimboRef{
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
impl std::fmt::Debug for LimboData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LimboRef")
         .finish()
    }
}

impl DockList {
    pub fn new(list:Vec<Uuid>) -> Self{
        Self{  
            list,
            msg_set:create_rw_signal(HashSet::new()),
            queue:VecDeque::new(),
            limbo_data:LimboData::new(),
            icon_elements:HashMap::new(),
            left_pos:HashMap::new(),
        }
    }

    pub fn put_limbo(&mut self,icon:HtmlElement<leptos::html::Div>,file_id:Uuid) {
        (*self.limbo_data.limbo_ref.0.get().unwrap())
            .append_child(&web_sys::Node::from((&*icon).clone())).unwrap();
        self.limbo_data.file_id=Some(file_id);
        self.list.iter().position(|id|*id==file_id).map(|idx|self.list.remove(idx));       
    }
    pub fn remove_limbo(&mut self) {
        self.limbo_data = LimboData::new();
    }
    pub fn push_msg(&mut self,msg:DockMsg) {
        if !self.msg_set.read_only()().contains(&msg) {
            self.queue.push_back(msg);
            self.msg_set.update(|set|{set.insert(msg);});
        }
    }
    
    pub fn pop_msg(&mut self) -> Option<DockMsg> {
        if let Some(msg) = self.queue.pop_front() {
            let msg_set = self.msg_set;
            let remove = move || msg_set.update(|set|{set.remove(&msg);});
            set_timeout(remove,std::time::Duration::from_millis(250));
            Some(msg)
        } else {
            None
        }
    }
    pub fn handle_message(&mut self,drag_data:DragData) -> Option<Option<DragData>> {
        if let Some(msg) = self.pop_msg() {
            let mut data = drag_data.clone();
            let update_mouse_pos = |new_idx:usize,data:&mut DragData| {
                    let left = self.left_pos.get(&new_idx).unwrap();
                    let mouse_pos_x = left + data.offset_x;
                    data.mouse_pos_x = mouse_pos_x;
            };
            match msg {
                DockMsg::ShiftLeft(MsgInner{
                    dragging_id,
                    drag_over_id,
                }) => {
                    
                    let dragging_idx = self.list.iter().position(|id|*id==dragging_id)
                        .expect("Shift left expects the dragging item is in the list at the moment of the message being handled.");
                    let drag_over_idx = self.list.iter().position(|id|*id==drag_over_id)
                        .expect("Shift left expects the drag over item  in the list at the moment of the message being handled.");
                    data.dock_idx = Some(drag_over_idx);
                    self.list.swap(dragging_idx,drag_over_idx);
                    update_mouse_pos(drag_over_idx,&mut data);
                    return Some(Some(data));
                },
                DockMsg::ShiftRight(MsgInner{
                    dragging_id,
                    drag_over_id,
                }) => {
                    let dragging_idx = self.list.iter().position(|id|*id==dragging_id)
                        .expect("Shift left expects the dragging item is in the list at the moment of the message being handled.");
                    let drag_over_idx = self.list.iter().position(|id|*id==drag_over_id)
                        .expect("Shift left expects the drag over item  in the list at the moment of the message being handled.");
                    data.dock_idx = Some(drag_over_idx);
                    self.list.swap(dragging_idx,drag_over_idx);
                    update_mouse_pos(drag_over_idx,&mut data);
                    return Some(Some(data));
                },
                DockMsg::InsertLeft(MsgInner{
                    dragging_id,
                    drag_over_id,
                }) => {

                    let drag_over_idx = self.list.iter().position(|id|*id==drag_over_id)
                        .expect("Shift left expects the drag over item  in the list at the moment of the message being handled.");
                    data.dock_idx = Some(drag_over_idx);
                    self.list.insert(drag_over_idx,dragging_id);
                    update_mouse_pos(drag_over_idx,&mut data);
                    return Some(Some(data));
                }
                DockMsg::InsertRight(MsgInner{
                    dragging_id,
                    drag_over_id,
                }) => {
                    let drag_over_idx = self.list.iter().position(|id|*id==drag_over_id)
                    .expect("Shift left expects the drag over item  in the list at the moment of the message being handled.");
                    if self.list.len() ==  drag_over_idx + 1 {
                        self.list.push(dragging_id);
                        update_mouse_pos(self.list.len()-1,&mut data);
                        data.dock_idx = Some(self.list.len()-1);

                    } else if self.list.len() > drag_over_idx + 1 {
                        self.list.insert(drag_over_idx + 1,dragging_id);
                        update_mouse_pos(drag_over_idx+1,&mut data);
                        data.dock_idx = Some(drag_over_idx+1);
                    } else {
                        panic!("heyo! list length less than idx + 1 ???")
                    }
                    return Some(Some(data));

                },
                DockMsg::Remove(file_id) => {
                 /*let icon = self.icon_elements.get(&file_id).cloned().unwrap();
                   data.dock_idx = None;
                   self.put_limbo(icon, file_id);*/
                   return Some(Some(data));
                },
                DockMsg::Drop => {
                    self.remove_limbo();
                    return Some(None);
                }
            }
        } else {
            None
        }
    }
    
}

//when an icon moves into limbo, put it here
#[component]
pub fn LimboIcon() -> impl IntoView{
    let state = expect_context::<SystemState>().0;
    let div_ref = create_node_ref::<leptos::html::Div>();
    let set_ref = create_write_slice(state,|state,div_ref|state.dock_list.limbo_data.limbo_ref=LimboRef(div_ref));
    view!{
        <div _ref=div_ref>
        {set_ref(div_ref);}
        </div>
    }
}

#[component]
pub fn Dock() -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    let items = create_read_slice(state,|state|{
        state.dock_list.list.clone()
    });
    let messages = create_read_slice(state,|state|state.dock_list.queue.clone());
    let handle_msg = create_write_slice(state,move |state,()|{
        let drag_data = state.drag_data.clone();
        if let Some(drag_data) = state.dock_list.handle_message(drag_data.unwrap()) {
            state.drag_data = drag_data;
        }
    });

    provide_context(ZBoostDock(create_rw_signal(false)));
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    create_effect(move |_| {
        if !messages().is_empty() {
            handle_msg(());
        }
   
        //
        //leptos::logging::log!("{:?}",items());
    });

    view!{
        <AbyssTarp/>
        <div
        class=("z-[99]",move || zboost())
        >
        <LimboIcon/>
        <div 
        class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 height-20 flex"
        >
        
        <For 
        each=move || items().into_iter()
        key=|id| *id
        children=move |file_id| 
            view! {
                <Icon file_id />
                }
        />   
        </div>
      </div>
    }
}  



#[component]
pub fn Icon(file_id:Uuid) -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    let this_idx: Signal<Option<usize>> = create_read_slice(state,move|state|
        state.dock_list.list.iter().position(move |id|*id==file_id)
    );
    let (drag_data,set_drag_data) = create_slice(
        state,
        |state|state.drag_data.clone(),
        |state,data|state.drag_data = data,
    );
    let img_src = create_read_slice(state,move |state|state.img_src(file_id));
    let (is_running,run_app) = create_slice(
        state,
        move |state| state.is_running(file_id),
        move |state,()| state.run_app(file_id,0.) 
    );
    let (is_dragging,set_is_dragging) = create_signal(false);
    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();
    let drag_data = create_read_slice(state,|state|state.drag_data.clone());
    let div_ref = create_node_ref::<leptos::html::Div>();

    let insert_map = create_write_slice(state,move|state,div_ref:HtmlElement<Div>|{
        state.dock_list.icon_elements.insert(file_id,div_ref);}
    );
    create_effect(move |_| {
        if let Some(div) = div_ref() {
            insert_map(div);
        }
    });
    let remove_map =create_write_slice(state,move|state,()|{state.dock_list.icon_elements.remove(&file_id);});
    on_cleanup(move || remove_map(()));
    let push_msg = create_write_slice(state,|state,msg|state.dock_list.push_msg(msg));
    create_effect(move |idx|  {
        let style = 
            if is_dragging() {
                if let Some(drag_data) = drag_data() {
                    format!("
                    pointer-events: none;
                    z-index:100;
                    transform:translate({}px,{}px);",
                    x() - drag_data.mouse_pos_x,
                    y() - drag_data.mouse_pos_y,
                    ) 
                } else {"".to_string()}               
            }  else {
                format!("
                transform:translate(0px,0px);
                transition: transform 0.25s; 
                transition-timing-function: linear;"
            )
        };
        let div_ref = div_ref.get_untracked().unwrap();
        
        request_animation_frame(move ||{
            div_ref.set_attribute(
            "style",
            &style
        ).unwrap();
        });
    });
   

   
    let drag_over = move |ev:DragEvent| {
        let this_idx = this_idx().unwrap();
        if let Some(mut data) = drag_data() {
            let drag_over_id = file_id;
            let dragging_id = data.dragging_id;
            let el = event_target::<web_sys::HtmlElement>(&ev);
            let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
            let dragging_self = data.dragging_id == file_id;
            let cursor_on_left = cursor_position < el.offset_width() / 2;
            let cursor_on_right = cursor_position > el.offset_width() / 2;
            let self_is_leftmost = this_idx == 0;
            let dragging_elem_is_left_neighbor = data.dock_idx.unwrap_or(usize::MAX-1) + 1 == this_idx;
            let dragging_elem_is_right_neighbor = data.dock_idx.unwrap_or(usize::MAX) == this_idx+1;
            let is_removed = data.dock_idx.is_none();
            if dragging_self  {
                return ();
            } else if !self_is_leftmost &&
                cursor_on_left &&
                dragging_elem_is_right_neighbor {
                    push_msg(DockMsg::ShiftRight(MsgInner{drag_over_id,dragging_id}))
            } else if !self_is_leftmost &&
                cursor_on_right &&
                dragging_elem_is_left_neighbor {
                    push_msg(DockMsg::ShiftLeft(MsgInner{drag_over_id,dragging_id}))
            } else if cursor_on_left && !self_is_leftmost && is_removed {
                push_msg(DockMsg::InsertLeft(MsgInner{drag_over_id,dragging_id}))
            } else if cursor_on_right && is_removed {
                push_msg(DockMsg::InsertRight(MsgInner{drag_over_id,dragging_id}))
            } else { 
                // Do nothing.
            }
        }
    };
    let left_map = create_write_slice(state,|state,(idx,left)|{state.dock_list.left_pos.insert(idx,left);});
    create_effect(move |_| {
        let rect = (*div_ref.get_untracked().unwrap()).get_bounding_client_rect();
        let left = rect.left();
        left_map((this_idx.get_untracked().unwrap(),left));
    });
    view!{
        <div _ref=div_ref
        class="z-10 w-[4.5rem] h-[4.5rem]"
            style=move || format!(
                "
                transform:translate(0px,0px);
                transition: transform 0.25s; 
                transition-timing-function: linear;
                ")
            
            on:dragstart=move|ev| {    
                let document = web_sys::window().unwrap().document().unwrap();
                let transparent_image = document.create_element("img").unwrap()
                    .dyn_into::<web_sys::HtmlImageElement>().unwrap();
                transparent_image.set_src("data:image/gif;base64,R0lGODlhAQABAIAAAP///////yH5BAEAAAAALAAAAAABAAEAAAIBRAA7");
                let data_transfer = ev.data_transfer().unwrap();
                data_transfer.set_drag_image(&transparent_image, 0, 0);
                let mouse_pos_x = ev.client_x() as f64;
                let mouse_pos_y = ev.client_y() as f64;
                let div_rect = event_target::<web_sys::HtmlDivElement>(&ev).get_bounding_client_rect();
                let div_x = div_rect.left();
                let div_y = div_rect.top();
                let offset_x = mouse_pos_x - div_x;
                let offset_y = mouse_pos_y - div_y;
                set_drag_data(Some(DragData{dragging_id:file_id,dock_idx:this_idx(),mouse_pos_x,mouse_pos_y,offset_x,offset_y}));
                set_is_dragging(true);
            }

            on:dragover=move |ev| {drag_over(ev);}

            on:dragend=move|ev|{
                set_is_dragging(false);
                set_drag_data(None);
            }
        >
           
        <div> //1
        <img 
        
        class="rounded-md" src=img_src
        />
        </div>  //1
       

        <Show when=move||this_idx().is_some() fallback=||view!{}>
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
        </Show>
        </div>
    }
}

#[component]
pub fn AbyssTarp() -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let (drag_data,set_drag_data) = create_slice(
        state,
        |state|state.drag_data.clone(),
        |state,data|state.drag_data = data,
    );
    let push_msg = create_write_slice(state,|state,msg|state.dock_list.push_msg(msg));
    let handle = window_event_listener(ev::dragover, move |ev| {
        ev.prevent_default();
    });
    on_cleanup(move || handle.remove());
    let handle = window_event_listener(ev::drop, move |ev| {
       // gloo::timers::callback::Timeout::new(100, move || set_drag_data(None)).forget();
    });
    on_cleanup(move || handle.remove());
    view!{
        <div class="w-[99vw] h-[99vh] -z-50 absolute top-0"
            class=("z-[98]",move || zboost())
            on:drop=move|ev| push_msg(DockMsg::Drop)
            
            on:dragover=move |ev| {
                ev.prevent_default();
                push_msg(DockMsg::Remove(drag_data().unwrap().dragging_id))
            }
            >
        </div>
    }
}

/* 
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
pub fn AbyssTarp() -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let (drag_data,set_drag_data) = create_slice(
        state,
        |state|state.drag_data.clone(),
        |state,data|state.drag_data = data,
    );

    let remove = create_write_slice(state,move |state,idx:usize| {
        state.dock_list.list.remove(idx);
    });
    let handle = window_event_listener(ev::dragover, move |ev| {
        ev.prevent_default();
    });
    on_cleanup(move || handle.remove());
    let handle = window_event_listener(ev::drop, move |ev| {
        gloo::timers::callback::Timeout::new(100, move || set_drag_data(None)).forget();
    });
    on_cleanup(move || handle.remove());
    view!{
        <div class="w-[99vw] h-[99vh] -z-50 absolute top-0"
            class=("z-[98]",move || zboost())
            on:drop=move|ev|{
                if let Some(data) = drag_data() {
                    if let Some(idx) = data.dock_idx {
                        remove(idx);
                    }
                }
            }
            on:dragover=move|ev|{
                ev.prevent_default();
                if let Some(mut data) = drag_data() {
                    if let Some(idx) = data.dock_idx {
                        let get_id = create_read_slice(state,move |state|state.dock_list.list.get(idx).cloned());
                        if let Some(id) = get_id() {
                            if id == data.file_id {
                                data.dock_idx = None;
                                set_drag_data(Some(data));
                            }
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
    let item_len = create_read_slice(
        system,
        |state|state.dock_list.list.len());

    view!{ 
            <ProjectDraggingIcon/>
            <div class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
            left-1/2 transform -translate-x-1/2 flex"
            class=("z-[99]",move || zboost())
            >
            <div class="flex">
            <For 
                each=move || (0..item_len())
                key=|_| Uuid::new_v4()
                children=move |i| 
                    view! {
                        <DockingItem idx=i/>
                        }
                  
            />   
                </div>
            </div>
        }
}




#[component]
pub fn DockingItem(idx:usize) -> impl IntoView {
    let system = expect_context::<SystemState>().0;
    let file_id = create_read_slice(system,move|state|state.dock_list.list.get(idx).cloned().unwrap());
    let img_src = create_read_slice(system,move |state|state.img_src(file_id()));
    let (drag_data,set_drag_data) = create_slice(system,
        |state| state.drag_data.clone(),
        move |state,data:DragData| {
            state.drag_data = Some(data);
        }
    );
    let (is_dragging,set_is_dragging) = create_signal(false);
   
    let (is_running,run_app) = create_slice(
        system,
        move |state| state.is_running(file_id()),
        move |state,()| state.run_app(file_id(),0.) 
    );
    
    let (is_jumping,set_jumping) = create_signal(false);
    let run_app = create_write_slice(system,
        move |state,_|state.run_app(file_id(),0.));
        
    let swap = create_write_slice(system,move |state,dragging_idx|{
        state.dock_list.list.swap(dragging_idx,idx);
    });
 
    let insert = create_write_slice(system,move |state,id|{
        state.dock_list.list.insert(idx,id);
    });
    let throttle_drag_over = leptos_use::use_throttle_fn_with_arg(move |ev:DragEvent| {
     
        if let Some(mut data) = drag_data() {
            let el = event_target::<web_sys::HtmlElement>(&ev);
            let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
           if let Some(dragging_idx) = data.dock_idx {
            if idx == dragging_idx {
                return ();
            }
            // We're hovering on the left side of the icon.
            if cursor_position < el.offset_width() / 2
            // and we're not trying to drag over the finder...
            && idx != 0 // we won't eval next line if 0
            && dragging_idx != idx -1 {
                leptos::logging::log!("left swap : dragging:  {dragging_idx} drag_over : {idx}");
                data.dock_idx = Some(idx);
                swap(dragging_idx);
                set_drag_data(data);
                leptos::logging::log!("drag_data idx : {:?}",drag_data().map(|data|data.dock_idx));
            } else if dragging_idx != idx + 1 
                && cursor_position > el.offset_width() / 2
                && dragging_idx != idx + 1 {
                leptos::logging::log!("right swap : dragging:  {dragging_idx} drag_over : {idx}");
                data.dock_idx = Some(idx);
                swap(dragging_idx);
                set_drag_data(data);
                leptos::logging::log!("drag_data idx : {:?}",drag_data().map(|data|data.dock_idx));
            }
           } else {
            let contains = create_read_slice(system,move |state| state.dock_list.list.contains(&data.file_id));
            if !contains() {
                insert(data.file_id);
                data.dock_idx = Some(idx);
                set_drag_data(data);
            }
           }
        }
    }, 1000.0);
    view!{
        <div>
        <div 
        class=("animate-jump",move || is_jumping())
        class=" w-[4.5rem]"        //hover:scale-[1.50] hover:-translate-y-2
        >
        <img 
        on:dragstart=move|ev| {
            leptos::logging::log!("drag start: {idx}");

            let el = event_target::<web_sys::HtmlElement>(&ev);
            let rect = el.get_bounding_client_rect();
            let offset_x = (ev.client_x() as f64 - rect.left());
            let offset_y = (ev.client_y() as f64 - rect.top());
            set_drag_data(DragData{file_id:file_id(),dock_idx:Some(idx),offset_x,offset_y,remove_from_dock:false});
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
        on:dragover=move |ev| {throttle_drag_over(ev);}
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
}*/