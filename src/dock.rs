use std::{collections::{HashSet, VecDeque, HashMap}, hash::{Hash, Hasher}, cmp::Ordering, f32::consts::E, rc::Rc, cell::RefCell};

use leptos::{html::{Div, P}, ev::drag};
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
pub static SHIFT_TIME:u64 = 250;
pub static DOCK_AREA_ABOVE:f64 = 100.;
pub static SHIFT_SPACE:f64 = 76.5;
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DockData{
    init_ids:RwSignal<Vec<Uuid>>,
    bg_rect:RwSignal<Option<Rect>>,
    dock_icons:RwSignal<HashMap<Uuid,(RwSignal<DockIconData>,RwSignal<DockIconSpacingData>)>>,
    drag_start_data:RwSignal<Option<DragStartData>>,
}
impl DockData {
    pub fn init(list:Vec<Uuid>) -> Self {
        Self{
            init_ids: create_rw_signal(list.clone()),
            bg_rect: create_rw_signal(None),
            drag_start_data: create_rw_signal(None),
            dock_icons: create_rw_signal(list.into_iter().map(|id: Uuid|(id,{
                let move_data = create_rw_signal(MoveData::default());
                let rect = create_rw_signal(Rect::default());
                (
                    create_rw_signal(DockIconData{
                        is_being_dragged: create_rw_signal(false),
                        rect,
                        move_data,
                    }),
                    create_rw_signal(DockIconSpacingData{
                        move_data,   
                        icon_rect:rect,                
                    })
                )
            })).collect()),
        }
    }

    pub fn init_values(&self) {
        let dock_ref = create_node_ref::<Div>();
 
        let bg_rect = self.bg_rect;
        let dock_icons = self.dock_icons;
        create_effect(move|_| {
            let rect = (*dock_ref().unwrap()).get_bounding_client_rect();
            bg_rect.set_untracked(Some(Rect::new(rect)));
        });
        let item = self.init_ids;
        view!{
            <div _ref=dock_ref class="opacity-0 -z-100">
            <For 
            each=move || items.get_untracked().into_iter()
            key=|id| *id
            children=move |file_id| {
                view! {
                    <InitIcon file_id dock_icons/>
                    <InitIconSpacing/>
                    }   
            }
            />   
            </div>
        };
    }

    pub fn clear_moves(&self) {
        for (data,_) in self.dock_icons.get_untracked().values() {
            let data = data.get_untracked();
            data.move_data.set_untracked(MoveData::default());
        }
    }

    pub fn clear_drag_start(&self) {
        self.drag_start_data.set_untracked(None);
    }

    /// Should only be called from drag_over event.
    pub fn drag_over_icon(&self,
        ev:DragEvent,
        file_id:Uuid,) {
        let (icon_data,_) = self.dock_icons.get_untracked().get(&file_id).cloned().unwrap();
        let drag_start_data = self.drag_start_data.get_untracked().unwrap();
        let mouse_pos_x = ev.client_x() as f64;
        let rect = icon_data.get_untracked().rect.get_untracked();
        let left_bounds = rect.left..(rect.width/2.);
        let right_bounds = left_bounds.end..=rect.right;
        let move_data = icon_data.get_untracked().move_data.get_untracked();
        if left_bounds.contains(&mouse_pos_x) && drag_start_data.drag_start_x > rect.right && !move_data.has_moved_right {
            icon_data.get_untracked().move_data.update_untracked(|data|data.has_moved_right=true);
            icon_data.get_untracked().rect.update(|rect|{
                rect.left -= SHIFT_SPACE;
                rect.right -= SHIFT_SPACE;
            });
        } else if right_bounds.contains(&mouse_pos_x) && drag_start_data.drag_start_x < rect.left && !move_data.has_moved_left {
            icon_data.get_untracked().move_data.update_untracked(|data|data.has_moved_left=true);
            icon_data.get_untracked().rect.update(|rect|{
                rect.left += SHIFT_SPACE;
                rect.right += SHIFT_SPACE;
            });
        } 
        // Find out whether to shift left/right given drag_start_x and the icon_data left/right and mouse_pos_x
    }

    pub fn drag_over_spacing(&self,
        ev:DragEvent,
        file_id:Uuid,
        ) {
            let (_,spacing_data) = self.dock_icons.get_untracked().get(&file_id).cloned().unwrap();
            let list = self.init_ids.get_untracked();
            let idx = list.iter().position(|&id|id==file_id).unwrap();
            let list_len = list.len();
            let left_range = 0..=idx;
            let right_range = idx+1..list_len;
            for (i,id) in list.iter().enumerate() {
                let icon_data = self.dock_icons.get_untracked().get(i).cloned().unwrap().0.get_untracked();
                let move_data = icon_data.move_data.get_untracked();
                if left_range.contains(&i) && !move_data.has_shifted_left_new_addition {
                    icon_data.move_data.update_untracked(|data|data.has_shifted_left_new_addition=true);
                    icon_data.rect.update(|rect|{
                        rect.left -= SHIFT_SPACE/2.;
                        rect.right -= SHIFT_SPACE/2.;
                    });
                } else if right_range.contains(&i) && !move_data.has_shifted_right_new_addition {
                    icon_data.move_data.update_untracked(|data|data.has_shifted_right_new_addition=true);
                    icon_data.rect.update(|rect|{
                        rect.left += SHIFT_SPACE/2.;
                        rect.right += SHIFT_SPACE/2.;
                    });
                }
            }
        }
}

#[component]
pub fn InitIconSpacing() -> impl IntoView{
    view!{
        <div class="w-auto min-w-[0.25rem">
        </div>
    }
}

