use core::slice;
use std::{os::raw::c_char, char::decode_utf16};

use interoptopus::{ffi_function, ffi_type, inventory};
use ttf2mesh::*;

#[ffi_type]
#[repr(C)]
pub struct BufferVec2 {
    pub data : *const Vector2,
    pub len : u32
}

impl BufferVec2 {
    pub fn new(data : *const Vector2, len : u32) -> Self {
        Self {
            data,
            len
        }
    }
}

#[ffi_type]
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
impl Vector2{
    fn new(x : f32, y : f32) -> Self {
        Vector2 { x, y }
    }
}
#[ffi_type]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3{
    fn new(x : f32, y : f32, z : f32) -> Self {
        Vector3 { x, y, z }
    }
}


#[ffi_function]
#[no_mangle]
pub extern "C" fn my_function(input: Vector2) {
    println!("{}", input.x);
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn display_numbers(buffer : *const u8, length : u32){
    let data = slice::from_raw_parts(buffer as *mut u8, length as usize).to_vec();
    for v in data{
        println!("{}\n", &v);
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn char_mesh_2d(file_buf : *const u8, length : u32, ch : u32, quality : u8) -> BufferVec2 {
    let data = slice::from_raw_parts(file_buf as *mut u8, length as usize).to_vec();
    let mut file = TTFFile::from_buffer_vec(data).unwrap();
    let c = char::from_u32(ch).unwrap();
    let mut glyph = file.glyph_from_char(c).unwrap();
    let mesh2d : Mesh<Mesh2d> = glyph.to_2d_mesh(Quality::Custom(quality)).unwrap();
    let mut vs : Vec<Vector2> = mesh2d.iter_vertices().map(|v| Vector2::new(v.val().0, v.val().1)).collect();
    let len = vs.len() as u32;
    let ptr = vs.as_mut_ptr();
    std::mem::forget(vs); // so that it is not destructed at the end of the scope
    BufferVec2::new(ptr, len)
}

// #[ffi_function]
// #[no_mangle]
// pub unsafe extern "C" fn char_mesh_3d(file_buf : *const u8, length : u32, ch : char, quality : u8, depth : f32) {
//     let data = slice::from_raw_parts(file_buf as *mut u8, length as usize).to_vec();
//     let mut file = TTFFile::from_buffer_vec(data).unwrap();
//     let mut glyph = file.glyph_from_char(ch).unwrap();
//     let mesh2d : Mesh<Mesh3d> = glyph.to_3d_mesh(Quality::Custom(quality), depth).unwrap();
//     let vs : Vec<Vector3> = mesh2d.iter_vertices().map(|v| Vector3::new(v.val().0, v.val().1, v.val().2)).collect();

// }

// This defines our FFI interface as `ffi_inventory` containing
// no constants, a single function `my_function`, no additional
// types (types are usually inferred) and no codegen patterns.
inventory!(ffi_inventory, [], [my_function, char_mesh_2d, display_numbers], [], []);

#[cfg(test)]
mod tests {
    use interoptopus::util::NamespaceMappings;
    use interoptopus::{Error, Interop, Library};
    use interoptopus_backend_csharp::Unsafe;

    use crate::ffi_inventory;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn bindings_csharp() -> Result<(), Error> {
        use interoptopus_backend_csharp::{Config, Generator};

        Generator::new(
            Config {
                class: "TTFInterop".to_string(),
                dll_name: "ttf2meshbind.dll".to_string(),
                namespace_mappings: NamespaceMappings::new("Stride.TTF"),
                use_unsafe : Unsafe::UnsafePlatformMemCpy,
                ..Config::default()
            },
            ffi_inventory()
        )
        .write_file("bindings/csharp/TTFInterop.cs")?;

        Ok(())
    }
}
