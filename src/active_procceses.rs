
use super::*;

#[component]
pub fn ActiveProcesses() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let active_processes =state.active_proccesses;
    let views = move || {
        let mut views = Vec::new();
        let mut proccesses = active_processes().0.into_iter().map(|val|val).collect::<Vec<(Uuid,global_state::ActiveProcess)>>();
        proccesses.sort_by(|(_,p),(_,p1)|p.window_stack_idx.cmp(&p1.window_stack_idx));
        for (file_id,p) in proccesses {
            let file_type = state.file_type(file_id);
            let z = format!("z-{}",p.window_stack_idx);
            views.push(
            match file_type {
                global_state::FileType::Directory => {
                    view!{
                        <div class=z.clone()>
                        <folder::Folder file_id=file_id/>
                        </div>
                    }
                },
                global_state::FileType::File => {
                    view!{
                        <div class=z.clone()>
                        <application::Application file_id=file_id/>
                        </div>
                    }
                },
            });
        }
        views.collect_view()
    };
    views
}