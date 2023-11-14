use super::*;
use std::collections::HashMap;
use std::ops::Deref;

/*
    get the height and width of the div as it stretches to fill it's parent.
    create a grid of empty divs (for dragging icons around) to fit into the stretched div.

*/
#[derive(PartialEq,Clone,Debug,Default)]
pub struct RowColsCount{
    rows:u8,
    cols:u8,
}
#[component]
pub fn IconGrid(
     /// The file ids will be rendered by icon grid.
     file_ids:Vec<Uuid>,
     /// When files are created in the icon grid, or dragged to the icon grid, it will append them with a '/' to the path.
     #[prop(into)]
     path:String,
     /// We'll append this string to each icon div that we create.
     #[prop(into)]
     icon_class:String,
     /// icon len in pixels
     #[prop(into)]
     icon_side_len:u32,
) -> impl IntoView{
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
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
                views.push(
                    view!{
                        <div id=&key _ref=div class="bg-blue-50 m-1"
                            style=format!("width:{icon_side_len}px;height:{icon_side_len}px;")
                        >
                        </div>
                    }.into_view()
                );
                set_grid_nodes.update(|map|{map.insert(key,div);});
            }
        }
    };

    view!{
        <div _ref=grid_el class="w-full h-full">
            {
                move || {
                    let binding = grid_el.get().unwrap();
                    let div = binding.deref();
                    let width = div.client_width() as u32;
                    let height = div.client_height() as u32;
                    let rows = width / icon_side_len;
                    let cols = height / icon_side_len;
                    // the height/width not including margins and padding
                    set_grid_rows_cols(
                        RowColsCount{
                            rows: rows as u8,
                            cols: cols as u8,
                        }
                    );
                }
            }
            {div_grid}
        </div>
    }

}