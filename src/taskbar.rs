use std::str::FromStr;


use super::*;


#[component]
pub fn TaskBar() -> impl IntoView {
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
    let task_bar_ids = create_read_slice(
        system_runtime,
        |state|state.task_bar_ids());

       view!{
        <div class="bg-slate-700 p-2 backdrop-blur-md fixed bottom-0 bg-opacity-50 rounded-2xl
        left-1/2 transform -translate-x-1/2 flex">
        <div class="flex">
        { move || {
            task_bar_ids().into_iter().map(|id|
            view!{
                <TaskBarItem id />
            }).collect_view()
        }
    }        
            </div>
        </div>
    }
}



#[component]
pub fn TaskBarItem(id:Uuid) -> impl IntoView {
    let system_runtime: RwSignal<SystemRuntime> = expect_context::<RwSignal<SystemRuntime>>();
    let (is_jumping,set_jumping) = create_slice(
        system_runtime,
         move |state| state.is_jumping(id),
        move |state,()| state.set_jumping(id)
    );
    let (is_running,run_app) = create_slice(
        system_runtime,
        move |state| state.is_running(id),
        move |state,()| state.run_app(id,0.) 
    );
    let swap_taskbar = create_write_slice(system_runtime,
        |state,(id_a,id_b)|
        state.swap_taskbar(id_a,id_b)
    );
    let parent_id = move || format!("parent-{}",id);
    let running_marker_id = move || format!("marker-{}",id);
    let (dragover,set_dragover) = create_signal(false);
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
    let img_src = create_read_slice(system_runtime,move |state|state.img_src(id));
    let run_app = create_write_slice(system_runtime,
        move |state,_|state.run_app(id,0.));
 
    view!{
 
        <div 
        on:dragover=move |ev| {
            ev.prevent_default();
            set_dragover(true);
        }
        draggable="true"    
        on:dragstart=  move |ev| { 
            ev.data_transfer().unwrap().set_data("text",id.to_string().as_str()).unwrap();
        } 
        on:dragleave = move |_| set_dragover(false)
        on:drop=move |ev| {
                ev.prevent_default();
                let new_id = ev.data_transfer().unwrap().get_data("text").unwrap();
                set_dragover(false);
                swap_taskbar((id,Uuid::from_str(&new_id).unwrap()));
        }
        class="duration-300"
        style=move || {
        if is_jumping() {
            "transform:translateY(-15px); transition-timing-function: cubic-bezier(0, 0, 0.2, 1);".to_string()
        } else if dragover() {
            "transform:translateY(20px); transition-timing-function: cubic-bezier(0.4, 0, 1, 1);".to_string()
        } else {
            "transform:translateY(0px); transition-timing-function: cubic-bezier(0.4, 0, 1, 1);".to_string()
        }
        }
        >
        <button 
        class="pl-2 pr-2 pt-1 pb-1 transition-all ease-linear duration-100 hover:scale-[1.50] hover:-translate-y-2"
            id=id.to_string()
            on:pointerdown =  move |_| {
                run_app(());
                set_jumping(());
                let timeout = gloo::timers::callback::Timeout::new(325, move || {
                   set_jumping(());
                });
                timeout.forget();
            } 
           
         >
        <img src=img_src/>
        </button>  
        <div 
        class=("invisible", move || !is_running())
        class="rounded-full bg-slate-400 h-1 w-1 ml-auto mr-auto"> </div>
        </div>
    }
}