#[component]
pub fn InitIcon(file_id:Uuid,dock_icons:RwSignal<HashMap<Uuid,(RwSignal<DockIconData>,RwSignal<DockIconSpacingData>)>>) -> impl IntoView {
    let icon_ref = create_node_ref::<Div>();
    create_effect( move |_| {
        let rect = (*icon_ref().unwrap()).get_bounding_client_rect();
        dock_icons.update_untracked(|map|{
            let (icon_data,_) = map.get_mut(&file_id).unwrap();
            icon_data.get_untracked().rect.set(Rect::new(rect));
        });
    });

    view!{
        <div _ref=icon_ref
        class="w-[4.5rem] h-[4.5rem]"
        >
        <div> 
        </div>  
        <div 
        class="h-1 w-1 ml-auto mr-auto"> 
        </div>
        </div>
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DragStartData{
    pub drag_start_x:f64,
    pub offset_x:f64,
    pub offset_y:f64,
}
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DockIconData{
    is_being_dragged:RwSignal<bool>,
    move_data:RwSignal<MoveData>,
    rect:RwSignal<Rect>,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DockIconSpacingData{
    move_data:RwSignal<MoveData>,
    icon_rect:RwSignal<Rect>,
}

#[derive(Copy,Clone,Debug,PartialEq,Default)]
pub struct MoveData{
    has_moved_left:bool,
    has_moved_right:bool,
    has_shifted_left_new_addition:bool,
    has_shifted_right_new_addition:bool,
}

#[derive(Copy,Clone,Debug,PartialEq,Default)]
pub struct Rect{
    left:f64,
    top:f64,
    right:f64,
    bottom:f64,
    height:f64,
    width:f64,
}

impl Rect{
    pub fn new(rect:web_sys::DomRect) -> Self {
        Self { 
            left: rect.left(), 
            top: rect.top(), 
            right: rect.right(), 
            bottom: rect.bottom(), 
            height: rect.height(), 
            width: rect.width() 
        }
    }
}


#[component]
pub fn Dock() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    state.dock.init_values();
    let init_ids = state.dock.init_ids.get_untracked();
    let track_ids = create_memo(move |_|(state.dock.init_ids)());
    create_effect(move |_| {
        if init_ids != track_ids() {
            state.dock.init_values();
        }
    });
    view!{
        <div 
        class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 flex"
        >
        <For 
        each=move || (state.dock.init_ids)().into_iter()
        key=|id| *id
        children=move |file_id| {
            view! {
                <Icon file_id/>
                <IconSpacing file_id/>
            }
        }
        />   
        </div>
    }
}

pub fn set_img_transparent(ev:&DragEvent) -> anyhow::Result<()> {
    let transparent_image = web_sys::window()
    .ok_or(anyhow!("window"))?
    .document()
    .ok_or(anyhow!("document"))?
    .create_element("img")
    .map_err(|_|anyhow!("create_element"))?
    .dyn_into::<web_sys::HtmlImageElement>()
    .map_err(|_|anyhow!("dyn html image"))?;
    transparent_image.set_src("data:image/gif;base64,R0lGODlhAQABAIAAAP///////yH5BAEAAAAALAAAAAABAAEAAAIBRAA7");
    let data_transfer = ev.data_transfer().ok_or(anyhow!("data transfer"))?;
    data_transfer.set_drag_image(&transparent_image, 0, 0);
    Ok(())
}

#[component]
pub fn IconSpacing(file_id:Uuid) -> impl IntoView {
    /*
        when dragged, over expand, on drop the icon moves to occupy the position icon spacing expanded to
        and after the transition we'll insert the icon id in the ids of the dock (in correct position) and re_init it.
        Hide if file_id is being dragged.
     */
    let state = expect_context::<GlobalState>();
    let spacing_ref = create_node_ref::<Div>();
    let (_,data) = state.dock.dock_icons.get_untracked().get(&file_id).cloned().unwrap();
    let rect = data.get_untracked().icon_rect;
    create_effect( move |_| {
        let Rect {right,top,..}= rect();
        _ = spacing_ref().unwrap()
            .style("left",&format!("{}px",right))
            .style("top",&format!("{}px",top));    
    });
    view!{
        <div _ref=spacing_ref
        class="w-auto min-w-[0.25rem] bg-blue-200"
            style="transition:width 250ms linear,left 250ms linear;"
            on:dragover = move |ev| {

            }
            on:drop = move |ev| {

            }
            on:transitionrun = move |ev| {
                spacing_ref().unwrap().style("pointer-events","none");
            }   

            on:transitionend = move |ev| {
                spacing_ref().unwrap().style("pointer-events","auto");
            }
        >
        </div>
    }
}

#[component]
pub fn Icon(file_id:Uuid) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let icon_ref = create_node_ref::<leptos::html::Div>();
    let (data,_) = state.dock.dock_icons.get_untracked().get(&file_id).cloned().unwrap();
    let is_being_dragged = data.get_untracked().is_being_dragged;
    let img_src = move || state.img_src(file_id);
    let is_running = move || state.is_running(file_id);
    let run_app = move || state.run_app(file_id,0.);
    let (is_jumping,set_jumping) = create_signal(false);
    let rect = data.get_untracked().rect;
    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();
    let drag_data = state.dock.drag_start_data;
    create_effect( move |_| {
        if is_being_dragged() {
            let drag_data = drag_data.get_untracked().unwrap();
            _ = icon_ref().unwrap()
                .style("left",&format!("{}px",x()+drag_data.offset_x))
                .style("top",&format!("{}px",y()+drag_data.offset_y)); 
        } else {
            let Rect {left,top,..} = rect();
            _ = icon_ref().unwrap()
                .style("left",&format!("{}px",left))
                .style("top",&format!("{}px",top));
        }     
    });

    view!{
        <div _ref=icon_ref
        class="z-10 w-[4.5rem] h-[4.5rem]"
        class=("animate-jump",move || is_jumping())
            on:dragstart=move|ev| {    
                icon_ref().unwrap().style("transition","");
                set_img_transparent(&ev).unwrap();
                let x = ev.client_x();
                state.dock.drag_start_x.set_untracked(Some(x));
            }
            on:dragover=move |ev| {
            
            }
            on:drop=move |ev| {
                // when you drop an icon on an icon it moves into it's spacing..
                // i.e find out the left top of the spacing and move the icon there, stretch the spacing and reinit the dock.
            }
            on:dragend=move|_|{
                icon_ref().unwrap().style("transition","left 250ms linear, top 250ms linear");
            }
            on:click =  move |_| 
            if !is_running() {
                run_app();
                set_jumping(true);
            }
            on:transitionend = move |ev| {
                icon_ref().unwrap().style("pointer-events","auto");
            }
            on:transitionrun = move |ev| {
                icon_ref().unwrap().style("pointer-events","none");
            }   
        >
           
        <div> 
        <img class="rounded-md" src=img_src />
        </div>  
        <Show when=move||!is_being_dragged() fallback=||view!{}>
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
        </Show>
        </div>
    }
}




/* // take 3
pub static SHIFT_TIME:u64 = 250;
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DockEngine{
    icon_data:RwSignal<HashMap<Uuid,RwSignal<IconData>>>,
    left_idx:RwSignal<HashMap<usize,f64>>,
    top_idx:RwSignal<HashMap<usize,f64>>,
    list:RwSignal<Vec<Uuid>>,
    drag_data:RwSignal<Option<DragData>>,
    insert_set:RwSignal<HashSet<InsertMsg>>,
    insert_queue:RwSignal<VecDeque<InsertMsg>>,
}
#[derive(Clone)]
pub struct IconData{
    file_id:Uuid,
    animation_state:RwSignal<Option<AnimationState>>,
    left:RwSignal<f64>,
    el:NodeRef<Div>,
}
#[derive(Clone,Debug,PartialEq)]
pub enum AnimationState{
    BeingDragged,
    ShiftingLeft,
    ShiftingRight,
    ReturningToDock,
}
/// The two drag events we are watching for,
/// they  are initiated by static elements on the dock when they are interacted with, i.e when drag starts or they are dragged over.
pub enum DragEventMsg{
    DragStart(Uuid,DragEvent),
    DragOver(Uuid,DragEvent),
    DragEnd,
}

#[derive(Debug,Hash,PartialEq,Clone,Eq,Copy)]
pub enum InsertMsg{
    InsertLeft(Uuid),
    InsertRight(Uuid),
}
/// Drag Data is data we have on the object currently being dragged.
#[derive(Debug,PartialEq,Clone,Default,Copy)]
pub struct DragData{
    dragging_id:Uuid,
    dock_idx:usize,
    //mouse pos is the absolute position relative to the view port (at the time of click)
    mouse_pos_x:f64,
    mouse_pos_y:f64,
}

impl DockEngine{
    
    pub fn new(list:Vec<Uuid>) -> Self {
        let map = list.iter().map(|&id|(id,create_rw_signal(IconData{
            file_id:id,
            animation_state:create_rw_signal(None::<AnimationState>),
            left:create_rw_signal(0.),
            el:create_node_ref::<Div>(),
        })))
        .collect::<HashMap<Uuid,RwSignal<IconData>>>();
        Self{  
            icon_data: create_rw_signal(map),
            list: create_rw_signal(list),
            drag_data: create_rw_signal(None),
            insert_set: create_rw_signal(HashSet::new()),
            insert_queue: create_rw_signal(VecDeque::new()),
            left_idx: create_rw_signal(HashMap::new()),
            top_idx: create_rw_signal(HashMap::new()),
        }
    }
  
   

    pub fn drag_event(&self,msg:DragEventMsg) {
        let icon_data = self.icon_data.clone();
        let animate = move |anim_state:AnimationState,file_id:Uuid| {
            icon_data.get_untracked().get(&file_id).unwrap().update(|data|data.animation_state.set(Some(anim_state)));
        };

        match msg {
            DragEventMsg::DragStart(dragging_id,ev) => {
                let dock_idx = self.list.get_untracked().iter().position(|&id|id==dragging_id).unwrap();
                let (mouse_pos_x,mouse_pos_y) = (ev.client_x() as f64, ev.client_y() as f64);
                let drag_data = DragData{
                    dragging_id,
                    dock_idx,
                    mouse_pos_x,
                    mouse_pos_y,
                };
                self.drag_data.set(Some(drag_data));
                self.icon_data.get_untracked().get(&dragging_id).unwrap()
                    .update(|data|data.animation_state.set(Some(AnimationState::BeingDragged)));
            },
            DragEventMsg::DragOver(drag_over_id,ev) => {          
                let idx = self.list.get_untracked().iter().position(|&id|id==drag_over_id).unwrap();    
               let el = event_target::<web_sys::HtmlElement>(&ev);
               let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
               let (cursor_on_left,cursor_on_right) = (
                cursor_position < el.offset_width() / 2,
                cursor_position > el.offset_width() / 2
                );
              let drag_data = self.drag_data.get_untracked().unwrap();
               let is_leftmost = idx == 0;
              
               let (dragging_elem_is_lesser,dragging_elem_is_greater) = (
                drag_data.dock_idx < idx, 
                drag_data.dock_idx  > idx,
                );
                let list = self.list;
                let insert_set = self.insert_set;
                let insert_queue = self.insert_queue;
               if !is_leftmost &&
                   cursor_on_left &&
                   dragging_elem_is_greater {
                    let mut ids = Vec::new();
                    for i in idx..drag_data.dock_idx {
                        ids.push(list.get_untracked().get(i).cloned().unwrap())
                    };
                    for id in ids {
                        animate(AnimationState::ShiftingRight,id);
                        let msg = InsertMsg::InsertLeft(id);
                        insert_set.update(|set|
                            if set.insert(msg) {
                                insert_queue.update(|q|q.push_back(msg));
                            }
                        );
                    }
               } else if !is_leftmost &&
                   cursor_on_right &&
                   (dragging_elem_is_lesser) 
                    {
                        let mut ids = Vec::new();
                        for i in drag_data.dock_idx+1..=idx {
                            ids.push(list.get_untracked().get(i).cloned().unwrap())
                        };
                        for id in ids {
                            animate(AnimationState::ShiftingLeft,id);
                            insert_set.update(|set|{
                                let msg = InsertMsg::InsertRight(id);
                                if set.insert(msg) {
                                    insert_queue.update(|q|q.push_back(msg));
                                }
                        });
                        }       
                    }
            },
            DragEventMsg::DragEnd => {
                let dragging_id = self.drag_data.get_untracked().unwrap().dragging_id;
                self.icon_data.get_untracked().get(&dragging_id).unwrap().update(|data|data.animation_state.set(Some(AnimationState::ReturningToDock)));
                self.drag_data.set(None);
            }
        }
    }
    pub fn handle_insert_msg(&self) {
        let insert_set = self.insert_set;
        let list = self.list;
        let drag_data = self.drag_data;
        let dragging_icon_data = self.icon_data.get_untracked().get(&drag_data.get_untracked().unwrap().dragging_id).cloned().unwrap();
        self.insert_queue.update(|queue| if let Some(msg) = queue.pop_front() {
            match msg {
                InsertMsg::InsertLeft(right_id) => {
                    let idx = list.get_untracked().iter().position(|&id|id==right_id).unwrap();
                    let dragging_id = drag_data.get_untracked().unwrap().dragging_id;
                    let dragging_idx = list.get_untracked().iter().position(|&id|id==dragging_id).unwrap();
                    if idx > 0 {
                        list.update_untracked(|list|{list.remove(dragging_idx);});
                        list.update(|list|list.insert(idx-1 ,dragging_id));
                    }
                    leptos::logging::log!("{:?}",list());
                    set_timeout(move || {
                        insert_set.update(|set|{set.remove(&msg);});
                    },std::time::Duration::from_millis(SHIFT_TIME));
                }
                InsertMsg::InsertRight(left_id) =>{
                    let idx = list.get_untracked().iter().position(|&id|id==left_id).unwrap();
                    let dragging_id = drag_data.get_untracked().unwrap().dragging_id;
                    let dragging_idx = list.get_untracked().iter().position(|&id|id==dragging_id).unwrap();
                    list.update_untracked(|list|{list.remove(dragging_idx);});
                    list.update(|list|list.insert(idx,dragging_id));

                    set_timeout(move || {
                        insert_set.update(|set|{set.remove(&msg);});
                    },std::time::Duration::from_millis(SHIFT_TIME-10));
                },
            }
        });
    }
    pub fn animate_state(&self,file_id:Uuid,left_file_id:Option<Uuid>,right_file_id:Option<Uuid>,x:Option<f64>,y:Option<f64>) {
        let data = self.icon_data.get_untracked().get(&file_id).cloned().unwrap();
        if let (Some(el),Some(state)) = (
            data.get_untracked().el.get_untracked(),
            data.get_untracked().animation_state.get_untracked()
        ) {
            match state {
                AnimationState::BeingDragged => {
                    let DragData {  mouse_pos_x,mouse_pos_y,.. } = self.drag_data.get_untracked().unwrap();
                    let left = x.unwrap() - mouse_pos_x;
                    let top = y.unwrap() - mouse_pos_y;
                    _ = el
                    .style("z-index","100")
                    .style("transform",&format!("translate({}px,{}px)",left,top))
                    .style("transition","transform 0ms")
                    .style("pointer-events","none");
                },
                AnimationState::ShiftingLeft => {
                    leptos::logging::log!("{file_id} : {state:?}");
                    let idx = self.list.get_untracked().iter().position(|&id|id==file_id).unwrap();
                    let left = self.left_idx.get_untracked().get(&idx).cloned().unwrap();
                    let left_idx =  self.list.get_untracked().iter().position(|&id|id==left_file_id.unwrap())
                    .unwrap();
                    let left_el_left = self.left_idx.get_untracked().get(&left_idx).cloned().unwrap();
                    _ = el
                    .style("transform",&format!("translate({}px,0px)",left_el_left-left))// left_el_left is less than left which will give us <0 which is left shift along the x axis
                    .style("transition",&format!("transform {}ms",SHIFT_TIME))
                    .style("transition-timing-function","linear")
                    .style("pointer-events","none");
                },
                AnimationState::ShiftingRight => {
                    leptos::logging::log!("{file_id} : {state:?}");
                    let idx = self.list.get_untracked().iter().position(|&id|id==file_id).unwrap();
                    let left = self.left_idx.get_untracked().get(&idx).cloned().unwrap();
                    let right_idx =  self.list.get_untracked().iter().position(|&id|id==left_file_id.unwrap())
                    .unwrap();
                    let right_el_left = self.left_idx.get_untracked().get(&right_idx).cloned().unwrap();
                    _ = el
                    .style("transform",&format!("translate({}px,0px)",right_el_left-left))//right_el_left > left -> diff is >0 -> shift right on x axis.
                    .style("transition",&format!("transform {}ms",SHIFT_TIME))
                    .style("transition-timing-function","linear")
                    .style("pointer-events","none");
                },
                AnimationState::ReturningToDock => {
                    _ = el
                    .style("transform",&format!("translate(0px,0px)"))
                    .style("transition",&format!("transform {}ms",SHIFT_TIME))
                    .style("transition-timing-function","linear")
                    .style("pointer-events","none");
                }       
            }
        }   
    }
    pub fn transition_end(&self,file_id:Uuid) {
        let data = self.icon_data.get_untracked().get(&file_id).cloned().unwrap();
        let el = data.get_untracked().el.get_untracked().unwrap();
        let idx = self.list.get_untracked().iter().position(|&id|id==file_id).unwrap();
        let left = self.left_idx.get_untracked().get(&idx).cloned().unwrap();
        let top = self.top_idx.get_untracked().get(&idx).cloned().unwrap();
        _ = el
                    .style("pointer-events","auto")
                    .style("transform","translate(0px,0px)")
                    .style("z-index","auto")
                    .style("transition","transform 0ms")
                    .style("top",&format!("{}px",top));
        data.update(|data|data.animation_state.set(None));
        // match state , left -> get left pos for idx-1 etc
        /*
         AnimationState::OnDock => {
                    let el = el
                    .style("pointer-events","auto")
                    .style("transform","translate(0px,0px)")
                    .style("z-index","auto")
                    .style("transition","transform 0ms");
                    //data.update(|data|{data.left=el.get_bounding_client_rect().left();data.top=el.get_bounding_client_rect().top();})
                },
         */
    }
}

