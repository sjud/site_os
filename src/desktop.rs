use super::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
#[component]
pub fn Desktop() -> impl IntoView{
    view!{
    <div 
        class="flex flex-wrap pt-8 pl-4 pr-4 pb-16 w-[95vw] h-[95vh] m-auto -z-50" 
        on:dragover = move |ev| ev.prevent_default()>
        <DesktopGrid/>
    </div>
      
    }
}
#[derive(PartialEq,Clone,Debug,Default)]
pub struct RowColsCount{
    rows:u8,
    cols:u8,
}

#[component]
pub fn DesktopGrid() -> impl IntoView {
    let system =expect_context::<SystemState>().0;
    let file_ids = create_read_slice(system,|state|
        state.file_ids_direct_children_of_path(
        PathBuf::from_str("/").unwrap())
    );
    let settings = create_read_slice(system,|state|state.settings.desktop.clone());
    // we need the node ref to examine the dimension of our box.
    let grid_el = create_node_ref::<leptos::html::Div>();
    let (grid_rows_cols,set_grid_rows_cols) = create_signal(RowColsCount::default());
    let (grid_nodes,set_grid_nodes) = create_signal(HashMap::new());
    let div_grid = move || {
        let mut views = Vec::new();
        let RowColsCount{rows,cols} = grid_rows_cols();
        for r in  0..rows {
            for c in  0..cols {
                let key = format!("grid_space_{r}_{c}");
                let div = create_node_ref::<leptos::html::Div>();
                let size = settings.get_untracked().icon_size;
                views.push(
                    view!{
                        <div id=&key _ref=div class="bg-blue-50 m-1"
                            style=format!("width:{size}rem;height:{size}rem;")
                        >
                        </div>
                    }.into_view()
                );
                set_grid_nodes.update(|map|{map.insert(key,div);});
            }
        }
    };
    create_effect(move |_| {
        if let Some( binding) = grid_el.get() {
            let size = settings.get_untracked().icon_size * 16.;
            let div = binding.deref();
            let width = div.client_width();
            let height = div.client_height();
            let rows = height as f32 / size;
            let cols = width as f32 / size;
            let row_cols_count = RowColsCount{
                rows: rows as u8,
                cols: cols as u8,
            };

            // the height/width not including margins and padding
            set_grid_rows_cols(row_cols_count);
        }
    });
    view!{
        <div _ref=grid_el>
            {div_grid}
        </div>
    }
}

#[component]
pub fn DesktopIcon(file_id:Uuid) -> impl IntoView {
    let system_runtime =expect_context::<SystemState>().0;
    let img_src = create_read_slice(system_runtime,move |state|state.img_src(file_id));
    let run_app = create_write_slice(system_runtime,
        move |state,_|state.run_app(file_id,0.));
    let select_file = create_write_slice(system_runtime,
        move |state,()| state.select_file(file_id));

    let inner = view!{
        <button 
        class="pl-2 pr-2 pt-1 pb-1 transition-all ease-linear duration-100 hover:scale-[1.50] hover:-translate-y-2"
            id=file_id.to_string()
            on:pointerdown =  move |_| select_file(())
            on:dblclick = move |_| run_app(())
         >
        <img src=img_src/>
        </button> 
    };
  
}