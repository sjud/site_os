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
    pub list:HashSet<DockItem>,
    pub dragging_id:Uuid,
    // An ID without an idx.
    pub limbo_id:Option<Uuid>,
}
#[derive(Debug,PartialEq,Clone,Default)]
pub struct FileDraggingData{
    pub file_id:Uuid,
    pub offset_x:usize,
    pub offset_y:usize,
}
impl FileDraggingData{
    /// When we click on a file we store the position of the click as offset incase we drag later.
    pub fn mousedown(&mut self, ev:MouseEvent) {
        let el = event_target::<web_sys::HtmlElement>(&ev);
        let rect = el.get_bounding_client_rect();
        let offset_x = ev.client_x() as f64 - rect.left();
        let offset_y = ev.client_y() as f64 - rect.top();
    }
    /// When we start dragging a file. We need the file id, the img_src of the file.
    pub fn dragstart(&mut self,ev:DragEvent,id:Uuid,img_src:String) {
        self.file_id=id;
        let img = web_sys::HtmlImageElement::new().expect("should create HtmlImageElement");
        img.set_src(img_src);
        ev.data_transfer().unwrap().set_drag_image(&img,self.offset_x,self.offset_y);
        ev.data_transfer().unwrap().set_data("text",id.to_string().as_str()).unwrap();  
    }
}

#[derive(Debug,Copy,Clone)]
pub struct DockItem{
    id:Uuid,
    idx:usize,
}

impl Hash for DockItem {
    // We only has the ID,
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for DockItem{
    fn eq(&self, other: &Self) -> bool {
        // We ignore 
        self.id == other.id
    }
}
impl Eq for DockItem{}

impl PartialOrd for DockItem{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DockItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
    }
}
pub enum Shift{
    Left,
    Right,
}
impl DockList {

    /// When we dragover a docking space, we adjust the docking item indexes.
    pub fn dragover(&mut self,ev:DragEvent,drag_over_id:Uuid,shift:Shift) {
        let id = ev.data_transfer().unwrap().get_data("text").unwrap().parse::<Uuid>().unwrap();
        let drag_over_item = self.list.get(DockItem{id,idx:0}).cloned().unwrap();
        let idx = drag_over_item.idx;
        let dragging_item = DockItem{
            id:Uuid,
            idx,
        };
        self.shift(idx,shift);
        self.list.insert(dragging_item);
    }

    pub fn spaces_count(&self) -> usize {
        self.list.len()
    }

    pub fn put_in_limbo(&mut self,id:Uuid) -> usize {
        // we'll get the actualy idx back since we don't include it in our hash or eq.
        let item = self.list.get(DockItem{id,idx:0}).cloned().unwrap();
        self.limbo_id = Some(item.id);
        self.list.remove(&item);
        item.idx
    }

    pub fn new(ordered_ids:Vec<Uuid>) -> Self{
        Self{  
            list:{
                let mut set = HashSet::new();
                for (idx,id) in ordered_ids.into_iter().enumerate() {
                    set.insert(Self{id,idx});
                }
                set
            },
        }
    }

    /// When we shift an item at id left, we shift the exact item left.
    /// We're expecting there to be an empty space in the docking space array at idx-1
    /// 
    /// When we shift an item right we shift all items at the idx and greater than it
    /// to the right, we're expecting an empty space at the end of the docking space array.
    pub fn shift(&mut self, idx:usize,shift:Shift) {
        for item in self.list.iter_mut() {
            match shift {
                Shift::Left => {
                    if item.idx == idx {
                        item.idx -= 1;
                    }
                },
                Shift::Right => {
                    if item.idx >= idx {
                        item.idx += 1;
                    }
                }
            }
        }
    }


    
}

#[component]
pub fn Dock() -> impl IntoView{
    let system: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let task_bar_ids = create_read_slice(
        system_runtime,
        |state|state.task_bar_ids());

    view!{

    }
}

#[component]
pub fn DockingSpace(children:Children) -> impl IntoView {

}

#[component]
pub fn DockingItem(file_id:Uuid) -> impl IntoView{

}