#[component]
pub fn Dock() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let items = state.dock.list;
    let dock_ref = create_node_ref::<Div>();
    let queue = state.dock.insert_queue;
    let set = state.dock.insert_set;
    let queue = create_memo(move |_| queue());
    let icon_list = create_rw_signal(Vec::<HtmlElement<Div>>::new());
    create_effect(move|_| {
        let rect = (*dock_ref().unwrap()).get_bounding_client_rect();
        _ = dock_ref().unwrap()
        .style("height",&format!("{}px",rect.height()))
        .style("width",&format!("{}px",rect.width()));
        
    });
    create_effect(move |_| {
        if icon_list().len() == items().len() {
            for (idx,icon) in icon_list().into_iter().enumerate() {
                let left = state.dock.left_idx.get_untracked().get(&idx).cloned().unwrap();
                let top = state.dock.top_idx.get_untracked().get(&idx).cloned().unwrap();
                _ = icon.style("position","absolute")
                .style("left",&format!("{}px",left))
                .style("top",&format!("{}px",top));
            }
        }
    });

    create_effect(move |_| {
        if !queue().is_empty() {
            state.dock.handle_insert_msg();
        }
    });

  
    let set = create_memo(move |_| set());

 

    view!{
        <div _ref=dock_ref
        class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 height-20 flex"
        >
        <For 
        each=move || items().into_iter().enumerate()
        key=|(_,id)| *id
        children=move |(idx,file_id)| {
            leptos::logging::log!("render {idx}");
            let id_left = if idx > 0 {
                Some(items().get(&idx-1).cloned().unwrap())
            } else {
                None
            };
            let id_right = if idx + 1 < items.get_untracked().len() {
                Some(items.get_untracked().get(&idx+1).cloned().unwrap())
            } else {
                None
            }; 
            view! {
                <Icon file_id id_left id_right parent=dock_ref idx icon_list/>
                }
        }
        />   
        </div>
    }
}

