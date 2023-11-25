use super::*;



#[component]
pub fn Folder(file_id:Uuid) -> impl IntoView{
    let state =expect_context::<GlobalState>();
    let path = move || state.path_from_file_id(file_id);
    let children_ids = move || state.file_ids_direct_children_of_path(std::path::PathBuf::from_str(&path()).unwrap());
    
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