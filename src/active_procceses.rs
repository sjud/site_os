
use super::*;

#[component]
pub fn ActiveProcesses() -> impl IntoView {
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
    let active_processes = create_read_slice(system_runtime,|state|state.active_proccesses.clone());
    let views = move || {
        let mut views = Vec::new();
        let mut proccesses = active_processes().0.into_iter().map(|val|val).collect::<Vec<(Uuid,system_runtime::ActiveProcess)>>();
        proccesses.sort_by(|(_,p),(_,p1)|p.window_stack_idx.cmp(&p1.window_stack_idx));
        for (file_id,p) in proccesses {
            let file_type = system_runtime.get_untracked().file_type(file_id);
            let z = format!("z-{}",p.window_stack_idx);
            views.push(
            match file_type {
                system_runtime::FileType::Directory => {
                    view!{
                        <div class=z.clone()>
                        <folder::Folder file_id=file_id/>
                        </div>
                    }
                },
                system_runtime::FileType::File => {
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