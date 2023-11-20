use super::*;



#[component]
pub fn Folder(file_id:Uuid) -> impl IntoView{
    let system_runtime =expect_context::<SystemState>().0;

    let path = create_read_slice(system_runtime, move |state| 
        state.path_from_file_id(file_id)
    );

    let children_ids = create_read_slice(system_runtime, move |state| 
        state.file_ids_direct_children_of_path(std::path::PathBuf::from_str(&path()).unwrap()));
    
    let children = move || {
        let mut views = Vec::new();
        for child_id in children_ids() {
            views.push(
                view!{
                }
            );
        }
        views.collect_view()
    };

    view!{
        <div class="resize w-9/12 h-9/12">
            {children}
        </div>
    }
}