pub fn set_img_transparent(ev:&DragEvent) -> anyhow::Result<()> {
    let transparent_image = web_sys::window()
    .ok_or(anyhow!("window"))?
    .document()
    .ok_or(anyhow!("document"))?
    .create_element("img")
    .map_err(|_|anyhow!("create_element"))?
    .dyn_into::<web_sys::HtmlImageElement>()
    .map_err(|_|anyhow!("dyn html image"))?;
    transparent_image.set_src("data:image/gif;base64,R0lGODlhAQABAIAAAP///////yH5BAEAAAAALAAAAAABAAEAAAIBRAA7");
    let data_transfer = ev.data_transfer().ok_or(anyhow!("data transfer"))?;
    data_transfer.set_drag_image(&transparent_image, 0, 0);
    Ok(())
}

#[component]
pub fn Icon(file_id:Uuid, id_left:Option<Uuid>,id_right:Option<Uuid>,parent:NodeRef<Div>,idx:usize, icon_list:RwSignal<Vec<HtmlElement<Div>>>) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let icon_ref = create_node_ref::<leptos::html::Div>();
    let is_being_dragged = move || state.dock.drag_data.with(|data|data.map(|data|data.dragging_id==file_id));
    let img_src = move || state.img_src(file_id);
    let is_running = move || state.is_running(file_id);
    let run_app = move || state.run_app(file_id,0.);
    let (is_jumping,set_jumping) = create_signal(false);
    let left = state.dock.icon_data.get_untracked().get(&file_id).unwrap().get_untracked().left;
    let update_left = move |l| {
        state.dock.left_idx.update_untracked(move |data|{data.insert(idx,l);});
        left.set(l);
    };  
    let update_top = move |top| state.dock.top_idx.update_untracked(move |data|{data.insert(idx,top);});    
    let update_node = move |icon_ref| state.dock.icon_data.get_untracked().get(&file_id).unwrap().update_untracked(move |data|data.el=icon_ref);
    let animation_state = state.dock.icon_data.get_untracked().get(&file_id).unwrap().get_untracked().animation_state;
    let list = state.dock.list;
    create_effect( move |_| {
        let left = icon_ref().unwrap().offset_left();
        let top = icon_ref().unwrap().offset_top();
        update_left(left as f64);
        update_top(top as f64);
        icon_list.update(|list|list.push(icon_ref().unwrap()));
      
    });
    create_effect(move |_| update_node(icon_ref));
    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();
    create_effect(move |_| {
        let (x,y) = (
            (animation_state() == Some(AnimationState::BeingDragged)).then(move||x()),
            (animation_state() == Some(AnimationState::BeingDragged)).then(move||y()),
        );
        request_animation_frame(move || 
            state.dock.animate_state(file_id,id_left,id_right,x,y)
        );
    });


    view!{
        <div _ref=icon_ref
        class="z-10 w-[4.5rem] h-[4.5rem]"
        class=("animate-jump",move || is_jumping())
            on:dragstart=move|ev| {    
                set_img_transparent(&ev).unwrap();
                state.dock.drag_event(DragEventMsg::DragStart(file_id,ev));
            }

            on:dragover=move |ev| {
                state.dock.drag_event(DragEventMsg::DragOver(file_id,ev));
            }

            on:dragend=move|_|{
                state.dock.drag_event(DragEventMsg::DragEnd);
            }
            on:click =  move |_| 
            if !is_running() {
                run_app();
                set_jumping(true);
            }
            on:transitionend = move |ev| {
                state.dock.transition_end(file_id);
            }
        >
           
        <div> 
        <img class="rounded-md" src=img_src />
        </div>  
       

        <Show when=move||!is_being_dragged().unwrap_or_default() fallback=||view!{}>
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> 
        </div>
        </Show>
        </div>
    }
}
*/
/* //take 2
#[derive(PartialEq,Clone,Copy,Debug)]
pub struct ZBoostDock(pub RwSignal<bool>);

pub type DragOverId = Uuid;
#[derive(Debug,PartialEq,Copy,Clone,Eq,Hash)]
pub enum DragOverMsg{
    ShiftLeft(DragOverId),
    ShiftRight(DragOverId),
    InsertLeft(DragOverId),
    InsertRight(DragOverId),
}
#[derive(Debug,PartialEq,Copy,Clone,Eq,Hash)]
pub enum DragMsg{
    DragOver(DragOverMsg),
    DragOut,
    Drop
}

impl DragOverMsg{
    pub fn drag_over_id(self) -> Uuid {
        match self {
            DragOverMsg::ShiftLeft(id) => id,
            DragOverMsg::ShiftRight(id) => id,
            DragOverMsg::InsertLeft(id) => id,
            DragOverMsg::InsertRight(id) => id,
        }
    }
}
#[derive(Debug,PartialEq,Copy,Clone,Eq,Hash)]

pub enum AnimateMsg{
    Left(Uuid),
    Right(Uuid),
    Over((Uuid,usize)),
}
#[derive(Clone,Default,Copy)]
pub struct DockList{
    icon_set:RwSignal<HashSet<Uuid>>,
    list:RwSignal<Vec<Uuid>>,
    msg_set:RwSignal<HashSet<DragMsg>>,
    queue:RwSignal<VecDeque<DragMsg>>,
    dimensions_map:RwSignal<HashMap<Uuid,IconDimensions>>,
    dock_dimensions:RwSignal<DockDimensions>,
    drag_data:RwSignal<Option<DragData>>,
    drag_msg:RwSignal<Option<DragMsg>>,
    animate_msg:RwSignal<Option<AnimateMsg>>,
    css_map:RwSignal<HashMap<Uuid,IconCss>>,
}
#[derive(Debug,PartialEq,Clone,Default)]
pub struct IconCss {
    position:IconPosition,
    transform:IconTransform,
    pointer_events:bool,
    z_index:Option<u8>,
    /// i.e if some transition:transform {usize}
    transition_transition:Option<usize>, 
    transition_linear:bool,
}
#[derive(Debug,PartialEq,Clone,Default)]
pub enum IconTransform{
    #[default]
    None,
    ReturnToPos,
    MousePosition,
    Left(f64),
    Right(f64,)
}

#[derive(Debug,PartialEq,Clone,Default)]
pub enum IconPosition{
    #[default]
    Static,
    Absolute{
        left:f64,
        top:f64,
    }
}

#[derive(Debug,PartialEq,Clone,Copy,Default)]
pub struct DockDimensions{
    left:f64,
    top:f64
}
#[derive(Debug,PartialEq,Clone,Default,Copy)]
pub struct DragData{
    dragging_id:Uuid,
    dock_idx:usize,
    // offset is the distance into the icon, from left and top respectively
    offset_x:f64,
    offset_y:f64,
    // mouse pos is the absolute position relative to the view port (at the time of click)
    mouse_pos_x:f64,
    mouse_pos_y:f64,
    // limb_orgin_left and limbo_origin_top when the icon is in limbo
    // the icon's position is translated from these origin values to underneath the mouse cursor.
    limbo_origin_left:f64,
    limbo_origin_top:f64,
    // The dock's dimensions at the moment prior to the item going into limbo.
    limbo_dock_dimensions:DockDimensions,
    // if in limbo don't show it's space in the dock, and on drop remove it from the dock.
    in_limbo:bool,
}

#[derive(Clone,Default,Debug,PartialEq,Copy)]
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


    pub fn new(list:Vec<Uuid>) -> Self {
        Self{  
            icon_set:create_rw_signal(list.clone().into_iter().collect()),
            list:create_rw_signal(list),
            msg_set:create_rw_signal(HashSet::new()),
            queue:create_rw_signal(VecDeque::new()),
            dimensions_map:create_rw_signal(HashMap::new()),
            dock_dimensions:create_rw_signal(DockDimensions::default()),
            drag_data:create_rw_signal(None),
            drag_msg:create_rw_signal(None::<DragMsg>),
            animate_msg:create_rw_signal(None::<AnimateMsg>),
        }
    }
    pub fn insert(&mut self,  idx_b:usize,id_a:Uuid,) {
        if !self.icon_set.with(|set|set.contains(&id_a)) {
            self.list.update(|list|{list.insert(idx_b,id_a);});
            self.icon_set.update(|set|{set.insert(id_a);});
        } else {
            let idx_a = self.list.with(|list|list.iter().position(|id_b| *id_b==id_a).unwrap());
            self.list.update(|list|{list.remove(idx_a);});
            self.list.update(|list|list.insert(idx_b,id_a));
        }
    }
    pub fn remove(&mut self, id_a:Uuid) {
        let idx_a = self.list.with(|list|list.iter().position(|id_b| *id_b==id_a).unwrap());
        self.list.update(|list|{list.remove(idx_a);});
        self.icon_set.update(|list|{list.remove(&id_a);});
    }
    pub fn push_msg(&self,msg:DragMsg) {
        if !self.msg_set.with(|set|set.contains(&msg)) {
            self.queue.update(|queue|queue.push_back(msg));
            self.msg_set.update(|set|{set.insert(msg);});
        }
    }
    /* 
    pub fn pop_msg(&mut self) -> Option<DragOverMsg> {
        if let Some(msg) = self.queue.pop_front() {
            let msg_set = self.msg_set;
            let remove = move || msg_set.update(|set|{set.remove(&msg);});
            set_timeout(remove,std::time::Duration::from_millis(250));
            Some(msg)
        } else {
            None
        }
    }*/
    /// We read the first message if available in queue.
    /// We set the drag over msg signal to that value. ( This will queue any animation or other data updates in icons.)
    /// After 250ms we'll clear the message, pop the queue, and change the msg set, which is what gurantees uniqueness.
    pub fn handle_message(&self) {
        let queue = self.queue;
        leptos::logging::log!("{:?}",queue());
        queue.update(|queue|{
            if let Some(msg) = queue.pop_front() {
                leptos::logging::log!("{msg:?}");
                let icon_dimensions = self.dimensions_map;
                let data = self.drag_data;
                let list = self.list;
                let msg_set = self.msg_set;
                let drag_msg = self.drag_msg;
                let animate_msg = self.animate_msg;
                drag_msg.set(Some(msg));
            
                let update_and_remove = move || {
                    match msg {
                        DragMsg::DragOver(msg) => {
                            match msg {
                                DragOverMsg::ShiftLeft(drag_over_id) => {
                                    let drag_over_idx = list.with_untracked(|list|list.iter().position(|id|*id==drag_over_id)).unwrap();
                                    let new_left = icon_dimensions.with_untracked(|icons|icons.get(&drag_over_id).cloned().unwrap());
                                    let dragging_id = list.with_untracked(|list|list.get(drag_over_idx-1).cloned().unwrap());
                                    let old_left = icon_dimensions.with_untracked(|icons|icons.get(&dragging_id).cloned().unwrap());
                                    //leptos::logging::log!("drag_over_idx : {drag_over_idx}\n new_left : {new_left:?} \n old_left : {old_left:?}\n");
                                    icon_dimensions.update(|icons|{
                                        icons.insert(dragging_id,new_left);
                                });
                                icon_dimensions.update(|icons|{
                                    icons.insert(drag_over_id,old_left);
                                });
                                animate_msg.set(Some(AnimateMsg::Over((drag_over_id,new_left.left as usize)))); 
                                list.update(|list|list.swap(drag_over_idx-1,drag_over_idx));
                                animate_msg.set(None); 
                                data.update(|data|{if let Some(data) = data.as_mut(){data.dock_idx=drag_over_idx}});
                             
                                },
                                DragOverMsg::ShiftRight(id) => leptos::logging::log!("todo"),
                                DragOverMsg::InsertLeft(id) => leptos::logging::log!("todo"),
                                DragOverMsg::InsertRight(id) => leptos::logging::log!("todo"),
                            }
                        }
                        DragMsg::DragOut => leptos::logging::log!("todo"),
                        DragMsg::Drop => leptos::logging::log!("todo"),
                    }
                    msg_set.update(|set: &mut HashSet<DragMsg>|{set.remove(&msg);});
                };
                match msg {
                    DragMsg::DragOver(msg) => match msg {
                        DragOverMsg::ShiftLeft(id) => animate_msg.set((Some(AnimateMsg::Left(id)))),
                        DragOverMsg::ShiftRight(id) => animate_msg.set((Some(AnimateMsg::Right(id)))),
                        DragOverMsg::InsertLeft(_) => todo!(),
                        DragOverMsg::InsertRight(_) => todo!(),
                    },
                    DragMsg::DragOut => todo!(),
                    DragMsg::Drop => todo!(),
                }
                set_timeout(move || request_animation_frame(move ||update_and_remove()),std::time::Duration::from_millis(2000));
           
            }
        });
    
      
       

            /* 
              if let Some(msg) = self.pop_msg() {
            let mut data = self.drag_data.clone().unwrap();
            let update_mouse_pos = |this:&Self,new_idx:usize,data:&mut DragData| {
                if let Some(IconDimensions{left,..}) = this.dimensions_map.get(&new_idx).cloned() {
                    
                    let mouse_pos_x = left + data.offset_x;
                    leptos::logging::log!("old mouse_pos_x: {}\n new mouse_pos_x : {}",data.mouse_pos_x,mouse_pos_x);
                    data.mouse_pos_x = mouse_pos_x;
                }                    
            };
            let dragging_id = data.dragging_id;
            let dragging_idx = data.dock_idx;
            match msg {
                DragOverMsg::ShiftLeft(drag_over_id) => {
                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    data.dock_idx = drag_over_idx;
                    self.list.swap(dragging_idx,drag_over_idx);
                    update_mouse_pos(&self,drag_over_idx,&mut data);
                    self.drag_data = Some(data);
                },
                DragOverMsg::ShiftRight(drag_over_id) => {

                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    data.dock_idx = drag_over_idx;
                    self.list.swap(dragging_idx,drag_over_idx);
                    update_mouse_pos(&self,drag_over_idx,&mut data);
                    self.drag_data = Some(data);
                },
                DragOverMsg::InsertLeft(drag_over_id) => {

                    let drag_over_idx = self.list_idx(drag_over_id).unwrap();
                    self.insert(drag_over_idx,dragging_id);                
                    data.in_limbo = false;
                    data.dock_idx = drag_over_idx;
                    update_mouse_pos(&self,drag_over_idx,&mut data);
                    self.drag_data = Some(data);
                }
                DragOverMsg::InsertRight(drag_over_id) => {

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

                    let IconDimensions{left,top,width} = self.dimensions_map.get(&dragging_idx).cloned().unwrap();
                    data.left = left-width/2.;
                    data.top = top;
                    data.in_limbo = true;
                    self.drag_data = Some(data);
                }
                DragOverMsg::Drop => {
                    self.remove(dragging_id);
                    self.drag_data = None;
                }
            
            }
            leptos::logging::log!("msg:{msg:?}
                data:{data:?}
            ") 
            } 
         */
    }
    
}



