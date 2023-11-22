use std::{collections::{HashSet, VecDeque, HashMap}, hash::{Hash, Hasher}, cmp::Ordering, f32::consts::E, rc::Rc, cell::RefCell};

use leptos::{html::Div, ev::drag};
use web_sys::{DragEvent, MouseEvent};
use anyhow::anyhow;
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

pub type DragOverId = Uuid;
#[derive(Debug,PartialEq,Copy,Clone,Eq,Hash)]
pub enum DragOverMsg{
    ShiftLeft(DragOverId),
    ShiftRight(DragOverId),
    InsertLeft(DragOverId),
    InsertRight(DragOverId),
    DragOut,
    Drop,
}
impl Default for DragOverMsg{
    fn default() -> Self {
        Self::Drop
    }
}
#[derive(Clone,Default)]
pub struct DockList{
    icon_set:HashSet<Uuid>,
    list:Vec<Uuid>,
    msg_set:RwSignal<HashSet<DragOverMsg>>,
    queue:VecDeque<DragOverMsg>,
    dimensions_map:HashMap<usize,IconDimensions>,
    dock_dimensions:DockDimensions,
    drag_data:Option<DragData>,
}
#[derive(Debug,PartialEq,Clone,Copy,Default)]
pub struct DockDimensions{
    left:f64,
    top:f64
}
#[derive(Debug,PartialEq,Clone,Default)]
pub struct DragData{
    dragging_id:Uuid,
    dock_idx:usize,
    // offset is the distance into the icon, from left and top respectively
    offset_x:f64,
    offset_y:f64,
    // mouse pos is the absolute position relative to the view port.
    mouse_pos_x:f64,
    mouse_pos_y:f64,
    // left and top are the icon's "origin" values which are used when translating the icon while dragging under the cursor.
    left:f64,
    top:f64,
    // if in limbo don't show it's space in the dock, and on drop remove it from the dock.
    in_limbo:bool,

}

#[derive(Clone,Default,PartialEq,Copy)]
pub struct IconDimensions{
    left:f64,
    top:f64,
    width:f64,
}


impl PartialEq for DockList {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list && self.msg_set == other.msg_set && self.queue == other.queue 
    }
}




impl DockList {

    pub fn list_idx(&self,file_id:Uuid) -> Option<usize> {
        self.list.iter().position(|id|*id==file_id)
    }
    pub fn new(list:Vec<Uuid>) -> Self {
        Self{  
            icon_set:list.clone().into_iter().collect(),
            list,
            msg_set:create_rw_signal(HashSet::new()),
            queue:VecDeque::new(),
            dimensions_map:HashMap::new(),
            dock_dimensions:DockDimensions::default(),
            drag_data:None,
        }
    }
    pub fn insert(&mut self,  idx_b:usize,id_a:Uuid,) {
        if !self.icon_set.contains(&id_a) {
            self.list.insert(idx_b,id_a);
            self.icon_set.insert(id_a);
        } else {
            let idx_a = self.list.iter().position(|id_b| *id_b==id_a).unwrap();
            self.list.remove(idx_a);
            self.list.insert(idx_b,id_a);
        }
    }
    pub fn remove(&mut self, id_a:Uuid) {
        let idx_a = self.list.iter().position(|id_b| *id_b==id_a).unwrap();
        self.list.remove(idx_a);
        self.icon_set.remove(&id_a);
    }
    pub fn push_msg(&mut self,msg:DragOverMsg) {
        if !self.msg_set.read_only()().contains(&msg) {
            self.queue.push_back(msg);
            self.msg_set.update(|set|{set.insert(msg);});
        }
    }
    
