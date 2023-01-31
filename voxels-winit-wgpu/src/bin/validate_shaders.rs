use std::{env, path};

fn main() {
    let mut validator =
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), Default::default());

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let args: Vec<String> = env::args().collect();
    for shader in args[1].split(";") {
        println!("\n{}", shader);
        let voxels_winit_wgpu::gpu::shader::load::Source { code, .. } =
            voxels_winit_wgpu::gpu::shader::load::load(path::Path::new(shader));
        match naga::front::wgsl::parse_str(&code) {
            Err(e) => e.emit_to_stderr(&code),
            Ok(module) => {
                if let Err(e) = validator.validate(&module) {
                    eprintln!("{}", e);
                    // match e {
                    //     naga::valid::ValidationError::Function { error, .. } => {
                    //         eprintln!("{}", error);
                    //     }
                    //     _ => {}
                    // }
                }
            }
        };
    }
}
