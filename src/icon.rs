use super::*;

#[component]
pub fn Icon(file_id:Uuid,icon_side_len:u8) -> impl IntoView{
    let system_runtime = expect_context::<RwSignal<SystemRuntime>>();
    let img_src = create_read_slice(system_runtime,move |state|state.img_src(file_id));
    let run_app = create_write_slice(system_runtime,
        move |state,_|state.run_app(file_id,0.));
    view!{
        <div class=format!("w-{icon_side_len}") on:pointerdown= move |_| run_app(())>
            <img src=img_src/>
        </div>
    }
}