    pub fn pop_msg(&mut self) -> Option<DragOverMsg> {
        if let Some(msg) = self.queue.pop_front() {
            let msg_set = self.msg_set;
            let remove = move || msg_set.update(|set|{set.remove(&msg);});
            set_timeout(remove,std::time::Duration::from_millis(250));
            Some(msg)
        } else {
            None
        }
    }
    pub fn handle_message(&mut self) {
        if let Some(msg) = self.pop_msg() {
            let mut data = self.drag_data.clone().unwrap();
            let update_mouse_pos = |this:&Self,new_idx:usize,data:&mut DragData| {
                if let Some(IconDimensions{left,..}) = this.dimensions_map.get(&new_idx).cloned() {
                    let mouse_pos_x = left + data.offset_x;
                    data.mouse_pos_x = mouse_pos_x;
                }                    
            };
            let dragging_id = data.dragging_id;
            let dragging_idx = data.dock_idx;
            match msg {
                DragOverMsg::ShiftLeft(drag_over_id) => {
                    leptos::logging::log!("shift left");
                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    data.dock_idx = drag_over_idx;
                    self.list.swap(dragging_idx,drag_over_idx);
                    update_mouse_pos(&self,drag_over_idx,&mut data);
                    self.drag_data = Some(data);
                },
                DragOverMsg::ShiftRight(drag_over_id) => {
                    leptos::logging::log!("shift right");
                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    data.dock_idx = drag_over_idx;
                    self.list.swap(dragging_idx,drag_over_idx);
                    update_mouse_pos(&self,drag_over_idx,&mut data);
                    self.drag_data = Some(data);
                },
                DragOverMsg::InsertLeft(drag_over_id) => {
                    leptos::logging::log!("insert left");
                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    self.insert(drag_over_idx,dragging_id);                
                    data.in_limbo = false;
                    data.dock_idx = drag_over_idx;
                    update_mouse_pos(&self,drag_over_idx,&mut data);
                    self.drag_data = Some(data);
                }
                DragOverMsg::InsertRight(drag_over_id) => {
                    leptos::logging::log!("insert right");
                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    if self.list.len() ==  drag_over_idx + 1 {
                        update_mouse_pos(&self,self.list.len()-1,&mut data);
                        self.insert(self.list.len()-1,dragging_id);
                        data.dock_idx = self.list.len()-1;
                    } else if self.list.len() > drag_over_idx + 1 {
                        update_mouse_pos(&self,drag_over_idx+1,&mut data);
                        self.insert(drag_over_idx+1,dragging_id);
                        data.dock_idx = drag_over_idx+1;
                    }
                    data.in_limbo = false;
                    self.drag_data = Some(data);
                },
                DragOverMsg::DragOut => {
                    leptos::logging::log!("drag out");
                    let IconDimensions{left,top,width} = self.dimensions_map.get(&dragging_idx).cloned().unwrap();
                    data.left = left-width/2.;
                    data.top = top;
                    data.in_limbo = true;
                    self.drag_data = Some(data);
                }
                DragOverMsg::Drop => {
                    self.list.remove(dragging_idx);
                    self.drag_data = None;
                }
            
            }
        } 
    }
    
}




#[component]
pub fn Dock() -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    provide_context(ZBoostDock(create_rw_signal(false)));
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;

    let items = create_read_slice(state,|state|{
        state.dock_list.list.clone()
    });
   
    let messages = create_read_slice(state,|state|state.dock_list.queue.clone());
   
    let handle_msg = create_write_slice(state,move |state,()|     
        state.dock_list.handle_message()
    );
      
    create_effect(move |_| {
        if !messages().is_empty() {
            handle_msg(());
        }
    });

    create_effect(move |_| {
        leptos::logging::log!("{:?}",items());
    });

    let div_ref = create_node_ref::<Div>();
    
    let set_pos = create_write_slice(state,|state,dock_dimensions|{state.dock_list.dock_dimensions = dock_dimensions;});
    
    create_effect(move |_| {
        let rect = (*div_ref().unwrap()).get_bounding_client_rect();
        set_pos(DockDimensions{ left: rect.left(), top: rect.top() });
    });

    view!{
        <div>
        <AbyssTarp/>
        <div 
        _ref = div_ref
        class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 height-20 flex"
        class=("z-[99]",move || zboost())
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
pub fn AbyssTarp() -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;

    let push_msg = create_write_slice(state,|state,msg|state.dock_list.push_msg(msg));
    let handle = window_event_listener(ev::dragover, move |ev| {
        ev.prevent_default();
    });
    on_cleanup(move || handle.remove());
    let handle = window_event_listener(ev::drop, move |ev| {
       // gloo::timers::callback::Timeout::new(100, move || set_drag_data(None)).forget();
    });
    on_cleanup(move || handle.remove());

    let in_limbo = create_read_slice(state,|state|{
        state.dock_list.drag_data.as_ref().map(|data|data.in_limbo).unwrap_or_default()
    });
    view!{
        <div class="w-[99vw] h-[99vh] -z-50 absolute top-0"
            class=("z-[98]",move || zboost())
            on:drop=move|_| push_msg(DragOverMsg::Drop)
            
            on:dragover=move |ev| {
                ev.prevent_default();
                if !in_limbo() {
                    push_msg(DragOverMsg::DragOut);
                }               
            }
            >
        </div>
    }
}