#[component]
pub fn Dock() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    provide_context(ZBoostDock(create_rw_signal(false)));
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let dock_ref = create_node_ref::<Div>();

    let items = state.dock_list.list;
    let has_message = move ||!(state.dock_list.queue)().is_empty();
    let handle_msg = move || state.dock_list.handle_message();
    let set_pos = move |dock_dimensions| state.dock_list.dock_dimensions.set(dock_dimensions);
 
    create_effect(move |_| {
        if has_message() {
            handle_msg()
        }
    });
    create_effect(move |_| {
        let rect = (*dock_ref().unwrap()).get_bounding_client_rect();
        //leptos::logging::log!("dock dimensions changed.");
        set_pos(DockDimensions{ left: rect.left(), top: rect.top() });
    });

    view!{
        <div>
        <AbyssTarp/>
        <div 
        _ref = dock_ref
        class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 height-20 flex"
        class=("z-[99]",move || zboost())
        >
        
        <For 
        each=move || items().into_iter()
        key=|id| *id
        children=move |file_id| 
            view! {
                <Icon file_id parent=dock_ref/>
                }
        />   
        </div>
      </div>
    }
}  

#[component]
pub fn AbyssTarp() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let zboost: RwSignal<bool> = expect_context::<ZBoostDock>().0;
    let push_msg = move |msg| state.dock_list.push_msg(msg);
    let in_limbo =move ||  state.dock_list.drag_data.with(|data|data.map(|data|data.in_limbo)).unwrap_or_default();
    let handle = window_event_listener(ev::dragover, move |ev| {
        ev.prevent_default();
    });
    on_cleanup(move || handle.remove());

    view!{
        <div class="w-[99vw] h-[99vh] -z-50 absolute top-0"
            class=("z-[98]",move || zboost())
            on:drop=move|_| push_msg(DragMsg::Drop)
            
            on:dragover=move |ev| {
                ev.prevent_default();
                if !in_limbo() {
                    push_msg(DragMsg::DragOut)
                }               
            }
            >
        </div>
    }
}