#[component]
pub fn Icon(file_id:Uuid) -> impl IntoView {
    let state = expect_context::<SystemState>().0;
    
    let this_idx: Signal<usize> = create_read_slice(state,move|state|
        state.dock_list.list.iter().position(move |id|*id==file_id).unwrap()
    );
    let (drag_data,set_drag_data) = create_slice(
        state,
        |state|state.dock_list.drag_data.clone(),
        |state,data|state.dock_list.drag_data = data,
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

    let div_ref = create_node_ref::<leptos::html::Div>();

    let dock_dimensions = create_read_slice(state,|state|state.dock_list.dock_dimensions);
  
    let push_msg = create_write_slice(state,|state,msg|state.dock_list.push_msg(msg));
 
    let pos_map = create_write_slice(state,|state,(idx,icon_dimensions)|{state.dock_list.dimensions_map.insert(idx,icon_dimensions);});
    
    create_effect(move |_| {
        let rect = (*div_ref().unwrap()).get_bounding_client_rect();
        let left = rect.left();
        let top = rect.top();
        let width = rect.width();
        pos_map((this_idx.get_untracked(),IconDimensions{left,top,width}));
    });

    let (is_jumping,set_jumping) = create_signal(false);

    create_effect(move|_| if is_jumping() {
        set_timeout(move || set_jumping(false), std::time::Duration::from_millis(250))
    });

    create_effect( move |_| {
        let style = {
            let dragging_style = 
            if drag_data().map(|data|data.dragging_id == file_id).unwrap_or_default() {
                let data = drag_data().unwrap();
                format!("
                pointer-events: none;
                z-index:100;
                transform:translate({}px,{}px);",
                x() - data.mouse_pos_x,
                y() - data.mouse_pos_y,
                ) 
            } else {
                format!("
                position:static;
                transform:translate(0px,0px);
                transition: transform 0.25s; 
                transition-timing-function: linear;
                ")
            };
            let limbo_style = 
            if drag_data().map(|data|data.in_limbo && data.dragging_id == file_id).unwrap_or_default() {
                let DragData{left:limbo_left,top:limbo_top,dragging_id,..} = drag_data().unwrap();
                let DockDimensions{left:dock_left,top:dock_top} = dock_dimensions();
                let left = limbo_left - dock_left;
                let top = limbo_top - dock_top;
                format!("
                    position:absolute;
                    left:{left}px;
                    top:{top}px;")
            } else {
                String::default()
            };
           
            dragging_style + &limbo_style
        };
        let div =div_ref().unwrap();
        request_animation_frame(move || {_ = div.attr("style",style);});
    });
    view!{
        <div _ref=div_ref
        class="z-10 w-[4.5rem] h-[4.5rem]"
        class=("animate-jump",move || is_jumping())
            on:dragstart=move|ev| {    
                set_img_transparent(&ev).unwrap();

                if this_idx() == 0 {
                    return ();
                }
               
                let mouse_pos_x = ev.client_x() as f64;
                let mouse_pos_y = ev.client_y() as f64;
                let div_rect = event_target::<web_sys::HtmlDivElement>(&ev).get_bounding_client_rect();
                let left = div_rect.left();
                let top = div_rect.top();
                let offset_x = mouse_pos_x - left;
                let offset_y = mouse_pos_y - top;

                set_drag_data(Some(DragData{dragging_id:file_id,dock_idx:this_idx(),
                    mouse_pos_x,
                    mouse_pos_y,
                    offset_x,
                    offset_y,
                    left,
                    top,
                    in_limbo:false,
                }));
                set_is_dragging(true);
            }

            on:dragover=move |ev| {
                let data = drag_data().unwrap();

                let dragging_self = data.dragging_id == file_id;
        
                if dragging_self {
                    return ();
                }
        
                let this_idx = this_idx();
               
                let el = event_target::<web_sys::HtmlElement>(&ev);
               
                let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
               
                let cursor_on_left = cursor_position < el.offset_width() / 2;
               
                let cursor_on_right = cursor_position > el.offset_width() / 2;
               
                let self_is_leftmost = this_idx == 0;
               
                let dragging_elem_is_left_neighbor = data.dock_idx + 1 == this_idx;
               
                let dragging_elem_is_right_neighbor = data.dock_idx - 1  == this_idx;
               
                let dragging_is_in_limbo = data.in_limbo;
                
                if !self_is_leftmost &&
                    cursor_on_left &&
                    dragging_elem_is_right_neighbor {
                        push_msg(DragOverMsg::ShiftRight(file_id))
                } else if !self_is_leftmost &&
                    cursor_on_right &&
                    dragging_elem_is_left_neighbor {
                        push_msg(DragOverMsg::ShiftLeft(file_id))
                } else if 
                    cursor_on_left && 
                    !self_is_leftmost && 
                    dragging_is_in_limbo {
                        push_msg(DragOverMsg::InsertLeft(file_id))
                } else if 
                    cursor_on_right && 
                    dragging_is_in_limbo {
                        push_msg(DragOverMsg::InsertRight(file_id))
                } 
            }

            on:dragend=move|_|{
                set_is_dragging(false);
                set_drag_data(None);
            }
            on:click =  move |_| 
            if !is_running() {
                run_app(());
                set_jumping(true);
            }
        >
           
        <div> 
        <img class="rounded-md" src=img_src />
        </div>  
       

        <Show when=move||!is_dragging() fallback=||view!{}>
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
        </Show>
        </div>
    }
}

pub fn set_img_transparent(ev:&DragEvent) -> anyhow::Result<()> {
    let transparent_image = web_sys::window().ok_or(anyhow!("window"))?
    .document().ok_or(anyhow!("document"))?
    .create_element("img").map_err(|_|anyhow!("create_element"))?
    .dyn_into::<web_sys::HtmlImageElement>().map_err(|_|anyhow!("dyn html image"))?;

    transparent_image.set_src("data:image/gif;base64,R0lGODlhAQABAIAAAP///////yH5BAEAAAAALAAAAAABAAEAAAIBRAA7");
    let data_transfer = ev.data_transfer().ok_or(anyhow!("data transfer"))?;
    data_transfer.set_drag_image(&transparent_image, 0, 0);
    Ok(())
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