#[component]
pub fn Icon(file_id:Uuid,parent:NodeRef<Div>) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let icon_ref = create_node_ref::<leptos::html::Div>();

    let this_idx = Signal::derive(move || state.dock_list.list
        .with(|list|
            list.iter()
                .position(move |&id| id==file_id))
        .expect(&format!("expect {file_id} to have and idx")));
    let set_drag_data = move |data| state.dock_list.drag_data.set(data);
    let x_y = move || state.dock_list.drag_data.with(|data|data.map(|data|(data.mouse_pos_x,data.mouse_pos_y)));
    let in_limbo = move || state.dock_list.drag_data.with(|data|data.map(|data|data.in_limbo)).unwrap_or_default();
    let dragged_id = move || state.dock_list.drag_data.with(|data|data.map(|data|data.dragging_id));
    let is_being_dragged = move || dragged_id().map(|id|id==file_id).unwrap_or_default();
    let dock_idx = move || state.dock_list.drag_data.with(|data|data.map(|data|data.dock_idx));
    let limbo_left_top_origin = move || state.dock_list.drag_data.with(|data|data.map(|data|(data.limbo_origin_left,data.limbo_origin_top)));
    let img_src = move || state.img_src(file_id);
    let is_running = move || state.is_running(file_id);
    let run_app = move || state.run_app(file_id,0.);
    let (is_dragging,set_is_dragging) = create_signal(false);
    let limbo_dock_dimensions = move || state.dock_list.drag_data.with(|data|data.map(|data|data.limbo_dock_dimensions));
    let push_msg = move |msg| {
        state.dock_list.push_msg(msg);
    };
    let pos_map = move |icon_dimensions| state.dock_list.dimensions_map.update(|map|{map.insert(file_id,icon_dimensions);});
    let leptos_use::UseMouseReturn {
        x, y, ..
    } = leptos_use::use_mouse();
    let (is_jumping,set_jumping) = create_signal(false);
    let animate_msg = state.dock_list.animate_msg;
    let is_being_animated = create_memo(move |_| if let Some(msg) = animate_msg() {
        leptos::logging::log!("{msg:?}");
        match msg {
        AnimateMsg::Left(id) => id==file_id,
        AnimateMsg::Right(id) => id==file_id,
        AnimateMsg::Over((id,_)) => id == file_id,
    }} else {false});
  
   /*  // returns the left attribute value of the icon that was dragged over, or the icon to the right of it (if insert right), or none if no icon dragover.
    let left_for_drag_over = ||  (state.dock_list.drag_msg)().and_then(|msg|
        if let DragMsg::DragOver(msg) = msg {
            let id = match msg {
                DragOverMsg::InsertRight(id) => {
                    let position = state.dock_list.list.with(|list|list.iter().position(|&iter_id| iter_id  ==  id).unwrap() + 1);
                    let id = state.dock_list.list.with( move |list|list.get(position).cloned()).unwrap();
                    id
                },
                msg => msg.drag_over_id()
            };
            state.dock_list.dimensions_map.with(|map|map.get(&id).map(|icon|icon.left))
        } else {None});*/
    let left_for_left = Signal::derive(move || {
        if this_idx() > 0 {
            let left_id = state.dock_list.list.with(|list|list.get(this_idx()-1).cloned()).unwrap();
            state.dock_list.dimensions_map.with(|map|map.get(&left_id).map(|icon|icon.left))
        } else {
            None
        }
    });
    let left_for_self =  Signal::derive(move || state.dock_list.dimensions_map.with(|map|map.get(&file_id).map(|icon|icon.left)));
    let offset_x =   move || state.dock_list.drag_data.with(|data|data.map(|data|data.offset_x));
    let shift_left_for_left_neighbor = move || {
        let left_id = state.dock_list.list.with(|list|list.get(this_idx()-1).cloned()).unwrap();
        state.dock_list.msg_set.with(|set|set.contains(&DragMsg::DragOver(DragOverMsg::ShiftLeft(left_id))))
    };
 
    // sets initial left
    create_effect( move |_| {
        let rect = (*icon_ref().unwrap()).get_bounding_client_rect();
        let left = rect.left();
        let top = rect.top();
        let width = rect.width();
        pos_map(IconDimensions{left,top,width});
    });

    create_effect( move|_| if is_jumping() {
        set_timeout(move || set_jumping(false), std::time::Duration::from_millis(250))
    });

    create_effect( move |_| {
        let style = {
            let movement_style = 
            if is_being_dragged() {
                let (mouse_pos_x,mouse_pos_y) = x_y().unwrap();
                let offset_x = offset_x().unwrap();
                let icon_left = left_for_self().unwrap();
                let x = x();
                let left = x - (offset_x + icon_left);
                let top = y() - mouse_pos_y;
                //leptos::logging::log!("x : {x}  - icon_left {icon_left} - offset_x: {offset_x}   = left : {left}  ");
                format!("
                pointer-events: none;
                z-index:100;
                transform:translate({}px,{}px);",
                left,
                top,
                ) 
            } else  {
                format!("
                position:static;
                transform:translate(0px,0px);
                transition: transform 0.25s; 
                transition-timing-function: linear;
                ")
            };
            let animation =  if is_being_animated() {
                leptos::logging::log!("animate");
                match animate_msg().unwrap() {
                    AnimateMsg::Left(_) => {
                        let left =  left_for_left.get_untracked().unwrap() - left_for_self.get_untracked().unwrap();
                        format!("
                position:static;
                transition: transform 1s; 
                transition-timing-function: linear;
                transform:translate({}px,0px);",
                left,
                ) 
                    }
                    AnimateMsg::Right(_) => todo!(),
                    AnimateMsg::Over((_,left)) => format!("
                    position:absolute;
                    left:{left}px;
                    ")
            }} else {
                String::new()
            };
            let limbo_style = 
            if in_limbo() && is_being_dragged() {
                let (limbo_left,limbo_top) = limbo_left_top_origin().unwrap();
                let DockDimensions{left:limbo_dock_left,top:limbo_dock_top} = limbo_dock_dimensions().unwrap();
                let left = limbo_left - limbo_dock_left;
                let top = limbo_top - limbo_dock_top;
                format!("
                    position:absolute;
                    left:{left}px;
                    top:{top}px;")
            } else {
                String::default()
            };
           
            movement_style + &animation + &limbo_style
        };
        let div =icon_ref().unwrap();
        request_animation_frame(move || {_ = div.attr("style",style);});
    });


    view!{
        <div _ref=icon_ref
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
                let rect = (*parent().unwrap()).get_bounding_client_rect();
                let limbo_dock_dimensions = DockDimensions{ left: rect.left(), top: rect.top() };

                let drag_data = DragData{dragging_id:file_id,dock_idx:this_idx(),
                    mouse_pos_x,
                    mouse_pos_y,
                    offset_x,
                    offset_y,
                    limbo_origin_left:left,
                    limbo_origin_top:top,
                    limbo_dock_dimensions,
                    in_limbo:false,
                };
                set_drag_data(Some(drag_data));
                set_is_dragging(true);
            }

            on:dragover=move |ev| {

                if is_being_dragged()
                 {
                    return ();
                }
                let this_idx = this_idx();
               
                let el = event_target::<web_sys::HtmlElement>(&ev);
               
                let cursor_position = ev.client_x() - el.get_bounding_client_rect().left() as i32;
               
                let cursor_on_left = cursor_position < el.offset_width() / 2;
               
                let cursor_on_right = cursor_position > el.offset_width() / 2;
               
                let self_is_leftmost = this_idx == 0;
               
                let dragging_elem_is_left_neighbor = dock_idx().unwrap() + 1 == this_idx;
               
                let dragging_elem_is_right_neighbor = dock_idx().unwrap()  == this_idx + 1;
               
                
                if !self_is_leftmost &&
                    cursor_on_left &&
                    dragging_elem_is_right_neighbor {
                        push_msg(DragMsg::DragOver(DragOverMsg::ShiftRight(file_id)))
                } else if !self_is_leftmost &&
                    cursor_on_right &&
                    (dragging_elem_is_left_neighbor) //||  shift_left_for_left_neighbor())
                     {
                        push_msg(DragMsg::DragOver(DragOverMsg::ShiftLeft(file_id)))
                } else if 
                    cursor_on_left && 
                    !self_is_leftmost && 
                    in_limbo() {
                        push_msg(DragMsg::DragOver(DragOverMsg::InsertLeft(file_id)))
                } else if 
                    cursor_on_right && 
                    in_limbo() {
                        push_msg(DragMsg::DragOver(DragOverMsg::InsertRight(file_id)))
                } 
            }

            on:dragend=move|_|{
                set_is_dragging(false);
                set_drag_data(None);
            }
            on:click =  move |_| 
            if !is_running() {
                run_app();
                set_jumping(true);
            }
        >
           
        <div> 
        <img class="rounded-md" src=img_src />
        </div>  
       

        <Show when=move||!is_being_dragged() fallback=||view!{}>
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
 */

// take 1 

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